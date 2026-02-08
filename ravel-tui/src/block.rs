//! Block/container builders with borders.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Borders};

use crate::{BuildCx, Builder, RebuildCx, Tui, ViewMarker};

/// A block container builder with optional borders and title.
pub struct BlockBuilder<Body> {
    title: Option<String>,
    borders: Borders,
    border_type: BorderType,
    style: Style,
    body: Body,
}

/// Create a block container builder.
pub fn block<Body>(body: Body) -> BlockBuilder<Body> {
    BlockBuilder {
        title: None,
        borders: Borders::ALL,
        border_type: BorderType::Rounded,
        style: Style::default(),
        body,
    }
}

impl<Body> BlockBuilder<Body> {
    /// Set the title for this block.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set which borders to draw.
    pub fn borders(mut self, borders: Borders) -> Self {
        self.borders = borders;
        self
    }

    /// Set the border type (style).
    pub fn border_type(mut self, border_type: BorderType) -> Self {
        self.border_type = border_type;
        self
    }

    /// Set the style for this block.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    fn make_block(&self) -> Block<'_> {
        let mut block = Block::default()
            .borders(self.borders)
            .border_type(self.border_type)
            .style(self.style);

        if let Some(title) = &self.title {
            block = block.title(title.as_str());
        }

        block
    }

    fn inner_area(&self, area: Rect) -> Rect {
        self.make_block().inner(area)
    }
}

impl<Body: Builder<Tui>> Builder<Tui> for BlockBuilder<Body> {
    type State = BlockState<Body::State>;

    fn build(self, cx: BuildCx<'_>) -> Self::State {
        let block = self.make_block();
        let inner = block.inner(cx.area);

        cx.frame().render_widget(block, cx.area);

        let body_state = self.body.build(cx.with_area(inner));

        BlockState { body: body_state }
    }

    fn rebuild(self, cx: RebuildCx<'_>, state: &mut Self::State) {
        let block = self.make_block();
        let inner = block.inner(cx.area);

        cx.frame().render_widget(block, cx.area);

        self.body.rebuild(cx.with_area(inner), &mut state.body);
    }
}

/// State for a block container.
pub struct BlockState<S> {
    body: S,
}

impl<S: ViewMarker> ViewMarker for BlockState<S> {}

impl<Output, S: ravel::State<Output>> ravel::State<Output> for BlockState<S> {
    fn run(&mut self, output: &mut Output) {
        self.body.run(output);
    }
}

/// A styled wrapper builder that applies a style to its body.
pub struct Styled<Body> {
    style: Style,
    body: Body,
}

/// Create a styled wrapper that applies a style to its body.
pub fn styled<Body>(style: Style, body: Body) -> Styled<Body> {
    Styled { style, body }
}

impl<Body: Builder<Tui>> Builder<Tui> for Styled<Body> {
    type State = Body::State;

    fn build(self, cx: BuildCx<'_>) -> Self::State {
        // For now, styled just renders the body directly
        // In the future, could apply background color, etc.
        self.body.build(cx)
    }

    fn rebuild(self, cx: RebuildCx<'_>, state: &mut Self::State) {
        self.body.rebuild(cx, state);
    }
}
