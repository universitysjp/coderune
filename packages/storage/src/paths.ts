import { homedir } from "node:os";
import path from "node:path";

// Keep the original storage identifiers so Coderune reopens existing profiles and captures.
export const APPLICATION_DIRECTORY_NAME = "Context Bridge";

export const resolveApplicationDataDir = function resolveApplicationDataDir(
  environment: NodeJS.ProcessEnv = process.env
): string {
  if (process.platform === "darwin") {
    return path.join(
      homedir(),
      "Library",
      "Application Support",
      APPLICATION_DIRECTORY_NAME
    );
  }

  if (process.platform === "win32") {
    const appData =
      environment.APPDATA ?? path.join(homedir(), "AppData", "Roaming");

    return path.join(appData, APPLICATION_DIRECTORY_NAME);
  }

  const xdgDataHome = environment.XDG_DATA_HOME;

  return xdgDataHome
    ? path.join(xdgDataHome, "context-bridge")
    : path.join(homedir(), ".local", "share", "context-bridge");
};

export const resolveDatabasePath = function resolveDatabasePath(
  dataDir: string
): string {
  return path.join(dataDir, "context-bridge.sqlite");
};
