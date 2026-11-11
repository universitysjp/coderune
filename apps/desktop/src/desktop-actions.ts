import {
  accessSync,
  constants,
  existsSync,
  realpathSync,
  statSync,
} from "node:fs";
import path from "node:path";

export const directoryArgument = (args: string[]): string | null => {
  if (args.length === 0) {
    return null;
  }

  if (args.length !== 1 || args[0]?.startsWith("-")) {
    throw new Error("Usage: coderune [directory]");
  }

  const directory = path.resolve(args[0] ?? ".");

  try {
    accessSync(directory, constants.R_OK);

    if (statSync(directory).isDirectory()) {
      return directory;
    }
  } catch {
    // Report the same actionable error for missing and inaccessible paths.
  }

  throw new Error(`Not a readable directory: ${directory}`);
};

export const sameDirectory = (first: string, second: string): boolean => {
  try {
    return (
      realpathSync(path.resolve(first)) === realpathSync(path.resolve(second))
    );
  } catch {
    return false;
  }
};

const editors = [
  { name: "Visual Studio Code", command: "code", app: "Visual Studio Code" },
  { name: "Cursor", command: "cursor", app: "Cursor" },
  { name: "Antigravity", command: "antigravity", app: "Antigravity" },
  { name: "Zed", command: "zed", app: "Zed" },
] as const;

type InstalledEditor = (typeof editors)[number] & {
  executable: string;
  appPath: string | null;
};

export const installedEditors = (): InstalledEditor[] =>
  editors.flatMap((editor): InstalledEditor[] => {
    const executable = Bun.which(editor.command, { PATH: process.env.PATH });
    const appPath = `/Applications/${editor.app}.app`;

    if (executable) {
      return [{ ...editor, executable, appPath: null }];
    }

    if (process.platform === "darwin" && existsSync(appPath)) {
      return [{ ...editor, executable: "/usr/bin/open", appPath }];
    }

    return [];
  });

export const openEditor = async (
  editor: ReturnType<typeof installedEditors>[number],
  directory: string
): Promise<void> => {
  const child = Bun.spawn(
    editor.appPath
      ? [editor.executable, "-a", editor.appPath, directory]
      : [editor.executable, directory],
    {
      stdin: "ignore",
      stdout: "ignore",
      stderr: "pipe",
    }
  );

  const stderr = new Response(child.stderr).text();
  const exitCode = await child.exited;

  if (exitCode !== 0) {
    const message = await stderr;

    throw new Error(message.trim() || `Editor exited with code ${exitCode}.`);
  }
};
