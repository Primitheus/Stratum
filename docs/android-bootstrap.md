# Android development

Stratum embeds Horizon in an Android native activity. Its Android integration is
implemented in `crates/stratum-android`, with packaging in `packaging/android`.
See [upstream-lock.json](../upstream-lock.json) for pinned source and toolchain versions.

## Prerequisites

- Python 3 and a JDK with `javac`, `java` and `keytool` available.
- Rust 1.97.1 with the `aarch64-linux-android` target.
- Android SDK platform and build tools matching the pinned configuration.
- Android NDK 28.2.13676358.

Set `ANDROID_HOME` or `ANDROID_SDK_ROOT` to the Android SDK. Set `JAVA_HOME` if the
JDK is not on PATH. An external NDK can be selected with `ANDROID_NDK_HOME`.
Workspace-local toolchains under `.local/` are also supported by the build scripts.

```sh
python tools/android_doctor.py
python tools/android_build.py --check
python packaging/android/build_apk.py
```

The APK is generated at `target/android/stratum-dev-arm64-v8a.apk`. Packaging creates
a local development signing key under `target/android/`; it does not install or
publish the APK. Build artifacts, keys and downloaded game resources are excluded
from version control. Resource preparation is separate from APK packaging.

## Runtime data

The development application ID is `org.stratumgame.android.dev`. Normal play uses
app-private persistent data under `files/play`. The debug wrapper configures the
native thread stack and captures logs within the application's private storage.

For automated tests, create the app-private `files/automated-test` marker before
launching. This selects fresh disposable data under `files/test-runs` and forces
audio off. Remove the marker before returning to normal play. Automated desktop
runs must also use fresh `HORIZON_HOME`, `HORIZON_LAUNCHER_HOME`, `XDG_DATA_HOME` and
`XDG_CONFIG_HOME`, with `HORIZON_LAUNCHER_NO_SELF_UPDATE=1` and `HORIZON_NO_AUDIO=1`.

Keep device-specific results, identifiers and network addresses in ignored local
notes. Public documentation should describe reproducible behavior and requirements.

## Portable workspace

Scripts resolve paths from the repository root or their own location. Configure
external tools through environment variables. Recreate caches after moving the
workspace; do not depend on absolute local paths, linked worktrees or Git alternates.
