use ratatui::prelude::{Buffer, Rect};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Widget};

pub(super) struct LoadingView<'a> {
    message: &'a str,
}

impl<'a> LoadingView<'a> {
    pub(super) fn new(message: &'a str) -> Self {
        Self { message }
    }

    pub(super) fn init(&mut self, context: &crate::Context) {}

    pub(super) fn update(&mut self, event: crate::Event, context: &crate::Context) {}
}

impl<'a> Widget for &LoadingView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let block = Block::new();

        let counter_text =
            Text::from(vec![Line::from("LOADING..."), Line::from(self.message)]).centered();

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}
