#!/bin/sh
# Build the prototype and wrap it in DoneRight.app so macOS treats it as a normal app.
set -e
cd "$(dirname "$0")/.."
cargo build
mkdir -p DoneRight.app/Contents/MacOS
cp target/debug/doneright-app DoneRight.app/Contents/MacOS/DoneRight
cat > DoneRight.app/Contents/Info.plist <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>DoneRight</string>
  <key>CFBundleDisplayName</key><string>DoneRight</string>
  <key>CFBundleIdentifier</key><string>dev.doneright.prototype</string>
  <key>CFBundleExecutable</key><string>DoneRight</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.0.1</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSMinimumSystemVersion</key><string>14.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST
echo "Built DoneRight.app. Open it with: open DoneRight.app"
