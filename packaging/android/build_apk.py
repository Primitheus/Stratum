#!/usr/bin/env python3
"""Package the Stratum development native library; never installs on a device."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[2]


def run(*args):
    subprocess.run([str(arg) for arg in args], cwd=ROOT, check=True)


def tool(directory, name):
    for suffix in ([".exe", ".bat", ""] if os.name == "nt" else [""]):
        candidate = directory / (name + suffix)
        if candidate.is_file():
            return candidate
    raise ValueError(f"Missing Android tool: {name} in {directory}")


def validate_library(path):
    with path.open("rb") as stream:
        header = stream.read(20)
    # ELF64, little endian, shared object, AArch64. Reject a host or wrong-ABI build.
    if (len(header) != 20 or header[:6] != b"\x7fELF\x02\x01"
            or int.from_bytes(header[16:18], "little") != 3
            or int.from_bytes(header[18:20], "little") != 183):
        raise ValueError("Expected an ARM64 Android ELF shared library")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--no-build", action="store_true", help="Package the existing fast-profile library")
    args = parser.parse_args()
    try:
        pins = json.loads((ROOT / "upstream-lock.json").read_text())["android_reference"]
        configured = os.environ.get("ANDROID_HOME") or os.environ.get("ANDROID_SDK_ROOT")
        if not configured:
            raise ValueError("Set ANDROID_HOME or ANDROID_SDK_ROOT for packaging")
        sdk = Path(configured)
        build_tools = sdk / "build-tools" / pins["build_tools"]
        aapt = tool(build_tools, "aapt2")
        align = tool(build_tools, "zipalign")
        signer = tool(build_tools, "apksigner")
        platform = sdk / "platforms" / f"android-{pins['platform']}" / "android.jar"
        if not platform.is_file():
            raise ValueError(f"Missing Android platform {pins['platform']}")
        if not args.no_build:
            run(sys.executable, ROOT / "tools/android_build.py")
        library = ROOT / "target" / pins["rust_target"] / "fast/libstratum_android.so"
        validate_library(library)
        output = ROOT / "target/android"
        output.mkdir(parents=True, exist_ok=True)
        unsigned = output / "stratum-dev-unsigned.apk"
        aligned = output / "stratum-dev-aligned.apk"
        signed = output / "stratum-dev-arm64-v8a.apk"
        run(aapt, "link", "-o", unsigned, "--manifest", Path(__file__).with_name("AndroidManifest.xml"),
            "-I", platform)
        with zipfile.ZipFile(unsigned, "a", compression=zipfile.ZIP_STORED) as archive:
            archive.write(library, "lib/arm64-v8a/libstratum_android.so")
            wrapper = zipfile.ZipInfo("lib/arm64-v8a/wrap.sh")
            wrapper.create_system = 3
            wrapper.external_attr = 0o100755 << 16
            archive.writestr(wrapper, Path(__file__).with_name("wrap.sh").read_bytes())
            for path in [ROOT / "LICENSE", ROOT / "THIRD_PARTY_NOTICES.md", *sorted((ROOT / "LICENSES").glob("*.txt"))]:
                archive.write(path, "assets/licenses/" + path.name)
        run(align, "-P", "16", "-f", "4", unsigned, aligned)
        key = output / "stratum-debug.keystore"
        if not key.exists():
            java_home = os.environ.get("JAVA_HOME")
            keytool = tool(Path(java_home) / "bin", "keytool") if java_home else shutil.which("keytool")
            if not keytool:
                raise ValueError("Set JAVA_HOME or put keytool on PATH")
            run(keytool, "-genkeypair", "-noprompt", "-keystore", key, "-storepass", "android",
                "-keypass", "android", "-alias", "stratum-dev", "-keyalg", "RSA", "-keysize", "2048",
                "-validity", "3650", "-dname", "CN=Stratum Development")
        run(signer, "sign", "--ks", key, "--ks-key-alias", "stratum-dev",
            "--ks-pass", "pass:android", "--key-pass", "pass:android", "--out", signed, aligned)
        run(signer, "verify", "--verbose", signed)
        print(f"Development APK: {signed}\nNot installed or device-tested.")
        return 0
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"build_apk: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
