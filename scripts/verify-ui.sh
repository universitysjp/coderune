#!/usr/bin/env bash
# Render every workspace scene headlessly. Fails if the app panics or a snapshot
# is missing; the PNGs land in target/ui-snapshots.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="$ROOT/target/ui-snapshots"
mkdir -p "$OUT"
cd "$ROOT"
cargo build -p coderune-desktop
BIN="$ROOT/target/debug/coderune"
DATA="$(mktemp -d)"
trap 'rm -rf "$DATA"' EXIT
export CODERUNE_DATA_DIR="$DATA"

"$BIN" --snap "$OUT/onboarding.png" --size 1280x860
"$BIN" --snap "$OUT/onboarding-narrow.png" --size 980x680
for scene in onboarding-start onboarding-review; do
  "$BIN" --snap "$OUT/$scene.png" --scene "$scene" --size 1280x860
done
for scene in home profile profile-details projects new-project documents subagents avatars capture chat chat-floating settings storage settings-agents settings-fonts home-dark settings-dark settings-agents-dark settings-shortcuts; do
  "$BIN" --snap "$OUT/$scene.png" --scene "$scene" --size 1280x860
done
"$BIN" --snap "$OUT/home-narrow.png" --scene home --size 980x680
"$BIN" --snap "$OUT/capture-preview.png" --scene capture-preview --size 1280x860

for file in onboarding onboarding-narrow home home-narrow profile profile-details projects new-project documents subagents avatars capture capture-preview chat chat-floating settings storage settings-agents settings-fonts home-dark settings-dark settings-agents-dark settings-shortcuts onboarding-start onboarding-review; do
  test -s "$OUT/$file.png" || { echo "missing $OUT/$file.png" >&2; exit 1; }
done
echo "UI snapshots written to $OUT"
