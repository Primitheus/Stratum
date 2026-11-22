# Stratum

Stratum is an Android client based on [Horizon](https://github.com/horizon-rust/horizon),
a Rust reimplementation of Minecraft: Java Edition. It brings Horizon's gameplay
and rendering to Android with controls designed for touchscreens and gamepads.

## Features

- **Touchscreen controls** — movement, camera control, combat, building and hotbar selection.
- **Customizable HUD** — adjustable button placement, sizing and visibility, with joystick and D-pad layouts.
- **Controller support** — physical gamepad input for gameplay and menus.
- **Shaders** — Horizon shader support, with compatibility and performance depending on the pack and hardware.
- **Persistent worlds** — local world creation, saving and loading.
- **Android audio** — low-latency audio output and background playback handling.

Stratum is under active development. Compatibility varies across Android devices
and shader packs.

## Building

The current Android target is ARM64, with Android 8.1 (API 27) or newer and Vulkan
support. Toolchain versions are pinned in [upstream-lock.json](upstream-lock.json).

See the [Android development guide](docs/android-bootstrap.md) for prerequisites,
build commands and test configuration.

## Project foundations

Stratum retains Horizon's source history and uses selected Android integration
from [Starling](https://github.com/starling-client/starling), with attribution
preserved. See [integration provenance](docs/android-provenance.md).

Mod integration is in development and targets the
[Horizon mod ecosystem](https://playhorizon.dev/mods).

## Licensing

Stratum's covered game code is licensed under **LGPL-3.0-or-later**. Third-party
components retain their respective licenses; see [LICENSE](LICENSE),
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and [LICENSES](LICENSES).

Minecraft resources and third-party shader packs are not included. Minecraft is
owned by Mojang Studios and Microsoft. Stratum is not affiliated with or endorsed
by Mojang Studios or Microsoft.
