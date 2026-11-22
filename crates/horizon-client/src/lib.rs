//! Gameplay and startup, above the reusable client library.
// No `#![recursion_limit]` or other inner attribute horizon-mixins doesn't know here: the
// launcher's horizon-mixins then rejects every native mod targeting this crate (#8169).
#![forbid(unsafe_code)]
extern crate self as horizon_client;
pub use horizon_client_lib::carrier_store;
pub use horizon_client_lib::*;
#[doc(hidden)]
pub mod entry;

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "android")]
pub mod bedrock_touch;

/// All interactive startup paths must use the activity supplied by the Android host.
pub(crate) fn window_event_loop() -> Result<winit::event_loop::EventLoop<()>> {
    #[cfg(target_os = "android")]
    {
        android::event_loop()
    }
    #[cfg(not(target_os = "android"))]
    {
        Ok(winit::event_loop::EventLoop::new()?)
    }
}

pub(crate) fn run_window_app(
    event_loop: winit::event_loop::EventLoop<()>,
    app: &mut (impl winit::application::ApplicationHandler + horizon_client_input::touch::RawTouch),
) -> std::result::Result<(), winit::error::EventLoopError> {
    #[cfg(target_os = "android")]
    return event_loop.run_app(&mut horizon_client_input::touch::PointerApp::new(app));
    #[cfg(not(target_os = "android"))]
    event_loop.run_app(app)
}

// The game's allocator lives in the library, not the executable: the prebuilt game (arismoko/horizon#589) is this crate as a
// dylib, and the allocator must be the one its code and the mods' executable share. The Windows and macOS archives
// build with `system-allocator`: their loaders bind each library to std's own allocator, which everything then
// shares (arismoko/horizon#2394, arismoko/horizon#3877).
#[cfg(not(feature = "system-allocator"))]
#[global_allocator]
static ALLOCATOR: horizon_threads::GameAllocator = horizon_threads::GAME_ALLOCATOR;
pub mod game;
pub mod orb_limits;
