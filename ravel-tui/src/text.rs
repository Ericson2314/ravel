//! Text rendering builders.

use std::borrow::Cow;
use std::marker::PhantomData;

use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::{BuildCx, Builder, RebuildCx, Tui, UnitState};

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

    fn render(&self, cx: &BuildCx<'_>) {
        let mut paragraph = Paragraph::new(self.content.as_ref()).style(self.style);
        if self.wrap {
            paragraph = paragraph.wrap(Wrap { trim: true });
        }
        cx.frame().render_widget(paragraph, cx.area);
    }
}

impl Builder<Tui> for Text<'_> {
    type State = UnitState;

    fn build(self, cx: BuildCx<'_>) -> Self::State {
        self.render(&cx);
        UnitState
    }

    fn rebuild(self, cx: RebuildCx<'_>, _state: &mut Self::State) {
        self.render(&BuildCx {
            frame_wrapper: cx.frame_wrapper,
            area: cx.area,
            _marker: PhantomData,
        });
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

impl<'a> StyledLine<'a> {
    fn render(&self, cx: &BuildCx<'_>) {
        let line = Line::from(self.spans.clone());
        let paragraph = Paragraph::new(line);
        cx.frame().render_widget(paragraph, cx.area);
    }
}

impl Builder<Tui> for StyledLine<'_> {
    type State = UnitState;

    fn build(self, cx: BuildCx<'_>) -> Self::State {
        self.render(&cx);
        UnitState
    }

    fn rebuild(self, cx: RebuildCx<'_>, _state: &mut Self::State) {
        self.render(&BuildCx {
            frame_wrapper: cx.frame_wrapper,
            area: cx.area,
            _marker: PhantomData,
        });
    }
}

/// Builder for &'static str - allows using string literals directly as views.
impl Builder<Tui> for &'static str {
    type State = UnitState;

    fn build(self, cx: BuildCx<'_>) -> Self::State {
        let paragraph = Paragraph::new(self);
        cx.frame().render_widget(paragraph, cx.area);
        UnitState
    }

    fn rebuild(self, cx: RebuildCx<'_>, _state: &mut Self::State) {
        let paragraph = Paragraph::new(self);
        cx.frame().render_widget(paragraph, cx.area);
    }
}

/// Builder for String - allows using owned strings as views.
impl Builder<Tui> for String {
    type State = UnitState;

    fn build(self, cx: BuildCx<'_>) -> Self::State {
        let paragraph = Paragraph::new(self);
        cx.frame().render_widget(paragraph, cx.area);
        UnitState
    }

    fn rebuild(self, cx: RebuildCx<'_>, _state: &mut Self::State) {
        let paragraph = Paragraph::new(self);
        cx.frame().render_widget(paragraph, cx.area);
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

impl<T: std::fmt::Display> Builder<Tui> for Display<T> {
    type State = UnitState;

    fn build(self, cx: BuildCx<'_>) -> Self::State {
        let text = self.value.to_string();
        let paragraph = Paragraph::new(text).style(self.style);
        cx.frame().render_widget(paragraph, cx.area);
        UnitState
    }

    fn rebuild(self, cx: RebuildCx<'_>, _state: &mut Self::State) {
        let text = self.value.to_string();
        let paragraph = Paragraph::new(text).style(self.style);
        cx.frame().render_widget(paragraph, cx.area);
    }
}
