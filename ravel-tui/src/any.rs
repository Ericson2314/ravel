//! Type-erased view wrapper for dynamic dispatch.

use std::{any::Any, marker::PhantomData, ops::DerefMut};

use ratatui::layout::Rect;
use ratatui::Frame;
use ravel::State;

use crate::{BuildCx, Builder, Draw, RebuildCx, Tui, View, ViewMarker};

/// A wrapper around a [`trait@View`], erasing its [`State`] type.
pub struct AnyView<V: View, Output> {
    inner: V,
    phantom: PhantomData<fn(&mut Output)>,
}

/// Trait object for combined State + Draw functionality
trait DynViewState<Output>: State<Output> {
    fn draw(&self, frame: &mut Frame, area: Rect);
    fn as_mut_dyn_any(&mut self) -> &mut dyn Any;
}

impl<T, Output> DynViewState<Output> for T
where
    T: State<Output> + Draw + Any,
{
    fn draw(&self, frame: &mut Frame, area: Rect) {
        Draw::draw(self, frame, area)
    }

    fn as_mut_dyn_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl<V: View, Output: 'static> Builder<Tui> for AnyView<V, Output>
where
    V::ViewState: State<Output> + Draw + Any,
{
    type State = AnyState<Output>;

    fn build(self, cx: BuildCx) -> Self::State {
        AnyState {
            state: Box::new(self.inner.build(cx)),
            area: cx.area,
        }
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        state.area = cx.area;
        match (DynViewState::as_mut_dyn_any(state.state.as_mut()).deref_mut() as &mut dyn Any)
            .downcast_mut::<V::ViewState>()
        {
            Some(inner_state) => self.inner.rebuild(cx, inner_state),
            None => {
                // Type changed, rebuild from scratch
                state.state = Box::new(self.inner.build(cx.as_build_cx()));
            }
        }
    }
}

/// The state for an [`AnyView`].
pub struct AnyState<Output> {
    state: Box<dyn DynViewState<Output>>,
    area: Rect,
}

impl<Output: 'static> State<Output> for AnyState<Output> {
    fn run(&mut self, output: &mut Output) {
        self.state.run(output)
    }
}

impl<Output: 'static> ViewMarker for AnyState<Output> {}

impl<Output: 'static> Draw for AnyState<Output> {
    fn draw(&self, frame: &mut Frame, _area: Rect) {
        self.state.draw(frame, self.area);
    }
}

/// Wraps a [`trait@View`], erasing its [`State`] type.
///
/// Using this inside a [`ravel::with`] callback makes it possible to dynamically
/// choose an implementation type.
pub fn any<V: View, Output: 'static>(view: V) -> AnyView<V, Output>
where
    V::ViewState: Draw + Any,
{
    AnyView {
        inner: view,
        phantom: PhantomData,
    }
}
