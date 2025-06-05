use std::sync::LazyLock;

use crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::line::THICK;
use ratatui::text::{Line, Text};
use ratatui::widgets::block::Title;
use ratatui::widgets::{Block, LineGauge, Widget};
use transmission_rpc::types::{Torrent, TorrentStatus};

use crate::Action;

const SIZE_FORMATTER: LazyLock<human_number::Formatter<'static>> =
    LazyLock::new(|| human_number::Formatter::si().with_unit("B"));
const SPEED_FORMATTER: LazyLock<human_number::Formatter<'static>> =
    LazyLock::new(|| human_number::Formatter::si().with_unit("B/s"));

struct TorrentItem(Torrent);

impl Widget for &TorrentItem {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [name_area, status, progress, info] = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(area);

        // filename
        if let Some(name) = self.0.name.as_deref() {
            Text::from(name).bold().render(name_area, buf);
        }

        // status
        match self.0.status.unwrap_or(TorrentStatus::Stopped) {
            TorrentStatus::Downloading => {
                let sending_peers = self.0.peers_sending_to_us.unwrap_or(0);
                let connected_peers = self.0.peers_connected.unwrap_or(0);
                let download_spped = self.0.rate_download.unwrap_or(0) as f64;
                let download_speed = SIZE_FORMATTER.format(download_spped).to_string();
                let upload_speed = self.0.rate_upload.unwrap_or(0) as f64;
                let upload_speed = SPEED_FORMATTER.format(upload_speed).to_string();
                Text::from(format!(
                    "Downloading from {sending_peers} of {connected_peers} peers - 🔻 {download_speed} / 🔺 {upload_speed}"
                ))
                .fg(Color::Gray)
                .render(status, buf);
            }
            TorrentStatus::Verifying => Text::from("Verifying...").render(status, buf),
            TorrentStatus::Seeding => {
                let receiving_peers = self.0.peers_getting_from_us.unwrap_or(0);
                let connected_peers = self.0.peers_connected.unwrap_or(0);
                let upload_speed = self.0.rate_upload.unwrap_or(0) as f64;
                let upload_speed = SPEED_FORMATTER.format(upload_speed).to_string();
                Text::from(format!(
                    "Seeding to {receiving_peers} of {connected_peers} peers - 🔺 {upload_speed}",
                ))
                .fg(Color::LightGreen)
                .render(status, buf);
            }
            _ => {}
        }

        // progress
        LineGauge::default()
            .ratio(self.0.percent_done.unwrap_or(0.0) as f64)
            .line_set(THICK)
            .filled_style(Style::new().green().on_black().bold())
            .render(progress, buf);

        // info
        let downloaded_size = self.0.downloaded_ever.unwrap_or(0) as f64;
        let downloaded_size = SIZE_FORMATTER.format(downloaded_size).to_string();
        let total_size = self.0.total_size.unwrap_or(0) as f64;
        let total_size = SIZE_FORMATTER.format(total_size).to_string();
        Text::from(format!("{downloaded_size} of {total_size}"))
            .fg(Color::Gray)
            .render(info, buf);
    }
}

impl crate::components::list::ListItem for &TorrentItem {
    fn height(&self) -> u16 {
        4
    }
}

#[derive(Default)]
pub(super) struct ListView {
    error: Option<String>,
    loading: bool,
    items: Vec<TorrentItem>,
    offset: usize,
    selected: Option<usize>,
}

impl ListView {
    fn handle_press_up(&mut self) {
        if self.items.is_empty() {
            self.selected = None;
        } else if let Some(current) = self.selected.as_mut() {
            if *current > 0 {
                *current -= 1;
            }
        } else {
            self.selected = Some(0);
        }
    }

    fn handle_press_down(&mut self) {
        if self.items.is_empty() {
            self.selected = None;
        } else if let Some(current) = self.selected.as_mut() {
            *current = (self.items.len() - 1).min(*current + 1);
        } else {
            self.selected = Some(1);
        }
    }

    fn get_selected(&self) -> Option<usize> {
        Some(self.selected.unwrap_or(0))
    }

    pub(super) fn init(&mut self, context: &crate::Context) {
        context.send_action(Action::RefreshList);
    }

    pub(super) fn update(&mut self, event: crate::Event, context: &crate::Context) {
        match event {
            crate::Event::InputEvent(Event::Key(inner)) => match inner.code {
                KeyCode::Up => self.handle_press_up(),
                KeyCode::Down => self.handle_press_down(),
                KeyCode::Char('r') => {
                    context.send_action(Action::RefreshList);
                }
                _ => {}
            },
            crate::Event::TorrentListUpdateStart => {
                self.error = None;
                self.loading = true;
            }
            crate::Event::TorrentListUpdate(list) => {
                let same_size = self.items.len() == list.torrents.len();
                self.items = list
                    .torrents
                    .into_iter()
                    .map(TorrentItem)
                    .collect::<Vec<_>>();
                self.loading = false;
                if !same_size {
                    self.selected = None;
                }
            }
            crate::Event::TorrentListUpdateError(err) => {
                self.loading = false;
                self.error = Some(err.to_string());
            }
            _ => {}
        }
    }
}

impl Widget for &ListView {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered()
            .title(if self.loading {
                Title::from(" Transmission (Loading...) ")
            } else if let Some(err) = self.error.as_ref() {
                Title::from(format!(" Transmission ({err:?}) ").red())
            } else {
                Title::from(" Transmission ")
            })
            .title_bottom(Line::from(vec![
                " Quit ".into(),
                "<ESC>".bold(),
                " - Reload ".into(),
                "<r> ".bold(),
            ]));
        let inner = block.inner(area);
        block.render(area, buf);
        crate::components::list::List::new(&self.items, self.offset, self.get_selected())
            .render(inner, buf);
    }
}
