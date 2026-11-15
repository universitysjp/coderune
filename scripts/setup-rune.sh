#!/usr/bin/env bash
# Fetch the pinned Rune checkout used by the desktop app.
#
# Cargo does not apply a dependency's [patch.crates-io] table, so this app patches
# wgpu-hal, winit, parley_engine, and the accesskit crates to Rune's patched copies.
# Those paths must exist before Cargo can resolve the workspace.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REV="8fc71fbf21db41920409bf6fa20ffb41ddb2f99d"
DEST="$ROOT/.rune"

# Turborepo discovers Cargo packages with `cargo metadata --all-features` and
# requires every local path package to be a workspace member. As a member,
# winit's optional `wayland-csd-adwaita-crossfont` feature would pull
# fontconfig 5 alongside fontique's fontconfig 6, and Cargo refuses two
# packages linking `fontconfig`. Rune uses the `...-notitle` decorations
# instead, so drop the unused crossfont feature from the vendored manifest.
apply_turborepo_member_patch() {
  local manifest="$DEST/vendor/winit/Cargo.toml"
  [ -f "$manifest" ] || return 0
  sed -i.bak \
    '/^wayland-csd-adwaita-crossfont = \[$/,/^\]$/d' \
    "$manifest"
  rm -f "$manifest.bak"
}

if [ -d "$DEST/.git" ]; then
  CURRENT="$(git -C "$DEST" rev-parse HEAD)"
  if [ "$CURRENT" = "$REV" ]; then
    apply_turborepo_member_patch
    echo "Rune $REV already present at .rune"
    exit 0
  fi
fi

if [ ! -d "$DEST/.git" ]; then
  git clone https://github.com/anomalyco/rune.git "$DEST"
fi
git -C "$DEST" fetch --depth 1 origin "$REV"
# Discard the previous manifest patch before switching revisions.
git -C "$DEST" checkout -- vendor/winit/Cargo.toml 2>/dev/null || true
git -C "$DEST" checkout --detach "$REV"
apply_turborepo_member_patch
echo "Rune $REV ready at .rune"
