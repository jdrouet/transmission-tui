use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Buffer, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Widget};

pub trait ListItem: Widget {
    fn height(&self) -> u16;
}

struct ItemWrapper<I> {
    item: I,
    selected: bool,
}

impl<I> ItemWrapper<I> {
    fn new(item: I, selected: bool) -> Self {
        Self { item, selected }
    }
}

impl<I> Widget for &ItemWrapper<I>
where
    I: Copy + Widget,
{
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [border, _, content] = Layout::new(
            Direction::Horizontal,
            vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ],
        )
        .areas(area);
        if self.selected {
            Block::new()
                .style(Style::new().on_dark_gray())
                .render(border, buf);
        }
        self.item.render(content, buf);
    }
}

#[derive(Default)]
pub struct List<'a, I> {
    items: &'a [I],
    offset: usize,
    selected: Option<usize>,
}

impl<'a, I> List<'a, I> {
    pub fn new(items: &'a [I], offset: usize, selected: Option<usize>) -> Self {
        Self {
            items,
            offset,
            selected,
        }
    }

    pub fn set_items(&mut self, items: &'a [I]) {
        self.items = items;
    }

    pub fn with_items(mut self, items: &'a [I]) -> Self {
        self.set_items(items);
        self
    }

    pub fn set_offset(&mut self, offset: usize) {
        self.offset = offset;
    }

    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    pub fn set_selected(&mut self, selected: Option<usize>) {
        self.selected = selected;
    }

    pub fn with_selected(mut self, selected: Option<usize>) -> Self {
        self.set_selected(selected);
        self
    }
}

impl<'a, I> Widget for &'a List<'a, I>
where
    &'a I: ListItem + 'a,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let available_height = area.height;
        let mut constraints = Vec::default();
        let mut total = 0u16;
        for item in self.items.iter().skip(self.offset) {
            let h = item.height();
            if total + h <= available_height {
                constraints.push(Constraint::Length(item.height()));
                total += h;
            } else {
                break;
            }
        }
        let layouts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);
        self.items
            .iter()
            .enumerate()
            .skip(self.offset)
            .zip(&*layouts)
            .for_each(|((index, item), area)| {
                let selected = self.selected.map(|i| i == index).unwrap_or(false);
                ItemWrapper::new(item, selected).render(*area, buf);
            });
    }
}
