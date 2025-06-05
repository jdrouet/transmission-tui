use ratatui::Frame;
use ratatui::widgets::Widget;

mod list;

enum Route {
    List(list::ListView),
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
        }
    }

    pub(crate) fn update(&mut self, event: crate::Event, ctx: &crate::Context) {
        match &mut self.inner {
            Route::List(inner) => inner.update(event, ctx),
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
        }
    }
}
