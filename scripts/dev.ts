import path from "node:path";

const rootDir = path.resolve(import.meta.dir, "..");

const desktop = Bun.spawn([process.execPath, "--hot", "app.tsx"], {
  cwd: path.resolve(rootDir, "apps", "desktop"),
  stderr: "inherit",
  stdout: "inherit",
});

const web = Bun.spawn(
  [
    process.execPath,
    path.resolve(rootDir, "node_modules", "vite-plus", "bin", "vp"),
    "dev",
  ],
  {
    cwd: path.resolve(rootDir, "apps", "web"),
    stderr: "inherit",
    stdout: "inherit",
  }
);

const stop = (signal: NodeJS.Signals) => {
  desktop.kill(signal);
  web.kill(signal);
};

process.on("SIGINT", () => stop("SIGINT"));

process.on("SIGTERM", () => stop("SIGTERM"));

const [exitCode] = await Promise.race([
  desktop.exited.then((code) => [code, "desktop"] as const),
  web.exited.then((code) => [code, "web"] as const),
]);

if (desktop.exitCode === null) {
  desktop.kill();
}

if (web.exitCode === null) {
  web.kill();
}

process.exitCode = exitCode;
