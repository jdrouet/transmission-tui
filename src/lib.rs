use color_eyre::Result;
use crossterm::event::KeyCode;
use futures::{FutureExt, StreamExt};
use ratatui::Terminal;
use ratatui::prelude::Backend;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use transmission_rpc::types::{Torrent, Torrents};

use crate::view::View;

mod components;
mod runner;
mod view;

pub enum Action {
    RefreshList,
}

pub struct Context {
    action_sender: UnboundedSender<Action>,
}

impl Context {
    fn send_action(&self, action: Action) {
        let _ = self.action_sender.send(action);
    }
}

pub enum Event {
    ActionError(std::io::Error),
    Noop,
    InputEvent(crossterm::event::Event),
    InputError(std::io::Error),
    TorrentListUpdate(Torrents<Torrent>),
    TorrentListUpdateStart,
    TorrentListUpdateError(Box<dyn std::error::Error + std::marker::Send + Sync>),
}

pub struct Application {
    cancellation_token: CancellationToken,
    context: Context,
    event_receiver: UnboundedReceiver<Event>,
    stream: crossterm::event::EventStream,
    task: JoinHandle<()>,
    view: View,
}

impl Application {
    pub fn new(client: transmission_rpc::TransClient) -> Self {
        let cancellation_token = CancellationToken::new();
        let stream = crossterm::event::EventStream::new();
        let view = View::default();

        let (action_sender, action_receiver) = tokio::sync::mpsc::unbounded_channel();
        let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();

        let context = Context { action_sender };

        let runner = crate::runner::Runner::new(client, action_receiver, event_sender);
        let task = tokio::spawn(async move { runner.run().await });

        Self {
            cancellation_token,
            context,
            event_receiver,
            stream,
            view,
            task,
        }
    }
}

impl Application {
    pub async fn run<B>(mut self, mut terminal: Terminal<B>) -> Result<()>
    where
        B: Backend,
    {
        self.view.init(&self.context);

        while !self.cancellation_token.is_cancelled() {
            terminal.draw(|frame| self.view.draw(frame))?;
            let crossterm_event = self.stream.next().fuse();

            tokio::select! {
                _ = self.cancellation_token.cancelled() => {
                    continue;
                }
                Some(event) = self.event_receiver.recv() => {
                    match event {
                        Event::ActionError(err) => eprintln!("error: {err:?}"),
                        other => self.view.update(other, &self.context),
                    }

                }
                Some(maybe_event) = crossterm_event => {
                    match maybe_event {
                        Ok(event) => match event {
                            crossterm::event::Event::Key(key_event) if key_event.code == KeyCode::Esc => {
                                self.cancellation_token.cancel();
                            }
                            _ => self.view.update(Event::InputEvent(event), &self.context),
                        }
                        Err(err) => {
                            self.view.update(Event::InputError(err), &self.context);
                        }
                    }
                }
            }
        }

        self.task.abort();
        let _ = self.task.await;

        Ok(())
    }
}
