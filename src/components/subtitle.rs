use ratatui::style::Stylize;
use ratatui::text::{Line, Span};

#[derive(Clone, Copy, Debug)]
pub(crate) struct SubtitleItem {
    label: &'static str,
    trigger: &'static str,
}

impl SubtitleItem {
    pub const fn new(trigger: &'static str, label: &'static str) -> Self {
        Self { label, trigger }
    }

    fn to_spans(&self) -> impl Iterator<Item = Span<'static>> {
        [
            " ".into(),
            self.label.into(),
            format!(" <{}> ", self.trigger).bold(),
        ]
        .into_iter()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Subtitle<const N: usize> {
    items: [SubtitleItem; N],
}

impl<const N: usize> Subtitle<N> {
    pub const fn new(items: [SubtitleItem; N]) -> Self {
        Self { items }
    }

    pub fn to_line(&self) -> Line {
        Line::from_iter(self.items.iter().enumerate().flat_map(|(index, item)| {
            if index == 0 {
                std::iter::once(Span::raw("")).chain(item.to_spans())
            } else {
                std::iter::once(Span::raw("-")).chain(item.to_spans())
            }
        }))
    }
}
