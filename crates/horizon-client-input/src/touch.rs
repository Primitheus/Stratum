//! Single-finger pointer compatibility for Android's existing desktop menus.
//! Coordinates stay in physical pixels; the screens apply their own GUI scale.
//! This is not the multi-touch movement/look control layer.
use std::collections::HashMap;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{
        DeviceEvent, DeviceId, ElementState, MouseButton, StartCause, Touch, TouchPhase,
        WindowEvent,
    },
    event_loop::ActiveEventLoop,
    window::WindowId,
};

#[derive(Default)]
pub struct Pointer {
    finger: Option<(DeviceId, u64)>,
}

impl Pointer {
    pub fn touch(&mut self, touch: Touch) -> Vec<WindowEvent> {
        let finger = (touch.device_id, touch.id);
        if touch.phase == TouchPhase::Started && self.finger.is_none() {
            self.finger = Some(finger);
        } else if self.finger != Some(finger) {
            // Another finger must not move the pointer or release a held drag.
            return Vec::new();
        }
        if touch.phase == TouchPhase::Cancelled {
            return self.cancel();
        }
        let mut events = vec![WindowEvent::CursorMoved {
            device_id: touch.device_id,
            position: touch.location,
        }];
        let state = match touch.phase {
            TouchPhase::Started => Some(ElementState::Pressed),
            TouchPhase::Ended => {
                self.finger = None;
                Some(ElementState::Released)
            }
            _ => None,
        };
        if let Some(state) = state {
            events.push(WindowEvent::MouseInput {
                device_id: touch.device_id,
                state,
                button: MouseButton::Left,
            });
        }
        events
    }

    pub fn cancel(&mut self) -> Vec<WindowEvent> {
        let Some((device_id, _)) = self.finger.take() else {
            return Vec::new();
        };
        // Release away from widgets so release-activated buttons do not click.
        vec![
            WindowEvent::CursorMoved {
                device_id,
                position: PhysicalPosition::new(-1.0, -1.0),
            },
            WindowEvent::MouseInput {
                device_id,
                state: ElementState::Released,
                button: MouseButton::Left,
            },
        ]
    }
}

/// Wrap the complete application so pointer state survives loading-to-game handoff.
/// Only Android installs this adapter; physical mouse events pass through unchanged.
pub struct PointerApp<'a, A> {
    app: &'a mut A,
    pointers: HashMap<WindowId, Pointer>,
}

impl<'a, A> PointerApp<'a, A> {
    pub fn new(app: &'a mut A) -> Self {
        Self {
            app,
            pointers: HashMap::new(),
        }
    }
}

pub trait RawTouch {
    fn raw_touch(&self) -> bool {
        false
    }
}

impl<A: ApplicationHandler + RawTouch> ApplicationHandler for PointerApp<'_, A> {
    fn new_events(&mut self, e: &ActiveEventLoop, cause: StartCause) {
        self.app.new_events(e, cause);
    }
    fn resumed(&mut self, e: &ActiveEventLoop) {
        self.app.resumed(e);
    }
    fn user_event(&mut self, e: &ActiveEventLoop, event: ()) {
        self.app.user_event(e, event);
    }
    fn window_event(&mut self, e: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self.app.raw_touch() {
            self.pointers.entry(id).or_default();
            self.app.window_event(e, id, event);
            return;
        }
        if let WindowEvent::Touch(touch) = event {
            for event in self.pointers.entry(id).or_default().touch(touch) {
                self.app.window_event(e, id, event);
            }
            return;
        }
        if matches!(event, WindowEvent::Focused(false) | WindowEvent::Destroyed)
            && let Some(mut pointer) = self.pointers.remove(&id)
        {
            for event in pointer.cancel() {
                self.app.window_event(e, id, event);
            }
        }
        self.app.window_event(e, id, event);
    }
    fn device_event(&mut self, e: &ActiveEventLoop, id: DeviceId, event: DeviceEvent) {
        self.app.device_event(e, id, event);
    }
    fn about_to_wait(&mut self, e: &ActiveEventLoop) {
        self.app.about_to_wait(e);
    }
    fn suspended(&mut self, e: &ActiveEventLoop) {
        for (id, mut pointer) in self.pointers.drain() {
            if self.app.raw_touch() {
                self.app.window_event(e, id, WindowEvent::Focused(false));
            }
            for event in pointer.cancel() {
                self.app.window_event(e, id, event);
            }
        }
        self.app.suspended(e);
    }
    fn exiting(&mut self, e: &ActiveEventLoop) {
        self.app.exiting(e);
    }
    fn memory_warning(&mut self, e: &ActiveEventLoop) {
        self.app.memory_warning(e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub fn touch(id: u64, phase: TouchPhase, x: f64) -> Touch {
        Touch {
            device_id: DeviceId::dummy(),
            phase,
            location: PhysicalPosition::new(x, 75.0),
            force: None,
            id,
        }
    }
    #[test]
    fn tap_positions_pointer_before_press_and_release() {
        let mut p = Pointer::default();
        for (phase, state) in [
            (TouchPhase::Started, ElementState::Pressed),
            (TouchPhase::Ended, ElementState::Released),
        ] {
            let events = p.touch(touch(1, phase, 150.0));
            assert!(
                matches!(events.as_slice(), [WindowEvent::CursorMoved { position, .. }, WindowEvent::MouseInput { state: actual, button: MouseButton::Left, .. }] if *position == PhysicalPosition::new(150.0,75.0) && *actual == state)
            );
        }
        assert!(p.finger.is_none());
    }
    #[test]
    fn second_finger_cannot_steal_or_release_drag() {
        let mut p = Pointer::default();
        p.touch(touch(1, TouchPhase::Started, 100.0));
        for phase in [
            TouchPhase::Started,
            TouchPhase::Moved,
            TouchPhase::Ended,
            TouchPhase::Cancelled,
        ] {
            assert!(p.touch(touch(2, phase, 400.0)).is_empty());
        }
        assert!(
            matches!(p.touch(touch(1, TouchPhase::Moved, 200.0)).as_slice(), [WindowEvent::CursorMoved { position, .. }] if position.x == 200.0)
        );
        assert_eq!(p.touch(touch(1, TouchPhase::Ended, 200.0)).len(), 2);
    }
    #[test]
    fn cancellation_releases_outside_widgets_and_accepts_next_tap() {
        let mut p = Pointer::default();
        p.touch(touch(1, TouchPhase::Started, 100.0));
        let events = p.touch(touch(1, TouchPhase::Cancelled, 100.0));
        assert!(
            matches!(events.as_slice(), [WindowEvent::CursorMoved { position, .. }, WindowEvent::MouseInput { state: ElementState::Released, .. }] if position.x < 0.0 && position.y < 0.0)
        );
        assert!(p.touch(touch(1, TouchPhase::Ended, 100.0)).is_empty());
        assert!(p.cancel().is_empty());
        assert_eq!(p.touch(touch(3, TouchPhase::Started, 300.0)).len(), 2);
    }
}
