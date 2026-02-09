//! Stateful list builder.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::{BuildCx, Builder, Draw, RebuildCx, Tui, ViewMarker};

/// A stateful list builder that maintains selection state.
pub struct ListBuilder<'a, T, F> {
    items: &'a [T],
    render_item: F,
    block: Option<ListBlock>,
    highlight_style: Style,
}

struct ListBlock {
    title: Option<String>,
    borders: Borders,
    border_type: BorderType,
    style: Style,
}

/// Create a list builder.
///
/// The `render_item` function receives each item, its index, and whether it's selected,
/// and should return a `Text` for rendering.
pub fn list<'a, T, F>(items: &'a [T], render_item: F) -> ListBuilder<'a, T, F>
where
    F: Fn(&T, usize, bool) -> Text<'a>,
{
    ListBuilder {
        items,
        render_item,
        block: None,
        highlight_style: Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    }
}

impl<'a, T, F> ListBuilder<'a, T, F>
where
    F: Fn(&T, usize, bool) -> Text<'a>,
{
    /// Add a block around the list.
    pub fn block(mut self) -> Self {
        self.block = Some(ListBlock {
            title: None,
            borders: Borders::ALL,
            border_type: BorderType::Rounded,
            style: Style::default(),
        });
        self
    }

    /// Set the block title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        if let Some(block) = &mut self.block {
            block.title = Some(title.into());
        } else {
            self.block = Some(ListBlock {
                title: Some(title.into()),
                borders: Borders::ALL,
                border_type: BorderType::Rounded,
                style: Style::default(),
            });
        }
        self
    }

    /// Set the block style.
    pub fn block_style(mut self, style: Style) -> Self {
        if let Some(block) = &mut self.block {
            block.style = style;
        }
        self
    }

    /// Set the highlight style for selected items.
    pub fn highlight_style(mut self, style: Style) -> Self {
        self.highlight_style = style;
        self
    }

    /// Render items with current selection into owned Text.
    fn render_items_with_selection(&self, selected: Option<usize>) -> Vec<StoredText> {
        self.items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let text = (self.render_item)(item, i, selected == Some(i));
                StoredText::from_text(text)
            })
            .collect()
    }
}

impl<'a, T, F> Builder<Tui> for ListBuilder<'a, T, F>
where
    F: Fn(&T, usize, bool) -> Text<'a>,
{
    type State = ListBuilderState;

    fn build(self, _cx: BuildCx) -> Self::State {
        let mut list_state = ListState::default();
        if !self.items.is_empty() {
            list_state.select(Some(0));
        }

        let items = self.render_items_with_selection(list_state.selected());

        ListBuilderState {
            list_state,
            items,
            block: self.block.map(|b| ListBlockState {
                title: b.title,
                borders: b.borders,
                border_type: b.border_type,
                style: b.style,
            }),
            highlight_style: self.highlight_style,
        }
    }

    fn rebuild(self, _cx: RebuildCx, state: &mut Self::State) {
        // Clamp selection to valid range
        if let Some(selected) = state.list_state.selected() {
            if selected >= self.items.len() {
                state
                    .list_state
                    .select(self.items.len().checked_sub(1));
            }
        } else if !self.items.is_empty() {
            state.list_state.select(Some(0));
        }

        // Re-render items with current selection
        state.items = self.render_items_with_selection(state.list_state.selected());

        // Update block config
        state.block = self.block.map(|b| ListBlockState {
            title: b.title,
            borders: b.borders,
            border_type: b.border_type,
            style: b.style,
        });
        state.highlight_style = self.highlight_style;
    }
}

/// Stored block configuration for drawing
struct ListBlockState {
    title: Option<String>,
    borders: Borders,
    border_type: BorderType,
    style: Style,
}

/// An owned representation of Text that can be stored across frames.
#[derive(Clone)]
struct StoredText {
    lines: Vec<StoredLine>,
}

/// An owned representation of a Line.
#[derive(Clone)]
struct StoredLine {
    spans: Vec<StoredSpan>,
}

/// An owned representation of a Span.
#[derive(Clone)]
struct StoredSpan {
    content: String,
    style: Style,
}

impl StoredText {
    fn from_text(text: Text<'_>) -> Self {
        let lines = text
            .lines
            .into_iter()
            .map(|line| StoredLine {
                spans: line
                    .spans
                    .into_iter()
                    .map(|span| StoredSpan {
                        content: span.content.into_owned(),
                        style: span.style,
                    })
                    .collect(),
            })
            .collect();
        Self { lines }
    }

    fn to_list_item(&self) -> ListItem<'_> {
        let lines: Vec<Line<'_>> = self
            .lines
            .iter()
            .map(|line| {
                Line::from(
                    line.spans
                        .iter()
                        .map(|span| Span::styled(span.content.as_str(), span.style))
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        ListItem::new(lines)
    }
}

/// State for a list builder.
pub struct ListBuilderState {
    pub list_state: ListState,
    items: Vec<StoredText>,
    block: Option<ListBlockState>,
    highlight_style: Style,
}

impl Default for ListBuilderState {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            list_state,
            items: Vec::new(),
            block: None,
            highlight_style: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }
}

impl ViewMarker for ListBuilderState {}

impl Draw for ListBuilderState {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem<'_>> = self.items.iter().map(|i| i.to_list_item()).collect();

        let mut list = List::new(items).highlight_style(self.highlight_style);

        if let Some(block) = &self.block {
            let mut b = Block::default()
                .borders(block.borders)
                .border_type(block.border_type)
                .style(block.style);

            if let Some(title) = &block.title {
                b = b.title(title.as_str());
            }

            list = list.block(b);
        }

        // We need to clone the list_state to use it mutably
        let mut list_state = self.list_state.clone();
        frame.render_stateful_widget(list, area, &mut list_state);
    }
}

impl ListBuilderState {
    /// Get the currently selected index.
    pub fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    /// Select a specific index.
    pub fn select(&mut self, index: Option<usize>) {
        self.list_state.select(index);
    }

    /// Move selection up.
    pub fn select_previous(&mut self) {
        self.list_state.select_previous();
    }

    /// Move selection down.
    pub fn select_next(&mut self) {
        self.list_state.select_next();
    }

    /// Get mutable access to the underlying ListState.
    pub fn inner_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }
}

/// Trait for outputs that can receive list selection.
pub trait ListOutput {
    fn set_selection(&mut self, index: Option<usize>);
}

impl<Output> ravel::State<Output> for ListBuilderState {
    fn run(&mut self, _output: &mut Output) {
        // List state doesn't emit events on its own
        // Selection is managed externally via the state methods
    }
}

/// Helper function to create a simple list item text.
pub fn simple_item(text: impl Into<String>, is_selected: bool) -> Text<'static> {
    let style = if is_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let prefix = if is_selected { "→ " } else { "  " };
    let text_string = text.into();

    Text::from(Line::from(vec![
        Span::styled(prefix.to_string(), style),
        Span::styled(text_string, style),
    ]))
}
