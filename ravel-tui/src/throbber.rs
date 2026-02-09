//! Throbber/spinner widget builder.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;
use throbber_widgets_tui::{CANADIAN, Throbber, ThrobberState, WhichUse};

use crate::{BuildCx, Builder, Draw, RebuildCx, Tui, ViewMarker};

/// A throbber/spinner builder.
pub struct ThrobberBuilder {
    label: String,
    style: Style,
    throbber_style: Style,
}

/// Create a throbber builder.
pub fn throbber(label: impl Into<String>) -> ThrobberBuilder {
    ThrobberBuilder {
        label: label.into(),
        style: Style::default(),
        throbber_style: Style::default().add_modifier(Modifier::BOLD),
    }
}

impl ThrobberBuilder {
    /// Set the label style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Set the throbber animation style.
    pub fn throbber_style(mut self, style: Style) -> Self {
        self.throbber_style = style;
        self
    }
}

impl Builder<Tui> for ThrobberBuilder {
    type State = ThrobberBuilderState;

    fn build(self, _cx: BuildCx) -> Self::State {
        let state = ThrobberState::default();
        ThrobberBuilderState {
            state,
            label: self.label,
            style: self.style,
            throbber_style: self.throbber_style,
        }
    }

    fn rebuild(self, _cx: RebuildCx, state: &mut Self::State) {
        // Advance the animation
        state.state.calc_next();
        // Update config
        state.label = self.label;
        state.style = self.style;
        state.throbber_style = self.throbber_style;
    }
}

/// State for a throbber builder.
pub struct ThrobberBuilderState {
    state: ThrobberState,
    label: String,
    style: Style,
    throbber_style: Style,
}

impl ViewMarker for ThrobberBuilderState {}

impl Draw for ThrobberBuilderState {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        let widget = Throbber::default()
            .label(&self.label)
            .style(self.style)
            .throbber_style(self.throbber_style)
            .throbber_set(CANADIAN)
            .use_type(WhichUse::Spin);

        // We need to clone the state to use it mutably
        let mut state = self.state.clone();
        frame.render_stateful_widget(widget, area, &mut state);
    }
}

impl ThrobberBuilderState {
    /// Advance the throbber animation.
    pub fn tick(&mut self) {
        self.state.calc_next();
    }
}

impl<Output> ravel::State<Output> for ThrobberBuilderState {
    fn run(&mut self, _output: &mut Output) {
        // Throbber doesn't emit events
    }
}
