//! Layout builders for arranging child components.

use paste::paste;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use crate::{BuildCx, Builder, Draw, RebuildCx, Tui, UnitState, ViewMarker};

/// Direction for layout (vertical or horizontal).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutDirection {
    Vertical,
    Horizontal,
}

impl From<LayoutDirection> for Direction {
    fn from(dir: LayoutDirection) -> Self {
        match dir {
            LayoutDirection::Vertical => Direction::Vertical,
            LayoutDirection::Horizontal => Direction::Horizontal,
        }
    }
}

/// A layout builder that arranges children according to constraints.
pub struct LayoutBuilder<Children> {
    direction: LayoutDirection,
    constraints: Vec<Constraint>,
    children: Children,
}

/// Create a vertical stack layout.
pub fn vstack<C>(children: C) -> LayoutBuilder<C> {
    LayoutBuilder {
        direction: LayoutDirection::Vertical,
        constraints: Vec::new(),
        children,
    }
}

/// Create a horizontal row layout.
pub fn hstack<C>(children: C) -> LayoutBuilder<C> {
    LayoutBuilder {
        direction: LayoutDirection::Horizontal,
        constraints: Vec::new(),
        children,
    }
}

impl<Children> LayoutBuilder<Children> {
    /// Set the constraints for this layout.
    pub fn constraints<I: IntoIterator<Item = Constraint>>(mut self, constraints: I) -> Self {
        self.constraints = constraints.into_iter().collect();
        self
    }

    fn split(&self, area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(self.direction.into())
            .constraints(&self.constraints)
            .split(area)
            .to_vec()
    }
}

/// State for layout builders.
pub struct LayoutState<S> {
    children: S,
    /// The areas for each child, computed during build/rebuild
    areas: Vec<Rect>,
}

// ViewMarker impls are in the macro below for each tuple size
impl ViewMarker for LayoutState<()> {}

impl<Output, S: ravel::State<Output>> ravel::State<Output> for LayoutState<S> {
    fn run(&mut self, output: &mut Output) {
        self.children.run(output);
    }
}

// Implement Builder for empty tuple
impl Builder<Tui> for LayoutBuilder<()> {
    type State = LayoutState<()>;

    fn build(self, _cx: BuildCx) -> Self::State {
        LayoutState {
            children: (),
            areas: Vec::new(),
        }
    }

    fn rebuild(self, _cx: RebuildCx, _state: &mut Self::State) {}
}

impl Draw for LayoutState<()> {
    fn draw(&self, _frame: &mut Frame, _area: Rect) {
        // Nothing to draw for empty layout
    }
}

// Implement Builder for tuple children using a macro
macro_rules! impl_layout_builder {
    ($($idx:tt: $name:ident),*) => {
        #[allow(non_camel_case_types)]
        impl<$($name: Builder<Tui>),*> Builder<Tui> for LayoutBuilder<($($name,)*)>
        where
            $($name::State: Draw,)*
        {
            type State = LayoutState<($($name::State,)*)>;

            fn build(self, cx: BuildCx) -> Self::State {
                let chunks = self.split(cx.area);
                let ($($name,)*) = self.children;

                LayoutState {
                    children: (
                        $($name.build(cx.with_area(chunks[$idx])),)*
                    ),
                    areas: chunks,
                }
            }

            fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
                let chunks = self.split(cx.area);
                state.areas = chunks.clone();
                let ($($name,)*) = self.children;
                let ($(paste!([< state_ $name >]),)*) = &mut state.children;

                $($name.rebuild(cx.with_area(chunks[$idx]), paste!([< state_ $name >]));)*
            }
        }

        #[allow(non_camel_case_types)]
        impl<$($name: Draw),*> Draw for LayoutState<($($name,)*)> {
            fn draw(&self, frame: &mut Frame, _area: Rect) {
                let ($($name,)*) = &self.children;
                $(
                    if let Some(area) = self.areas.get($idx) {
                        $name.draw(frame, *area);
                    }
                )*
            }
        }

        #[allow(non_camel_case_types)]
        impl<$($name: ViewMarker),*> ViewMarker for LayoutState<($($name,)*)> {}
    };
}

impl_layout_builder!(0: a);
impl_layout_builder!(0: a, 1: b);
impl_layout_builder!(0: a, 1: b, 2: c);
impl_layout_builder!(0: a, 1: b, 2: c, 3: d);
impl_layout_builder!(0: a, 1: b, 2: c, 3: d, 4: e);
impl_layout_builder!(0: a, 1: b, 2: c, 3: d, 4: e, 5: f);
impl_layout_builder!(0: a, 1: b, 2: c, 3: d, 4: e, 5: f, 6: g);
impl_layout_builder!(0: a, 1: b, 2: c, 3: d, 4: e, 5: f, 6: g, 7: h);

/// A spacer that takes up space but renders nothing.
pub struct Spacer;

/// Create a spacer element.
pub fn spacer() -> Spacer {
    Spacer
}

impl Builder<Tui> for Spacer {
    type State = UnitState;

    fn build(self, _cx: BuildCx) -> Self::State {
        UnitState
    }

    fn rebuild(self, _cx: RebuildCx, _state: &mut Self::State) {}
}

/// Center content within the available area.
pub struct Center<Body> {
    width: Option<u16>,
    height: Option<u16>,
    body: Body,
}

/// Create a centered container.
pub fn center<Body>(body: Body) -> Center<Body> {
    Center {
        width: None,
        height: None,
        body,
    }
}

impl<Body> Center<Body> {
    /// Set a fixed width for the centered content.
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set a fixed height for the centered content.
    pub fn height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    fn centered_area(&self, area: Rect) -> Rect {
        let width = self.width.unwrap_or(area.width);
        let height = self.height.unwrap_or(area.height);

        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;

        Rect {
            x,
            y,
            width: width.min(area.width),
            height: height.min(area.height),
        }
    }
}

/// State for a centered container.
pub struct CenterState<S> {
    body: S,
    area: Rect,
}

impl<S: ViewMarker> ViewMarker for CenterState<S> {}

impl<S: Draw> Draw for CenterState<S> {
    fn draw(&self, frame: &mut Frame, _area: Rect) {
        self.body.draw(frame, self.area);
    }
}

impl<Output, S: ravel::State<Output>> ravel::State<Output> for CenterState<S> {
    fn run(&mut self, output: &mut Output) {
        self.body.run(output);
    }
}

impl<Body: Builder<Tui>> Builder<Tui> for Center<Body>
where
    Body::State: Draw,
{
    type State = CenterState<Body::State>;

    fn build(self, cx: BuildCx) -> Self::State {
        let centered = self.centered_area(cx.area);
        CenterState {
            body: self.body.build(cx.with_area(centered)),
            area: centered,
        }
    }

    fn rebuild(self, cx: RebuildCx, state: &mut Self::State) {
        let centered = self.centered_area(cx.area);
        state.area = centered;
        self.body.rebuild(cx.with_area(centered), &mut state.body);
    }
}
