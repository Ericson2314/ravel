//! A ratatui/TUI backend for [`ravel`].
//!
//! This crate provides a terminal UI backend for ravel, allowing you to build
//! reactive TUI applications using ravel's declarative component model with
//! ratatui as the rendering layer.

use std::marker::PhantomData;

use ravel::{AdaptState, Builder, CxRep, WithLocalState};

pub use ratatui;
use ratatui::layout::Rect;
use ratatui::Frame;

mod any;
mod block;
mod input;
mod layout;
mod list;
mod modal;
mod text;
mod throbber;

pub use any::*;
pub use block::*;
pub use input::*;
pub use layout::*;
pub use list::*;
pub use modal::*;
pub use text::*;
pub use throbber::*;

// Re-export useful ravel items
pub use ravel::with;

/// The TUI backend marker type.
pub struct Tui;

impl CxRep for Tui {
    type BuildCx<'a> = BuildCx<'a>;
    type RebuildCx<'a> = RebuildCx<'a>;
}

/// Opaque wrapper to hold Frame pointer.
///
/// This uses raw pointers to avoid lifetime propagation issues with Frame's
/// invariant lifetime parameters.
pub struct FrameWrapper {
    frame: *mut (),
    area: Rect,
}

impl FrameWrapper {
    /// Create a new FrameWrapper from a mutable frame reference.
    pub fn new(frame: &mut Frame<'_>) -> Self {
        Self {
            frame: frame as *mut Frame<'_> as *mut (),
            area: frame.area(),
        }
    }

    /// Get the frame area.
    pub fn area(&self) -> Rect {
        self.area
    }

    /// Get mutable access to the frame for rendering.
    ///
    /// # Safety
    /// The caller must ensure this is only called while the original frame is valid.
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn get_frame<'a>(&self) -> &'a mut Frame<'a> {
        unsafe { &mut *(self.frame as *mut Frame<'a>) }
    }
}

/// Context for building TUI components.
///
/// Contains a reference to the frame buffer and the area allocated for rendering.
#[derive(Clone, Copy)]
pub struct BuildCx<'a> {
    /// The frame wrapper.
    frame_wrapper: *const FrameWrapper,
    /// The area allocated for this component
    pub area: Rect,
    _marker: PhantomData<&'a FrameWrapper>,
}

impl<'a> BuildCx<'a> {
    /// Create a new BuildCx from a frame wrapper and area.
    pub fn new(frame_wrapper: &'a FrameWrapper, area: Rect) -> Self {
        Self {
            frame_wrapper,
            area,
            _marker: PhantomData,
        }
    }

    /// Get mutable access to the frame for rendering.
    pub fn frame(&self) -> &mut Frame<'_> {
        unsafe { (*self.frame_wrapper).get_frame() }
    }

    /// Create a child context with a different area.
    pub fn with_area(&self, area: Rect) -> Self {
        Self {
            frame_wrapper: self.frame_wrapper,
            area,
            _marker: PhantomData,
        }
    }
}

/// Context for rebuilding TUI components.
///
/// For immediate-mode TUI rendering, this is functionally identical to BuildCx.
#[derive(Clone, Copy)]
pub struct RebuildCx<'a> {
    /// The frame wrapper.
    frame_wrapper: *const FrameWrapper,
    /// The area allocated for this component
    pub area: Rect,
    _marker: PhantomData<&'a FrameWrapper>,
}

impl<'a> RebuildCx<'a> {
    /// Create a new RebuildCx from a frame wrapper and area.
    pub fn new(frame_wrapper: &'a FrameWrapper, area: Rect) -> Self {
        Self {
            frame_wrapper,
            area,
            _marker: PhantomData,
        }
    }

    /// Get mutable access to the frame for rendering.
    pub fn frame(&self) -> &mut Frame<'_> {
        unsafe { (*self.frame_wrapper).get_frame() }
    }

    /// Create a child context with a different area.
    pub fn with_area(&self, area: Rect) -> Self {
        Self {
            frame_wrapper: self.frame_wrapper,
            area,
            _marker: PhantomData,
        }
    }

    /// Convert to a BuildCx with the same area.
    ///
    /// This is useful for calling build functions from rebuild contexts.
    pub fn as_build_cx(&self) -> BuildCx<'a> {
        BuildCx {
            frame_wrapper: self.frame_wrapper,
            area: self.area,
            _marker: PhantomData,
        }
    }

    /// Convert to a BuildCx with a different area.
    pub fn build_cx_with_area(&self, area: Rect) -> BuildCx<'a> {
        BuildCx {
            frame_wrapper: self.frame_wrapper,
            area,
            _marker: PhantomData,
        }
    }
}

/// Marker trait for TUI view state types.
///
/// This is implemented for state types that represent valid TUI components.
pub trait ViewMarker {}

/// Unit state type for components with no state.
/// We use our own wrapper to allow implementing State trait.
pub struct UnitState;

impl ViewMarker for UnitState {}

impl<Output> ravel::State<Output> for UnitState {
    fn run(&mut self, _output: &mut Output) {}
}

impl<T: 'static, S: ViewMarker> ViewMarker for WithLocalState<T, S> {}
impl<S: ViewMarker, F> ViewMarker for AdaptState<S, F> {}

// Implement ViewMarker for Option of ViewMarker types
impl<S: ViewMarker> ViewMarker for Option<S> {}

macro_rules! tuple_view_marker {
    ($($a:ident),*) => {
        #[allow(non_camel_case_types)]
        impl<$($a),*> ViewMarker for ($($a,)*)
        where
            $($a: ViewMarker,)*
        {
        }
    };
}

tuple_view_marker!(a);
tuple_view_marker!(a, b);
tuple_view_marker!(a, b, c);
tuple_view_marker!(a, b, c, d);
tuple_view_marker!(a, b, c, d, e);
tuple_view_marker!(a, b, c, d, e, f);
tuple_view_marker!(a, b, c, d, e, f, g);
tuple_view_marker!(a, b, c, d, e, f, g, h);

/// Trait for TUI views.
///
/// These types can be used as components in the TUI component tree.
pub trait View: Builder<Tui, State = Self::ViewState> {
    type ViewState: ViewMarker;
}

impl<T, S: ViewMarker> View for T
where
    T: Builder<Tui, State = S>,
{
    type ViewState = S;
}

/// Wrapper for Option state to allow implementing State trait.
pub struct OptionState<S>(pub Option<S>);

impl<S: ViewMarker> ViewMarker for OptionState<S> {}

impl<S: ravel::State<O>, O> ravel::State<O> for OptionState<S> {
    fn run(&mut self, output: &mut O) {
        if let Some(state) = &mut self.0 {
            state.run(output);
        }
    }
}

// Implement Builder for Option<B> where B: Builder
impl<B: Builder<Tui>> Builder<Tui> for Option<B> {
    type State = OptionState<B::State>;

    fn build(self, cx: BuildCx<'_>) -> Self::State {
        OptionState(self.map(|b| b.build(cx)))
    }

    fn rebuild(self, cx: RebuildCx<'_>, state: &mut Self::State) {
        match (self, &mut state.0) {
            (Some(builder), Some(inner_state)) => {
                builder.rebuild(cx, inner_state);
            }
            (Some(builder), state @ None) => {
                *state = Some(builder.build(BuildCx {
                    frame_wrapper: cx.frame_wrapper,
                    area: cx.area,
                    _marker: PhantomData,
                }));
            }
            (None, state) => {
                *state = None;
            }
        }
    }
}

/// Run a frame with build or rebuild.
///
/// This function handles the lifetime complexity of ratatui's Frame by using
/// a closure-based API. The state is passed by mutable reference and updated in place.
///
/// # Example
/// ```ignore
/// let mut ui_state: Option<MyState> = None;
/// terminal.draw(|f| {
///     run_frame(f, &mut ui_state, || my_builder(&app));
/// })?;
/// ```
pub fn run_frame<S, B, F>(frame: &mut Frame<'_>, state: &mut UiState<S>, make_builder: F)
where
    B: Builder<Tui, State = S>,
    F: FnOnce() -> B,
{
    // Clear dialog state at start of each frame
    state.dialog_state.clear();
    clear_input_registration();

    let area = frame.area();
    let frame_wrapper = FrameWrapper::new(frame);
    let builder = make_builder();

    match &mut state.view_state {
        None => {
            let new_state = builder.build(BuildCx::new(&frame_wrapper, area));
            // Safety: S has no lifetime parameters (it's a concrete type passed by the caller).
            // The build() function returns S, which we know doesn't borrow from frame_wrapper
            // because the Builder trait's State type has no lifetime connection to BuildCx.
            // We use ptr::write to avoid the borrow checker's conservative analysis.
            unsafe {
                let state_ptr = &mut state.view_state as *mut Option<S>;
                std::ptr::write(state_ptr, Some(new_state));
            }
        }
        Some(s) => {
            builder.rebuild(RebuildCx::new(&frame_wrapper, area), s);
        }
    }

    // Update dialog state from thread-local registration
    state.dialog_state.update_from_registration();
}

use std::cell::RefCell;

thread_local! {
    static DIALOG_INPUT: RefCell<Option<*mut InputBuilderState>> = const { RefCell::new(None) };
}

/// Register an input state for later access via DialogState.
pub(crate) fn register_input(state: *mut InputBuilderState) {
    DIALOG_INPUT.with(|cell| {
        *cell.borrow_mut() = Some(state);
    });
}

/// Clear the registered input state.
fn clear_input_registration() {
    DIALOG_INPUT.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// Get the registered input state.
fn get_registered_input() -> Option<*mut InputBuilderState> {
    DIALOG_INPUT.with(|cell| *cell.borrow())
}

/// A container for dialog-related state that can be accessed from outside the view tree.
///
/// This is populated by modal dialogs containing input fields during rendering.
pub struct DialogState {
    input: Option<*mut InputBuilderState>,
}

impl DialogState {
    /// Create a new empty DialogState.
    pub fn new() -> Self {
        Self { input: None }
    }

    /// Get mutable access to the input state, if present.
    ///
    /// # Safety
    /// This returns a mutable reference from a raw pointer. The caller must ensure
    /// the DialogState is only used while the UI state it references is valid.
    pub fn input_state(&mut self) -> Option<&mut InputBuilderState> {
        self.input.map(|ptr| unsafe { &mut *ptr })
    }

    /// Clear all state references.
    pub fn clear(&mut self) {
        self.input = None;
    }

    /// Update from thread-local registered input.
    fn update_from_registration(&mut self) {
        self.input = get_registered_input();
    }
}

impl Default for DialogState {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete UI state including view state and dialog state.
pub struct UiState<S> {
    /// The view tree state.
    pub view_state: Option<S>,
    /// Dialog state for accessing input fields.
    pub dialog_state: DialogState,
}

impl<S> UiState<S> {
    /// Create a new UiState.
    pub fn new() -> Self {
        Self {
            view_state: None,
            dialog_state: DialogState::new(),
        }
    }
}

impl<S> Default for UiState<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[doc(hidden)]
pub use ravel::State as ViewState;

/// A convenience macro for declaring a [`trait@View`] type.
///
/// Takes as a parameter the `Output` type of the [`trait@View`]'s
/// [`Builder::State`]. If no parameter is given, defaults to `()`.
#[macro_export]
macro_rules! View {
    () => {
        impl $crate::View<
            ViewState = impl use<> + $crate::ViewMarker + $crate::ViewState<()>
        >
    };
    ($output:ty) => {
        impl $crate::View<
            ViewState = impl use<> + $crate::ViewMarker + $crate::ViewState<$output>
        >
    };
}
