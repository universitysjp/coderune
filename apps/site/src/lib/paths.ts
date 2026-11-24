const base = import.meta.env.BASE_URL.replace(/\/$/u, "");

export const siteHref = (path: string) =>
  path === "/" ? `${base}/` : `${base}${path}/`;

export const isExternal = (path: string) =>
  path.startsWith("http") || path.startsWith("mailto:") || path.startsWith("#");
