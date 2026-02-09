//! A ratatui/TUI backend for [`ravel`].
//!
//! This crate provides a terminal UI backend for ravel, allowing you to build
//! reactive TUI applications using ravel's declarative component model with
//! ratatui as the rendering layer.

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
    type BuildCx<'a> = BuildCx;
    type RebuildCx<'a> = RebuildCx;
}

/// Context for building TUI components.
///
/// Contains the area allocated for rendering. The Frame is passed separately
/// during the draw phase.
#[derive(Clone, Copy)]
pub struct BuildCx {
    /// The area allocated for this component
    pub area: Rect,
}

impl BuildCx {
    /// Create a new BuildCx with the given area.
    pub fn new(area: Rect) -> Self {
        Self { area }
    }

    /// Create a child context with a different area.
    pub fn with_area(&self, area: Rect) -> Self {
        Self { area }
    }
}

/// Context for rebuilding TUI components.
///
/// For immediate-mode TUI rendering, this is functionally identical to BuildCx.
#[derive(Clone, Copy)]
pub struct RebuildCx {
    /// The area allocated for this component
    pub area: Rect,
}

impl RebuildCx {
    /// Create a new RebuildCx with the given area.
    pub fn new(area: Rect) -> Self {
        Self { area }
    }

    /// Create a child context with a different area.
    pub fn with_area(&self, area: Rect) -> Self {
        Self { area }
    }

    /// Convert to a BuildCx with the same area.
    pub fn as_build_cx(&self) -> BuildCx {
        BuildCx { area: self.area }
    }
}

/// Trait for drawing state to a frame.
///
/// This is the core trait for immediate-mode rendering in ravel-tui.
/// State types implement this to render themselves when given a Frame.
pub trait Draw {
    /// Draw this state to the frame at the given area.
    fn draw(&self, frame: &mut Frame, area: Rect);
}

/// Marker trait for TUI view state types.
///
/// This is implemented for state types that represent valid TUI components.
pub trait ViewMarker: Draw {}

/// Unit state type for components with no state.
/// We use our own wrapper to allow implementing State trait.
pub struct UnitState;

impl ViewMarker for UnitState {}

impl Draw for UnitState {
    fn draw(&self, _frame: &mut Frame, _area: Rect) {
        // Nothing to draw
    }
}

impl<Output> ravel::State<Output> for UnitState {
    fn run(&mut self, _output: &mut Output) {}
}

impl<T: 'static, S: ViewMarker> ViewMarker for WithLocalState<T, S> {}
impl<S: ViewMarker, F> ViewMarker for AdaptState<S, F> {}

impl<T: 'static, S: Draw> Draw for WithLocalState<T, S> {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        self.inner.draw(frame, area);
    }
}

impl<S: Draw, F> Draw for AdaptState<S, F> {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        self.inner.draw(frame, area);
    }
}

// Implement ViewMarker for Option of ViewMarker types
impl<S: ViewMarker> ViewMarker for Option<S> {}

impl<S: Draw> Draw for Option<S> {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        if let Some(state) = self {
            state.draw(frame, area);
        }
    }
}

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

macro_rules! tuple_draw {
    () => {
        impl Draw for () {
            fn draw(&self, _frame: &mut Frame, _area: Rect) {
                // Nothing to draw for empty tuple
            }
        }
    };
    ($($a:ident),+) => {
        #[allow(non_camel_case_types)]
        impl<$($a),+> Draw for ($($a,)+)
        where
            $($a: Draw,)+
        {
            fn draw(&self, frame: &mut Frame, area: Rect) {
                let ($($a,)+) = self;
                $($a.draw(frame, area);)+
            }
        }
    };
}

tuple_draw!();
tuple_draw!(a);
tuple_draw!(a, b);
tuple_draw!(a, b, c);
tuple_draw!(a, b, c, d);
tuple_draw!(a, b, c, d, e);
tuple_draw!(a, b, c, d, e, f);
tuple_draw!(a, b, c, d, e, f, g);
tuple_draw!(a, b, c, d, e, f, g, h);

// Empty tuple is a ViewMarker
impl ViewMarker for () {}

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
pub struct OptionState<S> {
    pub inner: Option<S>,
    /// The area this option was built/rebuilt with
    pub area: Rect,
}

impl<S: ViewMarker> ViewMarker for OptionState<S> {}

impl<S: Draw> Draw for OptionState<S> {
    fn draw(&self, frame: &mut Frame, _area: Rect) {
        if let Some(state) = &self.inner {
            state.draw(frame, self.area);
        }
    }
}

impl<S: ravel::State<O>, O> ravel::State<O> for OptionState<S> {
    fn run(&mut self, output: &mut O) {
        if let Some(state) = &mut self.inner {
            state.run(output);
        }
    }
}

// Implement Builder for Option<B> where B: Builder
impl<B: Builder<Tui>> Builder<Tui> for Option<B>
where
    B::State: Draw,
{
    type State = OptionState<B::State>;

    fn build(self, cx: BuildCx) -> Self::State {
        OptionState {
            inner: self.map(|b| b.build(cx)),
            area: cx.area,
        }
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        state.area = cx.area;
        match (self, &mut state.inner) {
            (Some(builder), Some(inner_state)) => {
                builder.rebuild(cx, inner_state);
            }
            (Some(builder), inner @ None) => {
                *inner = Some(builder.build(cx.as_build_cx()));
            }
            (None, inner) => {
                *inner = None;
            }
        }
    }
}

/// Run a frame with build or rebuild, then draw.
///
/// This function handles the two-phase rendering:
/// 1. Build/rebuild phase: construct or update the state tree
/// 2. Draw phase: render the state tree to the frame
///
/// # Example
/// ```ignore
/// let mut ui_state: UiState<MyState> = UiState::new();
/// terminal.draw(|f| {
///     run_frame(f, &mut ui_state, || my_builder(&app));
/// })?;
/// ```
pub fn run_frame<S: Draw, B, F>(frame: &mut Frame<'_>, state: &mut UiState<S>, make_builder: F)
where
    B: Builder<Tui, State = S>,
    F: FnOnce() -> B,
{
    let area = frame.area();
    let builder = make_builder();

    match &mut state.view_state {
        None => {
            let new_state = builder.build(BuildCx::new(area));
            state.view_state = Some(new_state);
        }
        Some(s) => {
            builder.rebuild(RebuildCx::new(area), s);
        }
    }

    // Draw phase: render the state tree to the frame
    if let Some(s) = &state.view_state {
        s.draw(frame, area);
    }
}

/// Complete UI state wrapping the view state.
pub struct UiState<S> {
    /// The view tree state.
    pub view_state: Option<S>,
}

impl<S> UiState<S> {
    /// Create a new UiState.
    pub fn new() -> Self {
        Self { view_state: None }
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
