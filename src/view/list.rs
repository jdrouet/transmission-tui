use crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::line::THICK;
use ratatui::text::Text;
use ratatui::widgets::block::Title;
use ratatui::widgets::{Block, LineGauge, Widget};
use transmission_rpc::types::{Torrent, TorrentStatus};

use crate::Action;
use crate::components::confirm::Confirm;
use crate::components::subtitle::{Subtitle, SubtitleItem};
use crate::components::{SIZE_FORMATTER, SPEED_FORMATTER, torrent_status_label};

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
        let text = match self.0.status.unwrap_or(TorrentStatus::Stopped) {
            TorrentStatus::Downloading => {
                let sending_peers = self.0.peers_sending_to_us.unwrap_or(0);
                let connected_peers = self.0.peers_connected.unwrap_or(0);
                let download_spped = self.0.rate_download.unwrap_or(0) as f64;
                let download_speed = SIZE_FORMATTER.format(download_spped).to_string();
                let upload_speed = self.0.rate_upload.unwrap_or(0) as f64;
                let upload_speed = SPEED_FORMATTER.format(upload_speed).to_string();
                Text::from(format!(
                    "{} from {sending_peers} of {connected_peers} peers - 🔻 {download_speed} / 🔺 {upload_speed}",
                    torrent_status_label(TorrentStatus::Downloading)
                ))
                .fg(Color::Gray)
            }
            TorrentStatus::Seeding => {
                let receiving_peers = self.0.peers_getting_from_us.unwrap_or(0);
                let connected_peers = self.0.peers_connected.unwrap_or(0);
                let upload_speed = self.0.rate_upload.unwrap_or(0) as f64;
                let upload_speed = SPEED_FORMATTER.format(upload_speed).to_string();
                Text::from(format!(
                    "{} to {receiving_peers} of {connected_peers} peers - 🔺 {upload_speed}",
                    torrent_status_label(TorrentStatus::Seeding)
                ))
                .fg(Color::LightGreen)
            }
            other => Text::from(torrent_status_label(other)),
        };
        text.render(status, buf);

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

const fn list_view_subtitle() -> Subtitle<4> {
    Subtitle::new([
        SubtitleItem::new("ESC", "Quit"),
        SubtitleItem::new("r", "Reload"),
        SubtitleItem::new("Enter", "Open"),
        SubtitleItem::new("d", "Delete"),
    ])
}

pub(super) struct ListView {
    error: Option<String>,
    loading: bool,
    items: Vec<TorrentItem>,
    offset: usize,
    selected: Option<usize>,
    delete_confirm: Option<i64>,
    //
    subtitle: Subtitle<4>,
}

impl Default for ListView {
    fn default() -> Self {
        Self {
            error: None,
            loading: false,
            items: Vec::default(),
            offset: 0,
            selected: None,
            delete_confirm: None,
            subtitle: list_view_subtitle(),
        }
    }
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

    fn handle_press_enter(&mut self, context: &crate::Context) {
        let index = self.selected.unwrap_or_default();
        if let Some(torrent_id) = self.items.get(index).and_then(|item| item.0.id) {
            context.send_event(crate::Event::OpenTorrent(torrent_id));
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
                KeyCode::Enter => self.handle_press_enter(context),
                KeyCode::Char('r') => {
                    context.send_action(Action::RefreshList);
                }
                KeyCode::Char('d') if self.delete_confirm.is_none() => {
                    self.delete_confirm = self
                        .get_selected()
                        .and_then(|index| self.items.get(index))
                        .and_then(|torrent| torrent.0.id);
                }
                KeyCode::Char('c') => {
                    let _ = self.delete_confirm.take();
                }
                KeyCode::Char('y') => {
                    if let Some(id) = self.delete_confirm.take() {
                        context.send_action(Action::DeleteTorrent(id, true));
                    }
                }
                KeyCode::Char('n') => {
                    if let Some(id) = self.delete_confirm.take() {
                        context.send_action(Action::DeleteTorrent(id, false));
                    }
                }
                _ => {}
            },
            crate::Event::TorrentDeleteStart(_) => {
                self.error = None;
                self.loading = true;
            }
            crate::Event::TorrentDelete(id) => {
                self.error = None;
                let previous = std::mem::replace(&mut self.items, Vec::new());
                self.items = previous
                    .into_iter()
                    .filter(|item| !item.0.id.map(|item_id| item_id == id).unwrap_or(false))
                    .collect();
                self.loading = false;
                self.selected = None;
                context.send_action(Action::RefreshList);
            }
            crate::Event::TorrentDeleteError(_, err) => {
                self.error = Some(err.to_string());
                self.loading = true;
                context.send_action(Action::RefreshList);
            }
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
            .title_bottom(self.subtitle.to_line());
        let inner = block.inner(area);
        block.render(area, buf);
        crate::components::list::List::new(&self.items, self.offset, self.get_selected())
            .render(inner, buf);

        if self.delete_confirm.is_some() {
            Confirm::<3>::new(
                " Delete torrent ",
                "Delete the local data?",
                [
                    SubtitleItem::new("c", "Cancel"),
                    SubtitleItem::new("y", "Yes"),
                    SubtitleItem::new("n", "No"),
                ],
                (40, 5),
            )
            .render(area, buf);
        }
    }
}
