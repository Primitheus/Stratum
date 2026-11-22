//! Android surface/focus state shared with the audio callback. Desktop never feeds this.
use std::sync::atomic::{AtomicU8, Ordering};
const SURFACE: u8 = 1;
const FOCUS: u8 = 2;
#[derive(Default)]
pub struct Lifecycle(AtomicU8);
impl Lifecycle {
    pub const fn new() -> Self {
        Self(AtomicU8::new(0))
    }
    pub fn set_surface(&self, available: bool) {
        self.set(SURFACE, available);
    }
    pub fn set_focused(&self, focused: bool) {
        self.set(FOCUS, focused);
    }
    fn set(&self, bit: u8, on: bool) {
        if on {
            self.0.fetch_or(bit, Ordering::SeqCst);
        } else {
            self.0.fetch_and(!bit, Ordering::SeqCst);
        }
    }
    pub fn has_surface(&self) -> bool {
        self.0.load(Ordering::SeqCst) & SURFACE != 0
    }
    pub fn audible(&self) -> bool {
        self.0.load(Ordering::SeqCst) == SURFACE | FOCUS
    }
}
pub static STATE: Lifecycle = Lifecycle::new();
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focus_and_surface_are_both_required() {
        let s = Lifecycle::new();
        assert!(!s.audible());
        s.set_focused(true);
        assert!(!s.audible());
        s.set_surface(true);
        assert!(s.audible());
        s.set_focused(false);
        assert!(s.has_surface() && !s.audible());
        s.set_focused(true);
        assert!(s.audible());
        s.set_surface(false);
        assert!(!s.audible());
    }
    #[test]
    fn repeated_lifecycle_events_and_resume_order() {
        let s = Lifecycle::new();
        for _ in 0..5 {
            s.set_surface(true);
            s.set_surface(true);
            assert!(!s.audible());
            s.set_focused(true);
            assert!(s.audible());
            s.set_surface(false);
            s.set_focused(false);
            s.set_surface(false);
            assert!(!s.has_surface() && !s.audible());
        }
    }
}
