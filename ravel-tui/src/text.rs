//! Text rendering builders.

use std::borrow::Cow;

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::{BuildCx, Builder, Draw, RebuildCx, Tui, ViewMarker};

/// A text builder for rendering styled text.
pub struct Text<'a> {
    content: Cow<'a, str>,
    style: Style,
    wrap: bool,
}

/// Create a text builder from a string.
pub fn text<'a>(content: impl Into<Cow<'a, str>>) -> Text<'a> {
    Text {
        content: content.into(),
        style: Style::default(),
        wrap: false,
    }
}

impl<'a> Text<'a> {
    /// Set the style for this text.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Enable word wrapping.
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }
}

/// State for a text component.
pub struct TextState {
    content: String,
    style: Style,
    wrap: bool,
}

impl ViewMarker for TextState {}

impl Draw for TextState {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        let mut paragraph = Paragraph::new(self.content.as_str()).style(self.style);
        if self.wrap {
            paragraph = paragraph.wrap(Wrap { trim: true });
        }
        frame.render_widget(paragraph, area);
    }
}

impl<Output> ravel::State<Output> for TextState {
    fn run(&mut self, _output: &mut Output) {}
}

impl Builder<Tui> for Text<'_> {
    type State = TextState;

    fn build(self, _cx: BuildCx) -> Self::State {
        TextState {
            content: self.content.into_owned(),
            style: self.style,
            wrap: self.wrap,
        }
    }

    fn rebuild(self, _cx: RebuildCx, state: &mut Self::State) {
        state.content = self.content.into_owned();
        state.style = self.style;
        state.wrap = self.wrap;
    }
}

/// A builder for rendering a line of styled spans.
pub struct StyledLine<'a> {
    spans: Vec<Span<'a>>,
}

/// Create a styled line builder.
pub fn styled_line<'a>(spans: impl Into<Vec<Span<'a>>>) -> StyledLine<'a> {
    StyledLine {
        spans: spans.into(),
    }
}

/// State for a styled line component.
pub struct StyledLineState {
    spans: Vec<(String, Style)>,
}

impl ViewMarker for StyledLineState {}

impl Draw for StyledLineState {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        let spans: Vec<Span<'_>> = self
            .spans
            .iter()
            .map(|(content, style)| Span::styled(content.as_str(), *style))
            .collect();
        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, area);
    }
}

impl<Output> ravel::State<Output> for StyledLineState {
    fn run(&mut self, _output: &mut Output) {}
}

impl Builder<Tui> for StyledLine<'_> {
    type State = StyledLineState;

    fn build(self, _cx: BuildCx) -> Self::State {
        StyledLineState {
            spans: self
                .spans
                .into_iter()
                .map(|s| (s.content.into_owned(), s.style))
                .collect(),
        }
    }

    fn rebuild(self, _cx: RebuildCx, state: &mut Self::State) {
        state.spans = self
            .spans
            .into_iter()
            .map(|s| (s.content.into_owned(), s.style))
            .collect();
    }
}

/// State for a static string.
pub struct StaticStrState {
    content: &'static str,
}

impl ViewMarker for StaticStrState {}

impl Draw for StaticStrState {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(self.content);
        frame.render_widget(paragraph, area);
    }
}

impl<Output> ravel::State<Output> for StaticStrState {
    fn run(&mut self, _output: &mut Output) {}
}

/// Builder for &'static str - allows using string literals directly as views.
impl Builder<Tui> for &'static str {
    type State = StaticStrState;

    fn build(self, _cx: BuildCx) -> Self::State {
        StaticStrState { content: self }
    }

    fn rebuild(self, _cx: RebuildCx, state: &mut Self::State) {
        state.content = self;
    }
}

/// State for an owned string.
pub struct StringState {
    content: String,
}

impl ViewMarker for StringState {}

impl Draw for StringState {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(self.content.as_str());
        frame.render_widget(paragraph, area);
    }
}

impl<Output> ravel::State<Output> for StringState {
    fn run(&mut self, _output: &mut Output) {}
}

/// Builder for String - allows using owned strings as views.
impl Builder<Tui> for String {
    type State = StringState;

    fn build(self, _cx: BuildCx) -> Self::State {
        StringState { content: self }
    }

    fn rebuild(self, _cx: RebuildCx, state: &mut Self::State) {
        state.content = self;
    }
}

/// Builder for formatted display values.
pub struct Display<T> {
    value: T,
    style: Style,
}

/// Create a display builder that formats a value.
pub fn display<T: std::fmt::Display>(value: T) -> Display<T> {
    Display {
        value,
        style: Style::default(),
    }
}

impl<T: std::fmt::Display> Display<T> {
    /// Set the style for this display.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

/// State for a display component.
pub struct DisplayState {
    content: String,
    style: Style,
}

impl ViewMarker for DisplayState {}

impl Draw for DisplayState {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        let paragraph = Paragraph::new(self.content.as_str()).style(self.style);
        frame.render_widget(paragraph, area);
    }
}

impl<Output> ravel::State<Output> for DisplayState {
    fn run(&mut self, _output: &mut Output) {}
}

impl<T: std::fmt::Display> Builder<Tui> for Display<T> {
    type State = DisplayState;

    fn build(self, _cx: BuildCx) -> Self::State {
        DisplayState {
            content: self.value.to_string(),
            style: self.style,
        }
    }

    fn rebuild(self, _cx: RebuildCx, state: &mut Self::State) {
        state.content = self.value.to_string();
        state.style = self.style;
    }
}
