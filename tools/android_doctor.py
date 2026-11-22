#!/usr/bin/env python3
"""Read-only checks of the pinned reference toolchain; never installs or runs games."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def probe(command):
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=30)
        return result.returncode == 0, (result.stdout + result.stderr).strip()
    except (OSError, subprocess.TimeoutExpired) as exc:
        return False, str(exc)


def version(text):
    match = re.search(r"\b(\d+)\.(\d+)(?:\.(\d+))?", text)
    return tuple(int(part or 0) for part in match.groups()) if match else (0, 0, 0)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="Print machine-readable results")
    args = parser.parse_args()
    pins = json.loads((ROOT / "upstream-lock.json").read_text(encoding="utf-8"))
    required = pins["android_reference"]
    checks = []

    def add(name, passed, detail):
        checks.append({"name": name, "ok": bool(passed), "detail": detail})

    add("Python", sys.version_info[:3] >= version(required["python_minimum"]),
        f"{sys.version.split()[0]}; need >= {required['python_minimum']}")
    ok, output = probe(["rustc", "--version"])
    add("Rust", ok and version(output) >= version(required["rust_minimum"]), output)
    ok, output = probe(["rustup", "target", "list", "--installed"])
    add("Rust Android target", ok and required["rust_target"] in output.splitlines(), output)
    ok, output = probe(["cargo", "ndk", "--version"])
    add("cargo-ndk", ok, output)

    java_home = os.environ.get("JAVA_HOME")
    suffix = ".exe" if os.name == "nt" else ""
    javac = str(Path(java_home) / "bin" / ("javac" + suffix)) if java_home else "javac"
    ok, output = probe([javac, "-version"])
    add("JDK", ok and version(output)[0] == required["jdk_major"],
        f"{output}; need javac {required['jdk_major']}")

    sdk_setting = os.environ.get("ANDROID_HOME") or os.environ.get("ANDROID_SDK_ROOT")
    sdk = Path(sdk_setting).expanduser() if sdk_setting else None
    add("Android SDK", sdk is not None and sdk.is_dir(),
        "Configured through environment" if sdk_setting else "Set ANDROID_HOME or ANDROID_SDK_ROOT")
    components = {
        "Android platform": Path("platforms") / f"android-{required['platform']}" / "android.jar",
        "Android build-tools": Path("build-tools") / required["build_tools"] / "source.properties",
        "Android NDK": Path("ndk") / required["ndk"] / "source.properties",
    }
    for name, relative in components.items():
        add(name, sdk is not None and (sdk / relative).is_file(), relative.as_posix())
    adb = sdk / "platform-tools" / ("adb" + suffix) if sdk else None
    add("adb", (adb is not None and adb.is_file()) or shutil.which("adb") is not None,
        "Required for later device verification; devices are not accessed by this check")

    report = {
        "reference": required,
        "checks": checks,
        "ready": all(check["ok"] for check in checks),
        "scope": "Tool availability only; no compilation, APK link, or device test performed",
    }
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        for check in checks:
            print(f"{'OK' if check['ok'] else 'MISSING'} {check['name']}: {check['detail']}")
        print(report["scope"])
    return 0 if report["ready"] else 1


if __name__ == "__main__":
    sys.exit(main())
