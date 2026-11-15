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

if [ -d "$DEST/.git" ]; then
  CURRENT="$(git -C "$DEST" rev-parse HEAD)"
  if [ "$CURRENT" = "$REV" ]; then
    echo "Rune $REV already present at .rune"
    exit 0
  fi
fi

if [ ! -d "$DEST/.git" ]; then
  git clone https://github.com/anomalyco/rune.git "$DEST"
fi
git -C "$DEST" fetch --depth 1 origin "$REV"
git -C "$DEST" checkout --detach "$REV"
echo "Rune $REV ready at .rune"
