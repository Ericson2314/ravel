//! Modal/popup overlay builders.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Borders, Clear};
use ratatui::Frame;

use crate::{BuildCx, Builder, Draw, RebuildCx, Tui, ViewMarker};

/// A modal popup builder.
pub struct Modal<Body> {
    title: Option<String>,
    style: Style,
    percent_x: u16,
    percent_y: u16,
    fixed_width: Option<u16>,
    fixed_height: Option<u16>,
    body: Body,
}

/// Create a modal popup builder.
pub fn modal<Body>(body: Body) -> Modal<Body> {
    Modal {
        title: None,
        style: Style::default(),
        percent_x: 60,
        percent_y: 25,
        fixed_width: None,
        fixed_height: None,
        body,
    }
}

impl<Body> Modal<Body> {
    /// Set the title for this modal.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the style for this modal's border.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the percentage of screen width/height for this modal.
    pub fn percent(mut self, x: u16, y: u16) -> Self {
        self.percent_x = x;
        self.percent_y = y;
        self
    }

    /// Set a fixed width (ignores percent_x).
    pub fn width(mut self, width: u16) -> Self {
        self.fixed_width = Some(width);
        self
    }

    /// Set a fixed height (ignores percent_y).
    pub fn height(mut self, height: u16) -> Self {
        self.fixed_height = Some(height);
        self
    }

    fn modal_area(&self, container: Rect) -> Rect {
        match (self.fixed_width, self.fixed_height) {
            (Some(w), Some(h)) => centered_rect_fixed(w, h, container),
            (Some(w), None) => {
                let area = centered_rect(100, self.percent_y, container);
                centered_rect_fixed(w, area.height, container)
            }
            (None, Some(h)) => {
                let area = centered_rect(self.percent_x, 100, container);
                centered_rect_fixed(area.width, h, container)
            }
            (None, None) => centered_rect(self.percent_x, self.percent_y, container),
        }
    }
}

/// State for a modal.
pub struct ModalState<S> {
    body: S,
    /// Modal configuration for drawing
    title: Option<String>,
    style: Style,
    /// The outer area (for clearing and drawing the block)
    outer_area: Rect,
    /// The inner area (for drawing the body)
    inner_area: Rect,
}

impl<S: ViewMarker> ViewMarker for ModalState<S> {}

impl<S: Draw> Draw for ModalState<S> {
    fn draw(&self, frame: &mut Frame, _area: Rect) {
        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(self.style);

        if let Some(title) = &self.title {
            block = block.title(title.as_str());
        }

        // Clear the background
        frame.render_widget(Clear, self.outer_area);
        frame.render_widget(block, self.outer_area);

        self.body.draw(frame, self.inner_area);
    }
}

impl<Output, S: ravel::State<Output>> ravel::State<Output> for ModalState<S> {
    fn run(&mut self, output: &mut Output) {
        self.body.run(output);
    }
}

impl<Body: Builder<Tui>> Builder<Tui> for Modal<Body>
where
    Body::State: Draw,
{
    type State = ModalState<Body::State>;

    fn build(self, cx: BuildCx) -> Self::State {
        let area = self.modal_area(cx.area);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(self.style);
        let inner = block.inner(area);

        let body_state = self.body.build(cx.with_area(inner));

        ModalState {
            body: body_state,
            title: self.title,
            style: self.style,
            outer_area: area,
            inner_area: inner,
        }
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        let area = self.modal_area(cx.area);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(self.style);
        let inner = block.inner(area);

        state.title = self.title;
        state.style = self.style;
        state.outer_area = area;
        state.inner_area = inner;

        self.body.rebuild(cx.with_area(inner), &mut state.body);
    }
}

/// Create a centered rectangle with percentage-based dimensions.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_width = r.width * percent_x / 100;
    let popup_height = r.height * percent_y / 100;

    let x = r.x + (r.width.saturating_sub(popup_width)) / 2;
    let y = r.y + (r.height.saturating_sub(popup_height)) / 2;

    Rect {
        x,
        y,
        width: popup_width,
        height: popup_height,
    }
}

/// Create a centered rectangle with fixed dimensions.
pub fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;

    Rect {
        x,
        y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}

/// A layer that renders multiple overlapping components.
///
/// Later components are rendered on top of earlier ones.
pub struct Layer<Base, Overlay> {
    base: Base,
    overlay: Overlay,
}

/// Create a layer with a base and overlay.
pub fn layer<Base, Overlay>(base: Base, overlay: Overlay) -> Layer<Base, Overlay> {
    Layer { base, overlay }
}

/// State for a layer.
pub struct LayerState<B, O> {
    base: B,
    overlay: O,
    area: Rect,
}

impl<B: ViewMarker, O: ViewMarker> ViewMarker for LayerState<B, O> {}

impl<B: Draw, O: Draw> Draw for LayerState<B, O> {
    fn draw(&self, frame: &mut Frame, _area: Rect) {
        self.base.draw(frame, self.area);
        self.overlay.draw(frame, self.area);
    }
}

impl<Output, B: ravel::State<Output>, O: ravel::State<Output>> ravel::State<Output>
    for LayerState<B, O>
{
    fn run(&mut self, output: &mut Output) {
        self.base.run(output);
        self.overlay.run(output);
    }
}

impl<Base: Builder<Tui>, Overlay: Builder<Tui>> Builder<Tui> for Layer<Base, Overlay>
where
    Base::State: Draw,
    Overlay::State: Draw,
{
    type State = LayerState<Base::State, Overlay::State>;

    fn build(self, cx: BuildCx) -> Self::State {
        let base_state = self.base.build(cx);
        let overlay_state = self.overlay.build(cx);
        LayerState {
            base: base_state,
            overlay: overlay_state,
            area: cx.area,
        }
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        state.area = cx.area;
        self.base.rebuild(cx, &mut state.base);
        self.overlay.rebuild(cx, &mut state.overlay);
    }
}
