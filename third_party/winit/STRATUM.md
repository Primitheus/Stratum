# Stratum Android input hook

Vendored from the crates.io winit 0.30.13 archive (upstream revision
`e9809ef54b18499bb4f2cac945719ecc2a61061b`), Apache-2.0. The upstream LICENSE
and source notices are preserved. The license is also packaged in the APK.

Stratum changes `src/platform/android.rs` and
`src/platform_impl/android/mod.rs` to add an optional native input filter.
It consumes Android controller events before winit translates joystick motion
into touchscreen events. All other input uses the upstream handling, and the
default filter is absent. Desktop backends are unchanged.

This is an original integration hook; the entire Starling winit fork is not
imported. Controller decoding uses Horizon's existing Starling-derived
`horizon-client-render/src/android_gamepad.rs` with its existing attribution.
