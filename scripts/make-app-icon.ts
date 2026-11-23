#!/usr/bin/env bun
/**
 * Build the desktop app icon from the supplied mark.
 *
 * `apps/desktop/assets/icon-source.png` is the mark alone: an off-white glyph
 * on a transparent field. The background is composed here, following the same
 * recipe as Hex's Icon Composer asset (`assets/AppIcon.icon/icon.json`): a
 * grey-to-black gradient with the mark at half scale.
 *
 * Run from the repository root:
 *
 *     bun run icon:build
 *
 * Outputs a 1024 px master, a 512 px Linux/Windows copy, `icon.icns` for macOS
 * packaging, and a masked PNG for standalone macOS builds. `AppIcon.icon` is
 * the editable Icon Composer source.
 */

import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";

import { Resvg } from "@resvg/resvg-js";

interface Size {
  width: number;
  height: number;
}

const ROOT = path.join(import.meta.dir, "..");
const ASSETS = path.join(ROOT, "apps/desktop/assets");
const SOURCE = path.join(ASSETS, "icon-source.png");
const SIDE = 1024;
const MARK_SCALE = 0.5;
const GRADIENT_STOP = 0.7;
const SQUARE_FIELD = { inset: 0, radius: 0 };
const DOCK_FIELD = { inset: 100, radius: 184 };

const SIZES = [
  { name: "icon-1024.png", size: 1024 },
  { name: "icon-512.png", size: 512 },
] as const;

const ICONSET_SIZES = [16, 32, 128, 256, 512] as const;
const SCALES = [1, 2] as const;

const pngSize = (bytes: Uint8Array): Size => {
  const signature = [137, 80, 78, 71, 13, 10, 26, 10];

  if (signature.some((byte, index) => bytes[index] !== byte)) {
    throw new Error(`${SOURCE} is not a PNG`);
  }

  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);

  return { width: view.getUint32(16), height: view.getUint32(20) };
};

const svg = (
  href: string,
  mark: Size,
  { inset, radius } = SQUARE_FIELD
): string => {
  const fieldSize = SIDE - 2 * inset;
  const markScale = MARK_SCALE * (fieldSize / SIDE);
  const width = Math.round(mark.width * markScale);
  const height = Math.round(mark.height * markScale);
  const x = (SIDE - width) / 2;
  const y = (SIDE - height) / 2;

  return `<svg xmlns="http://www.w3.org/2000/svg" width="${SIDE}" height="${SIDE}" viewBox="0 0 ${SIDE} ${SIDE}">
  <defs>
    <linearGradient id="field" gradientUnits="userSpaceOnUse" x1="0" y1="${inset}" x2="0" y2="${inset + fieldSize * GRADIENT_STOP}">
      <stop offset="0" stop-color="#333333"/>
      <stop offset="1" stop-color="#000000"/>
    </linearGradient>
  </defs>
  <rect x="${inset}" y="${inset}" width="${fieldSize}" height="${fieldSize}" rx="${radius}" fill="url(#field)"/>
  <image href="${href}" x="${x}" y="${y}" width="${width}" height="${height}"/>
</svg>`;
};

const render = (source: string, size: number): Buffer =>
  new Resvg(source, {
    fitTo: { mode: "width", value: size },
  })
    .render()
    .asPng();

const writeIcns = async (master: string): Promise<void> => {
  if (Bun.which("iconutil") === null) {
    console.log("iconutil not found; skipped icon.icns");

    return;
  }

  const directory = await mkdtemp(path.join(tmpdir(), "coderune-icon-"));
  const iconset = path.join(directory, "coderune.iconset");

  try {
    await mkdir(iconset);
    await Promise.all(
      ICONSET_SIZES.flatMap((size) =>
        SCALES.map((scale) => {
          const name = `icon_${size}x${size}${scale === 1 ? "" : "@2x"}.png`;

          return writeFile(
            path.join(iconset, name),
            render(master, size * scale)
          );
        })
      )
    );

    const result = Bun.spawnSync([
      "iconutil",
      "-c",
      "icns",
      iconset,
      "-o",
      path.join(ASSETS, "icon.icns"),
    ]);

    if (result.exitCode !== 0) {
      throw new Error(result.stderr.toString() || "iconutil failed");
    }
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
};

const source = new Uint8Array(await Bun.file(SOURCE).arrayBuffer());
const href = `data:image/png;base64,${source.toBase64()}`;
const mark = pngSize(source);
const document = svg(href, mark);

await Promise.all(
  SIZES.map(({ name, size }) =>
    Bun.write(path.join(ASSETS, name), render(document, size))
  )
);

await Bun.write(
  path.join(ASSETS, "icon-dock.png"),
  render(svg(href, mark, DOCK_FIELD), SIDE)
);

await writeIcns(document);

console.log(`wrote app icon assets to ${ASSETS}`);
