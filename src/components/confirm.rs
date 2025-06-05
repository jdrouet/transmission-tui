use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::block::Title;
use ratatui::widgets::{Block, Clear, Padding, Paragraph, Widget};

pub(crate) struct ConfirmOption {
    letter: char,
    label: &'static str,
}

impl ConfirmOption {
    pub fn new(letter: char, label: &'static str) -> Self {
        Self { label, letter }
    }
}

pub(crate) struct Confirm<const N: usize> {
    title: Option<&'static str>,
    message: &'static str,
    options: [ConfirmOption; N],
    size: (u16, u16),
}

impl<const N: usize> Confirm<N> {
    pub const fn new(message: &'static str, options: [ConfirmOption; N], size: (u16, u16)) -> Self {
        Self {
            title: None,
            message,
            options,
            size,
        }
    }

    pub fn with_title(mut self, title: &'static str) -> Self {
        self.title = Some(title);
        self
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

        let block = Block::bordered();
        let block = match self.title {
            Some(v) => block.title(Title::from(v.bold())),
            None => block,
        };
        let mut options = vec![" Cancel ".into(), "<c> ".bold()];
        for opt in self.options.iter() {
            options.push(format!("- {} ", opt.label).into());
            options.push(format!("<{}> ", opt.letter).bold());
        }
        let block = block
            .title_bottom(Line::from(options))
            .padding(Padding::symmetric(2, 1));

        Clear.render(area, buf);
        Paragraph::new(self.message).block(block).render(area, buf);
    }
}
