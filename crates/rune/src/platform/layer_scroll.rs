//! Pure conversion of one `wl_pointer.frame`'s scroll axes.
//!
//! Wayland's continuous values are surface-coordinate distances. Discrete values are
//! wheel steps; value120 is 1/120 of a step. Match vendored winit's Wayland conversion:
//! it negates Wayland's sign and applies 120/10 points per axis unit to finger and
//! continuous sources (Chromium's `kWheelDelta / kAxisValueScale`).

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Source {
    #[default]
    Unknown,
    Wheel,
    Finger,
    Continuous,
    WheelTilt,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Axis {
    pub raw: f64,
    pub discrete: Option<i32>,
    pub value120: Option<i32>,
    pub stopped: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Frame {
    pub horizontal: Axis,
    pub vertical: Axis,
    pub source: Source,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Scroll {
    /// Rune convention: positive means content moves right/down.
    pub dx: f32,
    pub dy: f32,
    pub precise: bool,
    pub began: bool,
    pub ended: bool,
}

/// The axes still participating in one continuous gesture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Axes {
    Horizontal,
    Vertical,
    Both,
}

impl Axes {
    fn has_horizontal(self) -> bool { matches!(self, Self::Horizontal | Self::Both) }
    fn has_vertical(self) -> bool { matches!(self, Self::Vertical | Self::Both) }

    fn with(self, horizontal: bool, vertical: bool) -> Option<Self> {
        match (self.has_horizontal() || horizontal, self.has_vertical() || vertical) {
            (true, true) => Some(Self::Both),
            (true, false) => Some(Self::Horizontal),
            (false, true) => Some(Self::Vertical),
            (false, false) => None,
        }
    }

    fn without(self, horizontal: bool, vertical: bool) -> Option<Self> {
        match (self.has_horizontal() && !horizontal, self.has_vertical() && !vertical) {
            (true, true) => Some(Self::Both),
            (true, false) => Some(Self::Horizontal),
            (false, true) => Some(Self::Vertical),
            (false, false) => None,
        }
    }
}

/// Gesture mode; source and participating axes exist only while a gesture is active.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum State {
    #[default]
    Idle,
    Active { axes: Axes, source: Source },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Event {
    Frame(Frame),
    Reset,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Effect {
    Emit(Scroll),
}

/// One Wayland protocol event in, at most one wheel event out.
pub(crate) fn step(state: &mut State, event: Event) -> Option<Effect> {
    let Event::Frame(frame) = event else {
        *state = State::Idle;
        return None;
    };
    let (old_axes, old_source) = match *state {
        State::Idle => (None, Source::Unknown),
        State::Active { axes, source } => (Some(axes), source),
    };
    let source = if frame.source == Source::Unknown { old_source } else { frame.source };
    let continuous = matches!(source, Source::Finger | Source::Continuous);
    let mut axes = old_axes;
    if continuous {
        let started_horizontal = frame.horizontal.raw != 0.0
            || frame.horizontal.discrete.is_some()
            || frame.horizontal.value120.is_some();
        let started_vertical = frame.vertical.raw != 0.0
            || frame.vertical.discrete.is_some()
            || frame.vertical.value120.is_some();
        axes = match axes {
            Some(axes) => axes.with(started_horizontal, started_vertical),
            None => match (started_horizontal, started_vertical) {
                (true, true) => Some(Axes::Both),
                (true, false) => Some(Axes::Horizontal),
                (false, true) => Some(Axes::Vertical),
                (false, false) => None,
            },
        };
        if let Some(active) = axes {
            axes = active.without(frame.horizontal.stopped, frame.vertical.stopped);
        }
    }
    let ended = old_axes.is_some() && axes.is_none();
    *state = axes.map_or(State::Idle, |axes| State::Active {
        axes,
        source: if continuous { source } else { old_source },
    });
    let mut scroll = Frame { source, ..frame }.scroll_with_end(false, ended)?;
    scroll.began = continuous && old_axes.is_none() && !ended && axes.is_some();
    Some(Effect::Emit(scroll))
}


impl Axis {
    fn distance(self, source: Source) -> f64 {
        // Winit turns line deltas into 120 Rune points per step and pixel deltas stay
        // in logical points. Wayland's axis sign is opposite winit's.
        if let Some(value) = self.value120 {
            -f64::from(value)
        } else if let Some(steps) = self.discrete {
            -f64::from(steps) * 120.0
        } else {
            let gain = if matches!(source, Source::Finger | Source::Continuous) {
                12.0
            } else {
                1.0
            };
            -self.raw * gain
        }
    }
}

impl Frame {
    #[cfg(test)]
    pub fn scroll(self, finger_active: bool) -> Option<Scroll> {
        self.scroll_with_end(
            finger_active,
            self.horizontal.stopped || self.vertical.stopped,
        )
    }

    fn scroll_with_end(self, finger_active: bool, ended: bool) -> Option<Scroll> {
        let x = self.horizontal.distance(self.source);
        let y = self.vertical.distance(self.source);
        if x == 0.0 && y == 0.0 && !ended {
            return None;
        }
        let finger = matches!(self.source, Source::Finger | Source::Continuous);
        Some(Scroll {
            dx: x as f32,
            dy: y as f32,
            precise: self.horizontal.value120.is_none()
                && self.vertical.value120.is_none()
                && self.horizontal.discrete.is_none()
                && self.vertical.discrete.is_none(),
            began: finger && !finger_active && !ended,
            ended,
        })
    }
}

#[cfg(test)]
#[path = "layer_scroll_tests.rs"]
mod tests;
