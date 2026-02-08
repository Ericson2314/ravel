//! Throbber/spinner widget builder.

use ratatui::style::{Modifier, Style};
use throbber_widgets_tui::{CANADIAN, Throbber, ThrobberState, WhichUse};

use crate::{BuildCx, Builder, RebuildCx, Tui, ViewMarker};

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

    fn make_widget(&self) -> Throbber<'_> {
        Throbber::default()
            .label(&self.label)
            .style(self.style)
            .throbber_style(self.throbber_style)
            .throbber_set(CANADIAN)
            .use_type(WhichUse::Spin)
    }
}

impl Builder<Tui> for ThrobberBuilder {
    type State = ThrobberBuilderState;

    fn build(self, cx: BuildCx<'_>) -> Self::State {
        let mut state = ThrobberState::default();
        let widget = self.make_widget();
        cx.frame().render_stateful_widget(widget, cx.area, &mut state);
        ThrobberBuilderState { state }
    }

    fn rebuild(self, cx: RebuildCx<'_>, state: &mut Self::State) {
        // Advance the animation
        state.state.calc_next();

        let widget = self.make_widget();
        cx.frame()
            .render_stateful_widget(widget, cx.area, &mut state.state);
    }
}

/// State for a throbber builder.
pub struct ThrobberBuilderState {
    state: ThrobberState,
}

impl ThrobberBuilderState {
    /// Advance the throbber animation.
    pub fn tick(&mut self) {
        self.state.calc_next();
    }
}

impl ViewMarker for ThrobberBuilderState {}

impl<Output> ravel::State<Output> for ThrobberBuilderState {
    fn run(&mut self, _output: &mut Output) {
        // Throbber doesn't emit events
    }
}
