# Android integration provenance

Base: Horizon `07296d9eff2663e6014420b35d5d2147d7d92d8b`, full history
retained on `stratum/android`. Root `LICENSE`, `LICENSES/` and
`THIRD_PARTY_NOTICES.md` are unchanged. Original overlapping bootstrap files are
backed up under ignored `.local/bootstrap-backup/`; upstream documentation is
retained in `docs/upstream/` and in the base commit.

## First startup slice

The Stratum host, development manifest and managed-command startup adapter are
new LGPL-3.0-or-later integration code. They use the Android activity handoff
pattern reviewed in Starling's `crates/horizon-client/src/android.rs` and
`crates/horizon-android/Cargo.toml` at
`5432370d573642041585c69b40fb045b27133488`. No Starling Java activities, game
tree, native-mod adapters, assets, update services, or signing keys are copied.
The reference license is preserved in `LICENSES/Starling-MIT.txt` as attribution
for the Android integration work.

Horizon already includes Starling-derived controller state in
`crates/horizon-client-render/src/android_gamepad.rs` and `android_runtime.rs`.
Their existing attribution remains intact. Their comments refer to the historic
MIT root license; current Horizon's root license is LGPL. The preserved historic
MIT notice is in `LICENSES/MIT.txt`.

The first slice passes an explicit command description to Horizon's existing
validation and mod-host startup, because Android's process arguments belong to
the JVM. It does not replace Horizon's mod revision or dependency rules. Mod
compatibility and native code loading on Android remain unverified.

`horizon-client-input/src/keybinds.rs` has a new target-specific scancode fallback:
winit 0.30.13 does not expose `PhysicalKeyExtScancode` on Android. Known physical
keys therefore report no raw scancode (0) on this target, while unidentified native
Android codes are preserved. Desktop behavior is unchanged. This is original
integration code, not a copied Starling implementation.

The Android keyboard hook uses winit's logical key where the desktop-only
unmodified-key extension is unavailable. Modifier-sensitive symbols may have no
layout-name mapping; physical key bindings remain separate.

The launch holder and private control modules now include Android. Its Unix peer
credential checks and local-filesystem allowlist use the existing Linux-kernel
paths (including ext4/F2FS); no peer authentication or filesystem trust checks are
removed. Runtime behavior is not yet device-tested. The embedded Android host
rejects desktop holder/entry environment markers because that bootstrap assumes
a single-threaded executable, whereas Android already has JVM threads.

## Android startup stack fix

The first device launch failed in `horizon_guest_stack::prepare_main`: stack
inspection did not include Android, and process exit tore down a live ART runtime.
Android now uses the existing pthread stack query. The development-only
`packaging/android/wrap.sh` sets `RUST_MIN_STACK=142606336` before the JVM/Rust
starts, so android-activity's standard Rust thread receives Horizon's 128 MiB
guest stack plus 8 MiB headroom. No third-party glue source was imported or
patched for this fix.

The wrapper also creates fresh app-private test homes and captures stdout/stderr.
It uses Android's supported debug [wrap-script mechanism](https://developer.android.com/ndk/guides/wrap-script),
which requires API 27+, so Stratum Dev's minimum SDK is now 27 (the reference
Starling/native compilation API remains 26). This wrapper is for disposable
development runs; persistent player storage needs a separate release policy.

# Android menu touch input

Stratum's `horizon-client-input::touch` is an original compatibility adapter,
not an import from Starling. Android sends winit touch events, while the existing
Horizon menus consume cursor and mouse-button events. The adapter translates the
primary finger into a physical-pixel pointer and left-button press/release, while
preserving the existing menu hit testing and GUI scaling. Other fingers cannot
steal a drag. Cancellation, focus loss and suspension release held input.

The adapter wraps all Android event-loop entry paths and forwards the application's
other callbacks. It does not fix graphics-surface lifecycle or provide gameplay
movement/look overlays. Desktop event loops retain their existing input behavior.

## Gameplay controls and physical controllers (0.1.4 development)

Original Stratum code adds a multi-touch gameplay overlay with independent
movement, look and action fingers. Loading and title screens retain pointer
translation; gameplay receives raw touches. Touch has its own held-key source,
and focus loss, suspension and screen changes release its controls.

Physical controller decoding reuses Horizon's existing Starling-derived
`horizon-client-render/src/android_gamepad.rs`. JNI device discovery is isolated
in the Stratum native host; it refreshes device names and removes disconnected
controllers. Horizon's existing controller bindings and menu handling receive
the resulting state. Rumble and Android splitscreen routing remain unsupported.

The crates.io winit 0.30.13 source is vendored under `third_party/winit` with its
Apache-2.0 license intact. A small original Android input filter intercepts pad
events before pointer conversion; details and the exact source revision are in
`third_party/winit/STRATUM.md`. `LICENSES/Winit-Apache-2.0.txt` is bundled in APKs.
Build instructions are in [Android development](android-bootstrap.md).



### 0.1.6 touch and audio work

Reuse Starling's MIT Bedrock texture fetcher at the existing pinned revision,
with exact revision/hash metadata and private resource caching. Checked the
license at that revision before import. Mojang sample assets have separate
Minecraft EULA terms; the code preserves those terms and fetches assets outside
the repository/APK. Rendering uses these same button/pressed/joystick textures
through Horizon's existing GUI atlas, with a compact layout, optional labels,
D-pad/joystick choice and direct hotbar taps. This does not replace Horizon with
a Starling fork or claim full Starling gesture/analogue-control parity.

The existing CPAL 0.16.0 dependency is patched locally for Android output:
low-latency AAudio, supported 48 kHz stereo preference and two-burst queue request.
The source archive's Apache-2.0 license was checked and retained; see its STRATUM.md.
