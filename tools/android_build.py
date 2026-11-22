#!/usr/bin/env python3
"""Check or build Stratum's ARM64 native library using the pinned Android NDK."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def environment():
    pins = json.loads((ROOT / "upstream-lock.json").read_text())["android_reference"]
    env = os.environ.copy()
    sdk = Path(env.get("ANDROID_HOME") or env.get("ANDROID_SDK_ROOT") or ROOT / ".local/android-sdk")
    default_ndk = sdk / "ndk" / pins["ndk"]
    if not default_ndk.is_dir():
        default_ndk = ROOT / ".local/android-sdk/ndk" / pins["ndk"]
    ndk = Path(env.get("ANDROID_NDK_HOME") or default_ndk)
    properties = ndk / "source.properties"
    if not properties.is_file() or f"Pkg.Revision = {pins['ndk']}" not in properties.read_text():
        raise ValueError(f"Set ANDROID_NDK_HOME to NDK {pins['ndk']}, or install it under .local/android-sdk/ndk")
    host = {"win32": "windows-x86_64", "linux": "linux-x86_64", "darwin": "darwin-x86_64"}.get(sys.platform)
    if host is None:
        raise ValueError(f"Unsupported NDK host: {sys.platform}")
    compiler_bin = ndk / "toolchains/llvm/prebuilt" / host / "bin"
    suffix = ".exe" if os.name == "nt" else ""
    for tool in ["clang", "clang++", "llvm-ar"]:
        if not (compiler_bin / (tool + suffix)).is_file():
            raise ValueError(f"Missing NDK tool: {tool}")

    # Optional workspace-local Rust installation; external tools still work via PATH.
    local_rustup = ROOT / ".local/rustup"
    toolchain = local_rustup / "toolchains" / ("1.97.1-" + {
        "win32": "x86_64-pc-windows-msvc", "linux": "x86_64-unknown-linux-gnu",
        "darwin": "aarch64-apple-darwin",
    }[sys.platform]) / "bin"
    if "RUSTUP_HOME" not in env and (toolchain / ("cargo" + suffix)).is_file():
        env["RUSTUP_HOME"] = str(local_rustup)
        env["PATH"] = str(toolchain) + os.pathsep + env.get("PATH", "")
    env.setdefault("CARGO_HOME", str(ROOT / ".local/cargo"))
    env["CARGO_TARGET_DIR"] = str(ROOT / "target")
    # Generated game/mod interfaces make incremental caches very large. Keep the
    # first Android builds small; users can opt in explicitly when space permits.
    env.setdefault("CARGO_INCREMENTAL", "0")
    env["ANDROID_NDK_HOME"] = str(ndk)
    target = f"{pins['rust_target']}{pins['min_sdk']}"
    env["CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER"] = str(compiler_bin / ("clang" + suffix))
    env["CC_aarch64_linux_android"] = str(compiler_bin / ("clang" + suffix))
    env["CXX_aarch64_linux_android"] = str(compiler_bin / ("clang++" + suffix))
    env["AR_aarch64_linux_android"] = str(compiler_bin / ("llvm-ar" + suffix))
    env["CFLAGS_aarch64_linux_android"] = f"--target={target}"
    env["CXXFLAGS_aarch64_linux_android"] = f"--target={target}"
    # Target-only flags: never pass Android linker arguments to host build scripts.
    env["CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS"] = (
        f"-C link-arg=--target={target} -C link-arg=-Wl,-z,max-page-size=16384"
    )
    # The repository's global -Zthreads flag otherwise overrides target flags.
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join([
        "-C", f"link-arg=--target={target}", "-C", "link-arg=-Wl,-z,max-page-size=16384",
    ])
    return pins, env


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Type-check only; no linked library or APK")
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    try:
        pins, env = environment()
        cargo = shutil.which("cargo", path=env["PATH"])
        if cargo is None:
            raise ValueError("cargo is not available; configure Rust on PATH")
        command = [cargo, "check" if args.check else "build", "--locked",
                   "-p", "stratum-android", "--target", pins["rust_target"]]
        if not args.check:
            command += ["--profile", "fast"]
        if args.offline:
            command.append("--offline")
        return subprocess.run(command, cwd=ROOT, env=env).returncode
    except (OSError, ValueError) as error:
        print(f"android_build: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
