//! Text input widget builder.

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use tui_input::Input;

use crate::{BuildCx, Builder, Draw, RebuildCx, Tui, ViewMarker};

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

    fn build(self, _cx: BuildCx) -> Self::State {
        let input = Input::default();
        let state = InputBuilderState {
            input,
            style: self.style,
            masked: self.masked,
        };
        // Register this input for dialog state access
        // Note: This uses a raw pointer, but it's registered after build
        // and cleared at the start of each frame
        state
    }

    fn rebuild(self, _cx: RebuildCx, state: &mut Self::State) {
        state.style = self.style;
        state.masked = self.masked;
    }
}

/// State for an input builder.
pub struct InputBuilderState {
    input: Input,
    style: Style,
    masked: bool,
}

impl ViewMarker for InputBuilderState {}

impl Draw for InputBuilderState {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        let width = area.width as usize;
        let scroll = self.input.visual_scroll(width);

        let display_value = if self.masked {
            "*".repeat(self.input.value().len())
        } else {
            self.input.value().to_string()
        };

        let paragraph = Paragraph::new(display_value)
            .style(self.style)
            .scroll((0, scroll as u16));

        frame.render_widget(paragraph, area);

        // Set cursor position
        let cursor_x = area.x + (self.input.visual_cursor().max(scroll) - scroll) as u16;
        frame.set_cursor_position((cursor_x, area.y));
    }
}

impl InputBuilderState {
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

impl<Output> ravel::State<Output> for InputBuilderState {
    fn run(&mut self, _output: &mut Output) {
        // Input doesn't emit events on its own
    }
}
