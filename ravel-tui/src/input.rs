//! Text input widget builder.

use std::marker::PhantomData;

use ratatui::style::Style;
use ratatui::widgets::Paragraph;
use tui_input::Input;

use crate::{register_input, BuildCx, Builder, RebuildCx, Tui, ViewMarker};

/// A text input builder.
pub struct InputBuilder {
    style: Style,
    /// Whether to mask the input (for passwords).
    masked: bool,
}

/// Create an input builder.
pub fn input() -> InputBuilder {
    InputBuilder {
        style: Style::default(),
        masked: false,
    }
}

impl InputBuilder {
    /// Set the input style.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Mask the input (for passwords).
    pub fn masked(mut self) -> Self {
        self.masked = true;
        self
    }
}

impl Builder<Tui> for InputBuilder {
    type State = InputBuilderState;

    fn build(self, cx: BuildCx<'_>) -> Self::State {
        let input = Input::default();
        let mut state = InputBuilderState {
            input,
            style: self.style,
            masked: self.masked,
        };
        state.render(&cx);
        // Register this input for dialog state access
        register_input(&mut state as *mut InputBuilderState);
        state
    }

    fn rebuild(self, cx: RebuildCx<'_>, state: &mut Self::State) {
        state.style = self.style;
        state.masked = self.masked;
        state.render(&BuildCx {
            frame_wrapper: cx.frame_wrapper,
            area: cx.area,
            _marker: PhantomData,
        });
        // Register this input for dialog state access
        register_input(state as *mut InputBuilderState);
    }
}

/// State for an input builder.
pub struct InputBuilderState {
    input: Input,
    style: Style,
    masked: bool,
}

impl InputBuilderState {
    fn render(&self, cx: &BuildCx<'_>) {
        let width = cx.area.width as usize;
        let scroll = self.input.visual_scroll(width);

        let display_value = if self.masked {
            "*".repeat(self.input.value().len())
        } else {
            self.input.value().to_string()
        };

        let paragraph = Paragraph::new(display_value)
            .style(self.style)
            .scroll((0, scroll as u16));

        cx.frame().render_widget(paragraph, cx.area);

        // Set cursor position
        let cursor_x = cx.area.x + (self.input.visual_cursor().max(scroll) - scroll) as u16;
        cx.frame().set_cursor_position((cursor_x, cx.area.y));
    }

    /// Get the current input value.
    pub fn value(&self) -> &str {
        self.input.value()
    }

    /// Handle a character input.
    pub fn insert_char(&mut self, c: char) {
        self.input.handle(tui_input::InputRequest::InsertChar(c));
    }

    /// Handle backspace.
    pub fn delete_prev_char(&mut self) {
        self.input.handle(tui_input::InputRequest::DeletePrevChar);
    }

    /// Handle delete previous word.
    pub fn delete_prev_word(&mut self) {
        self.input.handle(tui_input::InputRequest::DeletePrevWord);
    }

    /// Move cursor left.
    pub fn go_to_prev_char(&mut self) {
        self.input.handle(tui_input::InputRequest::GoToPrevChar);
    }

    /// Move cursor right.
    pub fn go_to_next_char(&mut self) {
        self.input.handle(tui_input::InputRequest::GoToNextChar);
    }

    /// Move cursor to previous word.
    pub fn go_to_prev_word(&mut self) {
        self.input.handle(tui_input::InputRequest::GoToPrevWord);
    }

    /// Move cursor to next word.
    pub fn go_to_next_word(&mut self) {
        self.input.handle(tui_input::InputRequest::GoToNextWord);
    }

    /// Clear the input.
    pub fn clear(&mut self) {
        self.input.reset();
    }

    /// Get mutable access to the underlying Input.
    pub fn inner_mut(&mut self) -> &mut Input {
        &mut self.input
    }
}

impl ViewMarker for InputBuilderState {}

impl<Output> ravel::State<Output> for InputBuilderState {
    fn run(&mut self, _output: &mut Output) {
        // Input doesn't emit events on its own
    }
}
