import { join } from "node:path";
import { Resvg } from "@resvg/resvg-js";
import { render } from "pele";

const docsDir = join(import.meta.dir, "..", "docs");
const sourcePath = join(docsDir, "work-breakdown-structure.mmd");
const svgPath = join(docsDir, "work-breakdown-structure.svg");
const pngPath = join(docsDir, "work-breakdown-structure.png");

const source = await Bun.file(sourcePath).text();
const { svg } = render(source);

const pngSvg = svg
  .replace(/var\(--_bg\)/g, "#fff")
  .replace(/var\(--_fg\)/g, "#1f1f1f")
  .replace(/var\(--_m\)/g, "#6e6e6e")
  .replace(/var\(--_l\)/g, "#1f1f1f")
  .replace(/var\(--_s\)/g, "#f3f3f3")
  .replace(/var\(--_a\)/g, "#e6e6e6")
  .replace(/var\(--_b\)/g, "#8a8a8a")
  .replace(/var\(--_c\)/g, "#5b7bd5")
  .replace(/var\(--_fm\)/g, "monospace")
  .replace(/var\((--[\w-]+),([^)]+)\)/g, "$2");

const png = new Resvg(pngSvg, {
  fitTo: { mode: "width", value: 1800 },
}).render().asPng();

await Bun.write(svgPath, svg);
await Bun.write(pngPath, png);

console.log(`Rendered ${svgPath}`);
console.log(`Rendered ${pngPath}`);
