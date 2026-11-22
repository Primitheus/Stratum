//! Android activity ownership for Stratum's embedded game host.
//!
//! Follows Starling's activity handoff pattern at 5432370d573642041585c69b40fb045b27133488.
//! The game and mod startup remain Horizon's. See docs/android-provenance.md.
use std::sync::OnceLock;
use winit::platform::android::{EventLoopBuilderExtAndroid, activity::AndroidApp};

static APP: OnceLock<AndroidApp> = OnceLock::new();

use crate::controller_ui::Device;
use horizon_client_input::gamepad::PadState;
use horizon_client_render::android_gamepad::{self as pad, Devices};
use std::sync::{LazyLock, Mutex};
static PADS: LazyLock<Mutex<Devices>> = LazyLock::new(Mutex::default);
type Inventory = fn() -> Vec<(i32, Device)>;
static INVENTORY: OnceLock<Inventory> = OnceLock::new();
static DEVICES: Mutex<(Option<std::time::Instant>, Vec<(i32, Device)>)> =
    Mutex::new((None, Vec::new()));

/// The native host owns JNI; gameplay only sees safe device descriptions.
pub fn install_controller_inventory(inventory: Inventory) {
    let _ = INVENTORY.set(inventory);
}
pub(crate) fn clear_controller() {
    PADS.lock().unwrap().clear();
}
pub(crate) fn controller_state() -> (Option<Device>, PadState) {
    let mut devices = DEVICES.lock().unwrap();
    if devices.0.is_none_or(|t| t.elapsed().as_millis() >= 500) {
        if let Some(query) = INVENTORY.get() {
            devices.1 = query();
        }
        devices.0 = Some(std::time::Instant::now());
        PADS.lock()
            .unwrap()
            .retain(&devices.1.iter().map(|(id, _)| *id).collect::<Vec<_>>());
    }
    let mut pads = PADS.lock().unwrap();
    let active = pads.active_id();
    let device = active
        .and_then(|id| devices.1.iter().find(|(d, _)| *d == id))
        .or_else(|| devices.1.first())
        .map(|(_, d)| d.clone());
    (device, pads.poll())
}
fn controller_event(event: &winit::platform::android::activity::input::InputEvent<'_>) -> bool {
    use winit::platform::android::activity::input::{Axis, InputEvent, KeyAction};
    let mut pads = PADS.lock().unwrap();
    match event {
        InputEvent::KeyEvent(key) => {
            let source: u32 = key.source().into();
            let code: u32 = key.key_code().into();
            // Shared navigation keys only belong to a pad when its source says so.
            if pad::button(code).is_some()
                && (matches!(code, 96..=109) || source & (0x400 | 0x10) != 0)
            {
                pads.key(key.device_id(), code, key.action() == KeyAction::Down);
                return true;
            }
        }
        InputEvent::MotionEvent(motion) if motion.source().is_joystick_class() => {
            let p = motion.pointer_at_index(0);
            for axis in [
                Axis::X,
                Axis::Y,
                Axis::Z,
                Axis::Rz,
                Axis::HatX,
                Axis::HatY,
                Axis::Ltrigger,
                Axis::Rtrigger,
                Axis::Brake,
                Axis::Gas,
            ] {
                pads.axis(
                    motion.device_id(),
                    axis.into(),
                    f64::from(p.axis_value(axis)),
                );
            }
            return true;
        }
        _ => {}
    }
    false
}

/// Install the activity before entering Horizon startup. One game loop per process.
pub fn install(app: AndroidApp) -> Result<(), AndroidApp> {
    APP.set(app)
}

pub(crate) fn event_loop() -> crate::Result<winit::event_loop::EventLoop<()>> {
    let app = APP
        .get()
        .ok_or("Android activity was not installed before startup")?;
    Ok(winit::event_loop::EventLoop::builder()
        .with_android_app(app.clone())
        .with_input_filter(controller_event)
        .build()?)
}
