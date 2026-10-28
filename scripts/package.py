#!/usr/bin/env python3
"""Package a locally built binary; run on the same OS as the target executable."""
import argparse
import hashlib
import platform
import plistlib
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", help="Rust target triple, if built with --target")
    args = parser.parse_args()
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]["version"]
    system = platform.system().lower()
    architecture = args.target or platform.machine()
    name = f"balatro-workshop-{version}-{system}-{architecture}"
    executable = "balatro-workshop.exe" if system == "windows" else "balatro-workshop"
    target = ROOT / "target"
    if args.target:
        target /= args.target
    binary = target / "release" / executable
    if not binary.is_file():
        raise SystemExit(f"Build the release binary first: {binary}")
    output = ROOT / "dist"
    output.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory() as temporary:
        stage = Path(temporary) / name
        stage.mkdir()
        for filename in ("README.md", "LICENSE", "NOTICE.md", "CHANGELOG.md"):
            shutil.copy2(ROOT / filename, stage)
        if (ROOT / "third-party").is_dir():
            shutil.copytree(ROOT / "third-party", stage / "third-party")
        if system == "darwin":
            app = stage / "Balatro Workshop.app" / "Contents"
            (app / "MacOS").mkdir(parents=True)
            shutil.copy2(binary, app / "MacOS" / executable)
            metadata = {
                "CFBundleName": "Balatro Workshop",
                "CFBundleDisplayName": "Balatro Workshop",
                "CFBundleIdentifier": "local.balatroworkshop.desktop",
                "CFBundleExecutable": executable,
                "CFBundlePackageType": "APPL",
                "CFBundleShortVersionString": version,
                "CFBundleVersion": version,
                "LSMinimumSystemVersion": "11.0",
                "NSHighResolutionCapable": True,
            }
            with (app / "Info.plist").open("wb") as stream:
                plistlib.dump(metadata, stream)
            # Ad-hoc local signing is not Developer ID signing or notarization.
            subprocess.run(["codesign", "--force", "--deep", "--sign", "-", str(app.parent)], check=True)
        else:
            shutil.copy2(binary, stage / executable)
        if system == "linux":
            archive = output / f"{name}.tar.gz"
            with tarfile.open(archive, "w:gz") as stream:
                stream.add(stage, arcname=name)
        else:
            archive = output / f"{name}.zip"
            with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED, strict_timestamps=False) as stream:
                for path in stage.rglob("*"):
                    if path.is_file():
                        stream.write(path, path.relative_to(stage.parent))
        checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
        archive.with_name(archive.name + ".sha256").write_text(f"{checksum}  {archive.name}\n")
        print(archive)


if __name__ == "__main__":
    main()
