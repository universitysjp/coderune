#!/usr/bin/env bash
set -euo pipefail

platform=${1:?desktop platform required}
version=${CODERUNE_BUILD_VERSION:?desktop version required}
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-pr\.[0-9]+\.[0-9]+)?$ ]]

out="$PWD/dist/desktop"
assets=apps/desktop/assets
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
mkdir -p "$out"

case "$platform" in
  linux-x64)
    package="$stage/coderune"
    mkdir -p "$package/DEBIAN"
    install -Dm755 target/release/coderune "$package/usr/bin/coderune"
    install -Dm644 "$assets/icon-512.png" "$package/usr/share/icons/hicolor/512x512/apps/coderune.png"
    install -Dm644 apps/desktop/packaging/coderune.desktop "$package/usr/share/applications/coderune.desktop"
    cat > "$package/DEBIAN/control" <<CONTROL
Package: coderune
Version: ${version/-pr./~pr.}
Architecture: amd64
Maintainer: Coderune
Depends: libc6 (>= 2.39), libgcc-s1, libxkbcommon0, libfontconfig1, libwayland-client0, libxcb1, libvulkan1
Description: Local context and assumption verification for AI coding assistants
CONTROL
    dpkg-deb --root-owner-group --build "$package" "$out/Coderune-linux-x64.deb"
    dpkg-deb --extract "$out/Coderune-linux-x64.deb" "$stage/check"
    executable="$stage/check/usr/bin/coderune"
    ;;
  macos-arm64)
    app="$stage/Coderune.app"
    mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
    install -m755 target/release/coderune "$app/Contents/MacOS/coderune"
    install -m644 "$assets/icon.icns" "$app/Contents/Resources/AppIcon.icns"
    cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key><string>edu.sjp.coderune</string>
  <key>CFBundleName</key><string>Coderune</string>
  <key>CFBundleExecutable</key><string>coderune</string>
  <key>CFBundleIconFile</key><string>AppIcon.icns</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>${version%%-*}</string>
  <key>CFBundleVersion</key><string>${version%%-*}</string>
</dict>
</plist>
PLIST
    plutil -lint "$app/Contents/Info.plist"
    codesign --force --sign - "$app"
    codesign --verify --strict "$app"
    ditto -c -k --sequesterRsrc --keepParent "$app" "$out/Coderune-macos-arm64.zip"
    ditto -x -k "$out/Coderune-macos-arm64.zip" "$stage/check"
    executable="$stage/check/Coderune.app/Contents/MacOS/coderune"
    ;;
  *)
    echo "Unsupported desktop platform: $platform" >&2
    exit 1
    ;;
esac

test "$("$executable" --version)" = "Coderune $version"
