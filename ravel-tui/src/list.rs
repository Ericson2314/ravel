//! Stateful list builder.

use std::marker::PhantomData;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState};

use crate::{BuildCx, Builder, RebuildCx, Tui, ViewMarker};

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
/// The `render_item` function receives each item and its index, and should return
/// a `ListItem` for rendering.
pub fn list<'a, T, F>(items: &'a [T], render_item: F) -> ListBuilder<'a, T, F>
where
    F: Fn(&T, usize, bool) -> ListItem<'a>,
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
    F: Fn(&T, usize, bool) -> ListItem<'a>,
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

    fn make_block(&self) -> Option<Block<'_>> {
        self.block.as_ref().map(|b| {
            let mut block = Block::default()
                .borders(b.borders)
                .border_type(b.border_type)
                .style(b.style);

            if let Some(title) = &b.title {
                block = block.title(title.as_str());
            }

            block
        })
    }

    fn render(&self, cx: &BuildCx<'_>, state: &mut ListState) {
        let selected = state.selected();

        let items: Vec<ListItem<'a>> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = selected == Some(i);
                (self.render_item)(item, i, is_selected)
            })
            .collect();

        let mut list = List::new(items).highlight_style(self.highlight_style);

        if let Some(block) = self.make_block() {
            list = list.block(block);
        }

        cx.frame().render_stateful_widget(list, cx.area, state);
    }
}

impl<'a, T, F> Builder<Tui> for ListBuilder<'a, T, F>
where
    F: Fn(&T, usize, bool) -> ListItem<'a>,
{
    type State = ListBuilderState;

    fn build(self, cx: BuildCx<'_>) -> Self::State {
        let mut list_state = ListState::default();
        if !self.items.is_empty() {
            list_state.select(Some(0));
        }

        self.render(&cx, &mut list_state);

        ListBuilderState { list_state }
    }

    fn rebuild(self, cx: RebuildCx<'_>, state: &mut Self::State) {
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

        self.render(
            &BuildCx {
                frame_wrapper: cx.frame_wrapper,
                area: cx.area,
                _marker: PhantomData,
            },
            &mut state.list_state,
        );
    }
}

/// State for a list builder.
pub struct ListBuilderState {
    pub list_state: ListState,
}

impl Default for ListBuilderState {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self { list_state }
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

impl ViewMarker for ListBuilderState {}

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

/// Helper function to create a simple list item from text.
pub fn simple_item<'a>(text: impl Into<String>, is_selected: bool) -> ListItem<'a> {
    let style = if is_selected {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let prefix = if is_selected { "→ " } else { "  " };

    ListItem::new(Line::from(vec![
        Span::styled(prefix, style),
        Span::styled(text.into(), style),
    ]))
}
