use super::*;

#[test]
    fn value120_is_one_hundred_twentieths_of_rune_line_step() {
        let event = Frame {
            vertical: Axis {
                raw: 17.0,
                discrete: Some(1),
                value120: Some(-30),
                stopped: false,
            },
            ..Frame::default()
        }
        .scroll(false)
        .unwrap();
        assert_eq!(event.dy, 30.0);
        assert!(!event.precise);
    }

#[test]
    fn discrete_step_uses_runes_linux_120_point_line_step() {
        let event = Frame {
            horizontal: Axis {
                discrete: Some(-2),
                ..Axis::default()
            },
            source: Source::Wheel,
            ..Frame::default()
        }
        .scroll(false)
        .unwrap();
        assert_eq!((event.dx, event.dy), (240.0, 0.0));
    }

#[test]
    fn finger_axis_uses_the_same_points_per_unit_as_winit() {
        let event = Frame {
            vertical: Axis {
                raw: -2.5,
                ..Axis::default()
            },
            source: Source::Finger,
            ..Frame::default()
        }
        .scroll(false)
        .unwrap();
        assert_eq!(event.dy, 30.0);
        assert!(event.precise);
        assert!(event.began);
    }

#[test]
    fn continuous_source_uses_winit_axis_gain_and_wheel_tilt_does_not() {
        let continuous = Frame {
            vertical: Axis {
                raw: 1.0,
                ..Axis::default()
            },
            source: Source::Continuous,
            ..Frame::default()
        }
        .scroll(false)
        .unwrap();
        let tilted_wheel = Frame {
            vertical: Axis {
                raw: 1.0,
                ..Axis::default()
            },
            source: Source::WheelTilt,
            ..Frame::default()
        }
        .scroll(false)
        .unwrap();
        assert_eq!((continuous.dy, tilted_wheel.dy), (-12.0, -1.0));
    }

#[test]
    fn finger_frame_after_start_is_moved_not_began() {
        let event = Frame {
            vertical: Axis {
                raw: 1.0,
                ..Axis::default()
            },
            source: Source::Finger,
            ..Frame::default()
        }
        .scroll(true)
        .unwrap();
        assert!(!event.began);
        assert!(!event.ended);
    }

#[test]
    fn stop_is_axis_specific_and_emits_end_even_without_delta() {
        let event = Frame {
            horizontal: Axis {
                stopped: true,
                ..Axis::default()
            },
            ..Frame::default()
        }
        .scroll(true)
        .unwrap();
        assert_eq!((event.dx, event.dy), (0.0, 0.0));
        assert!(event.ended);
    }

#[test]
    fn empty_frame_emits_nothing() {
        assert_eq!(Frame::default().scroll(false), None);
    }

#[test]
    fn stopping_one_axis_does_not_end_a_diagonal_gesture() {
        let mut gesture = State::default();
        let Effect::Emit(start) = step(&mut gesture, Event::Frame(Frame {
                horizontal: Axis {
                    raw: 1.0,
                    ..Axis::default()
                },
                vertical: Axis {
                    raw: 2.0,
                    ..Axis::default()
                },
                source: Source::Finger,
            })).unwrap();
        assert!(start.began);

        let horizontal_stop = step(&mut gesture, Event::Frame(Frame {
            horizontal: Axis {
                stopped: true,
                ..Axis::default()
            },
            source: Source::Unknown,
            ..Frame::default()
        }));
        assert_eq!(horizontal_stop, None);

        let Some(Effect::Emit(vertical_move)) = step(&mut gesture, Event::Frame(Frame {
                vertical: Axis {
                    raw: 1.0,
                    ..Axis::default()
                },
                source: Source::Unknown,
                ..Frame::default()
            })) else { panic!("vertical movement emits"); };
        assert_eq!(vertical_move.dy, -12.0);
        assert!(!vertical_move.ended);

        let Some(Effect::Emit(vertical_stop)) = step(&mut gesture, Event::Frame(Frame {
                vertical: Axis {
                    stopped: true,
                    ..Axis::default()
                },
                source: Source::Unknown,
                ..Frame::default()
            })) else { panic!("last axis stop emits end"); };
        assert!(vertical_stop.ended);
    }

#[test]
fn source_is_carried_until_final_axis_stops_then_reset_clears_mode() {
    let mut state = State::Idle;
    let Some(Effect::Emit(start)) = step(&mut state, Event::Frame(Frame {
        vertical: Axis { raw: 1.0, ..Axis::default() },
        source: Source::Finger,
        ..Frame::default()
    })) else { panic!("gesture begins"); };
    assert!(start.began);
    assert_eq!(state, State::Active { axes: Axes::Vertical, source: Source::Finger });

    let Some(Effect::Emit(moved)) = step(&mut state, Event::Frame(Frame {
        vertical: Axis { raw: 1.0, ..Axis::default() },
        ..Frame::default()
    })) else { panic!("unknown source continues active gesture"); };
    assert_eq!(moved.dy, -12.0);

    assert_eq!(step(&mut state, Event::Reset), None);
    assert_eq!(state, State::Idle);
    let Some(Effect::Emit(unknown)) = step(&mut state, Event::Frame(Frame {
        vertical: Axis { raw: 1.0, ..Axis::default() },
        ..Frame::default()
    })) else { panic!("standalone unknown frame emits"); };
    assert_eq!(unknown.dy, -1.0);
    assert!(!unknown.began);
}
