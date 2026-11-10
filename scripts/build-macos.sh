#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."

export PATH="$PATH:${CARGO_HOME:-$HOME/.cargo}/bin"
if ! command -v cargo >/dev/null; then
    echo "Cargo est requis pour compiler le moteur Rust." >&2
    exit 1
fi

build_dir="${SCREEN_TRANSLATE_BUILD_DIR:-$PWD/build}"
mkdir -p "$build_dir"
build_dir="$(cd "$build_dir" && pwd)"
export MACOSX_DEPLOYMENT_TARGET=15.0
cargo test --locked --offline --target-dir "$build_dir/rust" 2>&1 | tee "$build_dir/rust-tests.log"
cargo build --locked --offline --release --target aarch64-apple-darwin --target-dir "$build_dir/rust" 2>&1 | tee "$build_dir/rust-build.log"
xcodebuild build -project ScreenTranslate.xcodeproj -scheme ScreenTranslate \
    -configuration Release -destination 'platform=macOS,arch=arm64' \
    -derivedDataPath "$build_dir/xcode" CODE_SIGNING_ALLOWED=NO 2>&1 | tee "$build_dir/xcode-build.log"

app="$build_dir/xcode/Build/Products/Release/ScreenTranslate.app"
helper="$app/Contents/Helpers/screen-translate-export"
install -d "$(dirname "$helper")"
install -m 755 "$build_dir/rust/aarch64-apple-darwin/release/screen-translate-export" "$helper"
cp LICENSE "$app/Contents/Resources/ScreenTranslate-LICENSE.txt"
for binary in "$app/Contents/MacOS/ScreenTranslate" "$helper"; do
    lipo -verify_arch arm64 "$binary"
    codesign --force --sign - "$binary"
done
codesign --force --sign - "$app"
codesign --verify --deep --strict "$app"

if [[ "${1:-}" == "--dmg" ]]; then
    hdiutil create -ov -format UDZO -volname ScreenTranslate \
        -srcfolder "$app" "$build_dir/ScreenTranslate-arm64.dmg"
fi
printf '%s\n' "$app"
