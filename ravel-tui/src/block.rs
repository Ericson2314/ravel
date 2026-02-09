//! Block/container builders with borders.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::Frame;

use crate::{BuildCx, Builder, Draw, RebuildCx, Tui, ViewMarker};

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
}

/// State for a block container.
pub struct BlockState<S> {
    body: S,
    /// Block configuration for drawing
    title: Option<String>,
    borders: Borders,
    border_type: BorderType,
    style: Style,
    /// The outer area (for drawing the block)
    outer_area: Rect,
    /// The inner area (for drawing the body)
    inner_area: Rect,
}

impl<S: ViewMarker> ViewMarker for BlockState<S> {}

impl<S: Draw> Draw for BlockState<S> {
    fn draw(&self, frame: &mut Frame, _area: Rect) {
        let mut block = Block::default()
            .borders(self.borders)
            .border_type(self.border_type)
            .style(self.style);

        if let Some(title) = &self.title {
            block = block.title(title.as_str());
        }

        frame.render_widget(block, self.outer_area);
        self.body.draw(frame, self.inner_area);
    }
}

impl<Output, S: ravel::State<Output>> ravel::State<Output> for BlockState<S> {
    fn run(&mut self, output: &mut Output) {
        self.body.run(output);
    }
}

impl<Body: Builder<Tui>> Builder<Tui> for BlockBuilder<Body>
where
    Body::State: Draw,
{
    type State = BlockState<Body::State>;

    fn build(self, cx: BuildCx) -> Self::State {
        let block = Block::default()
            .borders(self.borders)
            .border_type(self.border_type)
            .style(self.style);
        let inner = block.inner(cx.area);

        let body_state = self.body.build(cx.with_area(inner));

        BlockState {
            body: body_state,
            title: self.title,
            borders: self.borders,
            border_type: self.border_type,
            style: self.style,
            outer_area: cx.area,
            inner_area: inner,
        }
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        let block = Block::default()
            .borders(self.borders)
            .border_type(self.border_type)
            .style(self.style);
        let inner = block.inner(cx.area);

        state.title = self.title;
        state.borders = self.borders;
        state.border_type = self.border_type;
        state.style = self.style;
        state.outer_area = cx.area;
        state.inner_area = inner;

        self.body.rebuild(cx.with_area(inner), &mut state.body);
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

    fn build(self, cx: BuildCx) -> Self::State {
        // For now, styled just renders the body directly
        // In the future, could apply background color, etc.
        self.body.build(cx)
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        self.body.rebuild(cx, state);
    }
}
