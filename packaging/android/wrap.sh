#!/system/bin/sh
# Development-only launch, before the JVM or Rust starts any threads. Android
# provides wrap.sh for debuggable packages on API 27+. Match horizon-guest-stack's
# 128 MiB guest stack plus 8 MiB host headroom; std::thread::spawn honors this.
export RUST_MIN_STACK=142606336
export HORIZON_LAUNCHER_NO_SELF_UPDATE=1
export HORIZON_NO_AUDIO=1
export RUST_BACKTRACE=1

# Android app UIDs encode the Android user in their upper decimal digits.
# Keep each launch's player data inside this development app's own sandbox.
android_user=$(($(id -u) / 100000))
private=/data/user/$android_user/org.stratumgame.android.dev/files
# Normal play persists across launches. Automation explicitly opts into isolated
# disposable homes; its marker always wins over the legacy manual-audio marker.
export STRATUM_CONTROLS_FILE="$private/touch-controls.json"
if [ -f "$private/automated-test" ]; then
    mkdir -p "$private/test-runs" || exit 1
    run=$(mktemp -d "$private/test-runs/run.XXXXXXXX") || exit 1
    export HORIZON_NO_AUDIO=1
    unset STRATUM_RECOVER_WORLDS
    log="$run"
else
    run="$private/play"
    mkdir -p "$run/logs" || exit 1
    log=$(mktemp -d "$run/logs/launch.XXXXXXXX") || exit 1
    unset HORIZON_NO_AUDIO
    export STRATUM_RECOVER_WORLDS=1
fi
export HORIZON_HOME="$run/home"
export HORIZON_LAUNCHER_HOME="$run/launcher"
export XDG_DATA_HOME="$run/data"
export XDG_CONFIG_HOME="$run/config"
export STRATUM_GAME_DIR="$run/game"
mkdir -p "$HORIZON_HOME" "$HORIZON_LAUNCHER_HOME" "$XDG_DATA_HOME" "$XDG_CONFIG_HOME" "$STRATUM_GAME_DIR" || exit 1
cd "$STRATUM_GAME_DIR" || exit 1
exec "$@" >"$log/stdout.log" 2>"$log/stderr.log"
