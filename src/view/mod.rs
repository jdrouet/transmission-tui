use ratatui::Frame;
use ratatui::widgets::Widget;

mod list;
mod torrent;

enum Route {
    List(list::ListView),
    Torrent(torrent::TorrentView),
}

impl Default for Route {
    fn default() -> Self {
        Self::List(list::ListView::default())
    }
}

#[derive(Default)]
pub(crate) struct View {
    inner: Route,
}

impl View {
    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub(crate) fn init(&mut self, ctx: &crate::Context) {
        match &mut self.inner {
            Route::List(inner) => inner.init(ctx),
            Route::Torrent(inner) => inner.init(ctx),
        }
    }

    pub(crate) fn update(&mut self, event: crate::Event, ctx: &crate::Context) {
        match event {
            crate::Event::OpenList => {
                let mut view = list::ListView::default();
                view.init(ctx);
                self.inner = Route::List(view);
            }
            crate::Event::OpenTorrent(torrent_id) => {
                let mut view = torrent::TorrentView::new(torrent_id);
                view.init(ctx);
                self.inner = Route::Torrent(view);
            }
            other => match &mut self.inner {
                Route::List(inner) => inner.update(other, ctx),
                Route::Torrent(inner) => inner.update(other, ctx),
            },
        }
    }
}

impl Widget for &View {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        match &self.inner {
            Route::List(inner) => inner.render(area, buf),
            Route::Torrent(inner) => inner.render(area, buf),
        }
    }
}
