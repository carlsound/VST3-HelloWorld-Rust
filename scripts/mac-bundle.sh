
#!/usr/bin/env bash
set -euo pipefail
CRATE_DIR=${1:-vst3_skeleton_db_vertical_meters_tooltips}
PLUGIN_NAME=${2:-Vst3Skeleton}
pushd "$CRATE_DIR" >/dev/null
cargo build --release
popd >/dev/null
ROOT=$(pwd)
TARGET="$ROOT/$CRATE_DIR/target/release"
DYLIB="$TARGET/libvst3_skeleton_db_vertical_meters_tooltips.dylib"
BUNDLE_ROOT="$ROOT/$PLUGIN_NAME.vst3"
CONTENTS="$BUNDLE_ROOT/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"
mkdir -p "$MACOS" "$RESOURCES"
cp "$DYLIB" "$MACOS/$PLUGIN_NAME"
cp -R "$CRATE_DIR/Resources"/* "$RESOURCES" || true
cat > "$CONTENTS/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Vst3Skeleton</string>
    <key>CFBundleIdentifier</key>
    <string>com.example.vst3skeleton</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundlePackageType</key>
    <string>BNDL</string>
    <key>CFBundleExecutable</key>
    <string>Vst3Skeleton</string>
</dict>
</plist>
PLIST
cat > "$CONTENTS/PkgInfo" << 'PKG'
BNDL????
PKG
echo "macOS VST3 bundle created at $BUNDLE_ROOT"
echo "Install to: /Library/Audio/Plug-Ins/VST3/$PLUGIN_NAME.vst3 or ~/Library/Audio/Plug-Ins/VST3/$PLUGIN_NAME.vst3"
