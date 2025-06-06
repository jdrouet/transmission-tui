use crossterm::event::{Event, KeyCode};
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::block::Title;
use ratatui::widgets::{Block, Padding, Paragraph, Widget};
use transmission_rpc::types::Torrent;

use crate::Action;
use crate::components::subtitle::{Subtitle, SubtitleItem};
use crate::components::{SIZE_FORMATTER, torrent_status_label};

pub struct TorrentView {
    id: i64,
    error: Option<String>,
    loading: bool,
    item: Option<Torrent>,
    //
    subtitle: Subtitle<3>,
}

const fn torrent_view_subtitle() -> Subtitle<3> {
    Subtitle::new([
        SubtitleItem::new("ESC", "Quit"),
        SubtitleItem::new("Backspace", "Back"),
        SubtitleItem::new("r", "Reload"),
    ])
}

impl TorrentView {
    pub(super) fn new(torrent_id: i64) -> Self {
        Self {
            id: torrent_id,
            error: None,
            loading: false,
            item: None,
            //
            subtitle: torrent_view_subtitle(),
        }
    }

    pub(super) fn init(&mut self, context: &crate::Context) {
        context.send_action(Action::RefreshTorrent(self.id));
    }

    pub(super) fn update(&mut self, event: crate::Event, context: &crate::Context) {
        match event {
            crate::Event::InputEvent(Event::Key(inner)) => match inner.code {
                KeyCode::Char('r') => context.send_action(Action::RefreshTorrent(self.id)),
                KeyCode::Backspace => context.send_event(crate::Event::OpenList),
                _ => {}
            },
            crate::Event::TorrentUpdateStart => {
                self.error = None;
                self.loading = true;
            }
            crate::Event::TorrentUpdate(item) => {
                self.error = None;
                self.item = Some(item);
                self.loading = false;
            }
            crate::Event::TorrentUpdateError(err) => {
                self.error = Some(err.to_string());
                self.loading = false;
            }
            _ => {}
        }
    }

    fn render_inner(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(vec![
            Line::from(vec![
                "Name: ".into(),
                self.item
                    .as_ref()
                    .and_then(|item| item.name.as_deref())
                    .unwrap_or("")
                    .bold(),
            ]),
            Line::from(vec![
                "Size: ".into(),
                self.item
                    .as_ref()
                    .and_then(|item| item.total_size)
                    .map(|value| SIZE_FORMATTER.format(value as f64).to_string())
                    .unwrap_or_default()
                    .bold(),
            ]),
            Line::from(vec![
                "Status: ".into(),
                self.item
                    .as_ref()
                    .and_then(|item| item.status)
                    .map(torrent_status_label)
                    .unwrap_or_default()
                    .bold(),
            ]),
        ])
        .render(area, buf);
    }
}

impl Widget for &TorrentView {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = if self.loading {
            Title::from(" Transmission > Torrent (Loading...) ")
        } else if let Some(err) = self.error.as_ref() {
            Title::from(format!(" Transmission > Torrent ({err:?}) ").red())
        } else {
            Title::from(" Transmission > Torrent ")
        };
        let block = Block::bordered()
            .title(title)
            .title_bottom(self.subtitle.to_line())
            .padding(Padding::horizontal(2));
        let inner = block.inner(area);
        block.render(area, buf);

        self.render_inner(inner, buf);
    }
}
