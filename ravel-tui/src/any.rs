//! Type-erased view wrapper for dynamic dispatch.

use std::{any::Any, marker::PhantomData, ops::DerefMut};

use ravel::State;

use crate::{BuildCx, Builder, RebuildCx, Tui, View, ViewMarker};

/// A wrapper around a [`trait@View`], erasing its [`State`] type.
pub struct AnyView<V: View, Output> {
    inner: V,
    phantom: PhantomData<fn(&mut Output)>,
}

impl<V: View, Output: 'static> Builder<Tui> for AnyView<V, Output>
where
    V::ViewState: State<Output>,
{
    type State = AnyState<Output>;

    fn build(self, cx: BuildCx<'_>) -> Self::State {
        AnyState {
            state: Box::new(self.inner.build(cx)),
        }
    }

    fn rebuild(self, cx: RebuildCx<'_>, state: &mut Self::State) {
        match (state.state.as_mut_dyn_any().deref_mut() as &mut dyn Any)
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
    state: Box<dyn State<Output>>,
}

impl<Output: 'static> State<Output> for AnyState<Output> {
    fn run(&mut self, output: &mut Output) {
        self.state.run(output)
    }
}

impl<Output> ViewMarker for AnyState<Output> {}

/// Wraps a [`trait@View`], erasing its [`State`] type.
///
/// Using this inside a [`ravel::with`] callback makes it possible to dynamically
/// choose an implementation type.
pub fn any<V: View, Output: 'static>(view: V) -> AnyView<V, Output> {
    AnyView {
        inner: view,
        phantom: PhantomData,
    }
}
