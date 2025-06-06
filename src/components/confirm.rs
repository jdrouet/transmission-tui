use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::Stylize;
use ratatui::widgets::block::Title;
use ratatui::widgets::{Block, Clear, Padding, Paragraph, Widget};

use crate::components::subtitle::{Subtitle, SubtitleItem};

pub(crate) struct Confirm<const N: usize> {
    title: &'static str,
    message: &'static str,
    subtitle: Subtitle<N>,
    size: (u16, u16),
}

impl<const N: usize> Confirm<N> {
    pub const fn new(
        title: &'static str,
        message: &'static str,
        options: [SubtitleItem; N],
        size: (u16, u16),
    ) -> Self {
        Self {
            title,
            message,
            subtitle: Subtitle::new(options),
            size,
        }
    }
}

impl<const N: usize> Widget for &Confirm<N> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let vertical = Layout::vertical([Constraint::Max(self.size.1)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Max(self.size.0)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);

        let block = Block::bordered()
            .title(Title::from(self.title.bold()))
            .title_bottom(self.subtitle.to_line())
            .padding(Padding::symmetric(2, 1));

        Clear.render(area, buf);
        Paragraph::new(self.message).block(block).render(area, buf);
    }
}
