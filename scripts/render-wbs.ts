import path from "node:path";

import { Resvg } from "@resvg/resvg-js";
import { render } from "pele";

const docsDir = path.join(import.meta.dir, "..", "docs/wbs");
const sourcePath = path.join(docsDir, "work-breakdown-structure.mmd");
const svgPath = path.join(docsDir, "work-breakdown-structure.svg");
const pngPath = path.join(docsDir, "work-breakdown-structure.png");

const source = await Bun.file(sourcePath).text();
const { svg } = render(source);

const pngSvg = svg
  .replaceAll("var(--_bg)", "#fff")
  .replaceAll("var(--_fg)", "#1f1f1f")
  .replaceAll("var(--_m)", "#6e6e6e")
  .replaceAll("var(--_l)", "#1f1f1f")
  .replaceAll("var(--_s)", "#f3f3f3")
  .replaceAll("var(--_a)", "#e6e6e6")
  .replaceAll("var(--_b)", "#8a8a8a")
  .replaceAll("var(--_c)", "#5b7bd5")
  .replaceAll("var(--_fm)", "monospace")
  .replaceAll(/var\((?<name>--[\w-]+),(?<value>[^)]+)\)/gu, "$<value>");

const png = new Resvg(pngSvg, {
  background: "#ffffff",
  fitTo: { mode: "width", value: 1800 },
})
  .render()
  .asPng();

await Bun.write(svgPath, svg);

await Bun.write(pngPath, png);

console.log(`Rendered ${svgPath}`);

console.log(`Rendered ${pngPath}`);
