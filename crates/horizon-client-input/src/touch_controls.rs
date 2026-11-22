//! Multi-touch ownership and layout, in physical pixels. No gameplay or renderer dependency.
use std::collections::HashMap;
use winit::event::{DeviceId, Touch, TouchPhase};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Forward,
    Backward,
    Left,
    Right,
    Jump,
    Sneak,
    Sprint,
    Attack,
    Use,
    Inventory,
    Pause,
    Drop,
    Previous,
    Next,
    Customize,
    Slot(u8),
}
#[derive(Clone, Copy)]
pub struct Button {
    pub action: Action,
    pub label: &'static str,
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}
pub fn buttons([w, h]: [f32; 2], playing: bool) -> Vec<Button> {
    let s = (w / 1000.0).min(h / 500.0);
    let mut out = vec![Button {
        action: Action::Pause,
        label: if playing { "Pause" } else { "Back" },
        x: w - 150.0 * s,
        y: 45.0 * s,
        radius: 32.0 * s,
    }];
    if playing {
        for (action, label, x, y) in [
            (Action::Jump, "Jump", w - 55.0 * s, h - 135.0 * s),
            (Action::Sneak, "Sneak", w - 140.0 * s, h - 135.0 * s),
            (Action::Attack, "Mine", w - 140.0 * s, h - 225.0 * s),
            (Action::Use, "Use", w - 55.0 * s, h - 225.0 * s),
            (Action::Sprint, "Run", 115.0 * s, h - 260.0 * s),
            (Action::Inventory, "Bag", w - 55.0 * s, 45.0 * s),
            (Action::Drop, "Drop", w - 245.0 * s, 45.0 * s),
            (Action::Previous, "<", w / 2.0 - 215.0 * s, h - 35.0 * s),
            (Action::Next, ">", w / 2.0 + 215.0 * s, h - 35.0 * s),
        ] {
            out.push(Button {
                action,
                label,
                x,
                y,
                radius: 32.0 * s,
            });
        }
    }
    out
}
pub fn stick([w, h]: [f32; 2]) -> (f32, f32, f32) {
    let s = (w / 1000.0).min(h / 500.0);
    (115.0 * s, h - 150.0 * s, 78.0 * s)
}
/// The same geometry is used for drawing and touch hit testing, including user edits.
pub struct Layout {
    pub buttons: Vec<Button>,
    pub stick: (f32, f32, f32),
    pub auto_sprint: bool,
}
impl Layout {
    pub fn new(size: [f32; 2], playing: bool) -> Self {
        Self {
            buttons: buttons(size, playing),
            stick: stick(size),
            auto_sprint: false,
        }
    }
}
#[derive(Clone, Copy)]
enum Role {
    Stick,
    Look,
    Button(Action),
}
struct Contact {
    role: Role,
    x: f32,
    y: f32,
}
#[derive(Default)]
pub struct Controls {
    contacts: HashMap<(DeviceId, u64), Contact>,
    held: Vec<Action>,
    stick_sprinting: bool,
}
#[derive(Default)]
pub struct Changes {
    pub edges: Vec<(Action, bool)>,
    pub look: [f64; 2],
}
impl Controls {
    pub fn stick_offset(&self, layout: &Layout) -> [f32; 2] {
        let Some(c) = self
            .contacts
            .values()
            .find(|c| matches!(c.role, Role::Stick))
        else {
            return [0., 0.];
        };
        let (x, y, r) = layout.stick;
        let dx = (c.x - x) / r.max(1.);
        let dy = (c.y - y) / r.max(1.);
        let length = dx.hypot(dy).max(1.);
        [dx / length, dy / length]
    }
    pub fn held(&self, a: Action) -> bool {
        self.held.contains(&a)
    }
    pub fn cancel(&mut self) -> Changes {
        self.contacts.clear();
        self.stick_sprinting = false;
        Changes {
            edges: self.held.drain(..).map(|a| (a, false)).collect(),
            ..Default::default()
        }
    }
    pub fn touch(&mut self, t: Touch, size: [f32; 2], playing: bool) -> Changes {
        self.touch_layout(t, size, playing, &Layout::new(size, playing))
    }
    pub fn touch_layout(
        &mut self,
        t: Touch,
        _size: [f32; 2],
        playing: bool,
        layout: &Layout,
    ) -> Changes {
        let id = (t.device_id, t.id);
        let x = t.location.x as f32;
        let y = t.location.y as f32;
        let mut changes = Changes::default();
        let (sx, sy, r) = layout.stick;
        if t.phase == TouchPhase::Started {
            let role = if let Some(b) = layout
                .buttons
                .iter()
                .find(|b| (x - b.x).abs() <= b.radius && (y - b.y).abs() <= b.radius)
            {
                Role::Button(b.action)
            } else if playing
                && (x - sx).abs() < r * 1.4
                && (y - sy).abs() < r * 1.4
                && !self
                    .contacts
                    .values()
                    .any(|c| matches!(c.role, Role::Stick))
            {
                Role::Stick
            } else if playing && !self.contacts.values().any(|c| matches!(c.role, Role::Look)) {
                Role::Look
            } else {
                return changes;
            };
            self.contacts.insert(id, Contact { role, x, y });
        } else if let Some(c) = self.contacts.get_mut(&id) {
            if t.phase == TouchPhase::Moved && matches!(c.role, Role::Look) {
                changes.look = [f64::from(x - c.x), f64::from(y - c.y)];
            }
            c.x = x;
            c.y = y;
            if matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                self.contacts.remove(&id);
            }
        }
        let mut held = Vec::new();
        let sprint_threshold = if self.stick_sprinting { 0.60 } else { 0.85 };
        self.stick_sprinting = layout.auto_sprint
            && self.contacts.values().any(|c| {
                matches!(c.role, Role::Stick)
                    && c.y < sy - r * sprint_threshold
                    && (c.x - sx).abs() < (sy - c.y) * 1.25
            });
        for c in self.contacts.values() {
            match c.role {
                Role::Button(a) => {
                    if !held.contains(&a) {
                        held.push(a);
                    }
                }
                Role::Stick => {
                    for (a, on) in [
                        (Action::Forward, c.y < sy - r * 0.2),
                        (Action::Backward, c.y > sy + r * 0.2),
                        (Action::Left, c.x < sx - r * 0.2),
                        (Action::Right, c.x > sx + r * 0.2),
                    ] {
                        if on {
                            held.push(a);
                        }
                    }
                }
                Role::Look => {}
            }
        }
        if self.stick_sprinting && !held.contains(&Action::Sprint) {
            held.push(Action::Sprint);
        }
        changes.edges.extend(
            self.held
                .iter()
                .filter(|a| !held.contains(a))
                .map(|a| (*a, false)),
        );
        changes.edges.extend(
            held.iter()
                .filter(|a| !self.held.contains(a))
                .map(|a| (*a, true)),
        );
        self.held = held;
        changes
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn event(id: u64, phase: TouchPhase, x: f64, y: f64) -> Touch {
        Touch {
            device_id: DeviceId::dummy(),
            id,
            phase,
            location: winit::dpi::PhysicalPosition::new(x, y),
            force: None,
        }
    }
    #[test]
    fn move_look_jump_are_independent() {
        let mut c = Controls::default();
        let size = [1000., 500.];
        assert_eq!(
            c.touch(event(1, TouchPhase::Started, 115., 290.), size, true)
                .edges,
            vec![(Action::Forward, true)]
        );
        c.touch(event(2, TouchPhase::Started, 600., 200.), size, true);
        assert_eq!(
            c.touch(event(2, TouchPhase::Moved, 620., 210.), size, true)
                .look,
            [20., 10.]
        );
        c.touch(event(3, TouchPhase::Started, 945., 365.), size, true);
        assert!(c.held(Action::Jump) && c.held(Action::Forward));
        c.touch(event(3, TouchPhase::Ended, 945., 365.), size, true);
        assert!(!c.held(Action::Jump) && c.held(Action::Forward));
        c.cancel();
        assert!(!c.held(Action::Forward));
        assert!(
            c.touch(event(1, TouchPhase::Moved, 115., 290.), size, true)
                .edges
                .is_empty()
        );
    }
    #[test]
    fn two_fingers_on_same_button_release_only_after_both() {
        let mut c = Controls::default();
        let size = [1000., 500.];
        for id in [1, 2] {
            c.touch(event(id, TouchPhase::Started, 945., 365.), size, true);
        }
        assert!(
            c.touch(event(1, TouchPhase::Cancelled, 945., 365.), size, true)
                .edges
                .is_empty()
        );
        assert_eq!(
            c.touch(event(2, TouchPhase::Ended, 945., 365.), size, true)
                .edges,
            vec![(Action::Jump, false)]
        );
    }
    #[test]
    fn custom_layout_moves_hit_targets_and_allows_left_hand_look() {
        let mut c = Controls::default();
        let size = [1000., 500.];
        let layout = Layout {
            stick: (850., 350., 60.),
            auto_sprint: false,
            buttons: vec![Button {
                action: Action::Jump,
                label: "Jump",
                x: 100.,
                y: 400.,
                radius: 30.,
            }],
        };
        c.touch_layout(
            event(1, TouchPhase::Started, 850., 310.),
            size,
            true,
            &layout,
        );
        assert!(c.held(Action::Forward));
        c.touch_layout(
            event(2, TouchPhase::Started, 200., 200.),
            size,
            true,
            &layout,
        );
        let motion = c.touch_layout(event(2, TouchPhase::Moved, 230., 210.), size, true, &layout);
        assert_eq!(motion.look, [30., 10.]);
        c.touch_layout(
            event(3, TouchPhase::Started, 100., 400.),
            size,
            true,
            &layout,
        );
        assert!(c.held(Action::Jump));
        c.touch_layout(event(3, TouchPhase::Ended, 100., 400.), size, true, &layout);
        assert!(!c.held(Action::Jump));
        assert!(c.held(Action::Forward));
        assert_eq!(c.cancel().edges, vec![(Action::Forward, false)]);
    }
    #[test]
    fn hotbar_contact_does_not_take_the_look_finger() {
        let mut c = Controls::default();
        let size = [1000., 500.];
        let layout = Layout {
            stick: (100., 350., 60.),
            auto_sprint: false,
            buttons: vec![Button {
                action: Action::Slot(3),
                label: "",
                x: 500.,
                y: 480.,
                radius: 20.,
            }],
        };
        c.touch_layout(
            event(1, TouchPhase::Started, 700., 200.),
            size,
            true,
            &layout,
        );
        assert_eq!(
            c.touch_layout(
                event(2, TouchPhase::Started, 500., 480.),
                size,
                true,
                &layout
            )
            .edges,
            vec![(Action::Slot(3), true)]
        );
        assert_eq!(
            c.touch_layout(event(1, TouchPhase::Moved, 730., 220.), size, true, &layout)
                .look,
            [30., 20.]
        );
        assert_eq!(
            c.touch_layout(event(2, TouchPhase::Ended, 500., 480.), size, true, &layout)
                .edges,
            vec![(Action::Slot(3), false)]
        );
    }
}

#[cfg(test)]
mod sprint_tests {
    use super::*;
    fn touch(
        c: &mut Controls,
        layout: &Layout,
        phase: TouchPhase,
        id: u64,
        x: f64,
        y: f64,
    ) -> Changes {
        c.touch_layout(
            Touch {
                device_id: DeviceId::dummy(),
                id,
                phase,
                location: winit::dpi::PhysicalPosition::new(x, y),
                force: None,
            },
            [1000., 500.],
            true,
            layout,
        )
    }
    #[test]
    fn joystick_walk_sprint_hysteresis_and_cancel() {
        let mut c = Controls::default();
        let mut l = Layout::new([1000., 500.], true);
        l.auto_sprint = true;
        l.buttons.clear();
        l.stick = (150., 350., 100.);
        touch(&mut c, &l, TouchPhase::Started, 1, 150., 310.);
        assert!(c.held(Action::Forward) && !c.held(Action::Sprint));
        touch(&mut c, &l, TouchPhase::Moved, 1, 150., 260.);
        assert!(c.held(Action::Sprint));
        touch(&mut c, &l, TouchPhase::Moved, 1, 150., 280.);
        assert!(c.held(Action::Sprint));
        touch(&mut c, &l, TouchPhase::Moved, 1, 150., 300.);
        assert!(c.held(Action::Forward) && !c.held(Action::Sprint));
        touch(&mut c, &l, TouchPhase::Moved, 1, 260., 330.);
        assert!(!c.held(Action::Sprint));
        touch(&mut c, &l, TouchPhase::Moved, 1, 200., 255.);
        assert!(c.held(Action::Sprint));
        c.cancel();
        assert!(!c.held(Action::Sprint));
        touch(&mut c, &l, TouchPhase::Started, 1, 150., 280.);
        assert!(!c.held(Action::Sprint));
        l.auto_sprint = false;
        touch(&mut c, &l, TouchPhase::Moved, 1, 150., 250.);
        assert!(!c.held(Action::Sprint));
    }
    #[test]
    fn joystick_release_does_not_release_manual_sprint() {
        let mut c = Controls::default();
        let mut l = Layout::new([1000., 500.], true);
        l.auto_sprint = true;
        l.stick = (150., 350., 100.);
        touch(&mut c, &l, TouchPhase::Started, 1, 150., 350.);
        touch(&mut c, &l, TouchPhase::Moved, 1, 150., 250.);
        touch(&mut c, &l, TouchPhase::Started, 2, 115., 240.);
        assert!(
            touch(&mut c, &l, TouchPhase::Ended, 1, 150., 250.)
                .edges
                .iter()
                .all(|e| *e != (Action::Sprint, false))
        );
        assert!(c.held(Action::Sprint));
        touch(&mut c, &l, TouchPhase::Ended, 2, 115., 240.);
        assert!(!c.held(Action::Sprint));
    }
}
