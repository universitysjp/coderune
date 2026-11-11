import { expect, test } from "bun:test";
import path from "node:path";

import {
  directoryArgument,
  openEditor,
  sameDirectory,
} from "../src/desktop-actions";

test("terminal directory arguments resolve and reject missing paths", () => {
  expect(directoryArgument(["."])).toBe(process.cwd());
  expect(() =>
    directoryArgument([path.join(process.cwd(), "missing-folder-for-test")])
  ).toThrow("Not a readable directory");
  expect(() => directoryArgument([".", "."])).toThrow("Usage");
});

test("equivalent directory paths identify the same workspace", () => {
  expect(sameDirectory(process.cwd(), path.join(process.cwd(), "."))).toBe(
    true
  );
  expect(
    sameDirectory(
      process.cwd(),
      path.join(process.cwd(), "missing-folder-for-test")
    )
  ).toBe(false);
});

test("editor launch reports a failing executable", async () => {
  const editor = {
    name: "Cursor",
    command: "cursor",
    app: "Cursor",
    executable: "/usr/bin/false",
    appPath: null,
  } as const;
  await expect(openEditor(editor, process.cwd())).rejects.toThrow(
    "Editor exited with code"
  );
});
