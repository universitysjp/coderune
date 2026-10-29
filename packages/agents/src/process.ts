import { ApplicationError } from "@coderune/core";
import type { JsonObject, JsonValue } from "@coderune/core";
import { z } from "zod";

import { jsonObjectSchema, jsonValueSchema } from "./parse";

export interface ProcessOptions {
  cwd: string;
  input?: string;
  timeoutMs?: number;
  signal?: AbortSignal;
}

export interface ProcessResult {
  stdout: string;
  stderr: string;
  exitCode: number;
}

const DEFAULT_TIMEOUT_MS = 120_000;

const jsonRpcIdSchema = z.union([z.string(), z.number()]);
const jsonRpcErrorSchema = z.object({ message: z.string().optional() });

const jsonRpcMessageSchema = z.object({
  id: jsonRpcIdSchema.optional(),
  method: z.string().optional(),
  params: jsonObjectSchema.optional(),
  result: jsonValueSchema.optional(),
  error: jsonRpcErrorSchema.optional(),
});

export const runProcess = async function runProcess(
  command: string[],
  options: ProcessOptions
): Promise<ProcessResult> {
  const timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
  let processHandle: Bun.Subprocess<"pipe", "pipe", "pipe">;

  try {
    processHandle = Bun.spawn({
      cmd: command,
      cwd: options.cwd,
      env: process.env,
      stdin: "pipe",
      stdout: "pipe",
      stderr: "pipe",
    });
  } catch (error) {
    throw new ApplicationError(
      "agent-unavailable",
      error instanceof Error
        ? error.message
        : "The selected coding agent could not be started."
    );
  }

  let timedOut = false;

  const timer = setTimeout(() => {
    timedOut = true;
    processHandle.kill();
  }, timeoutMs);

  const killOnAbort = () => processHandle.kill();
  options.signal?.addEventListener("abort", killOnAbort, { once: true });

  try {
    if (options.input !== undefined) {
      processHandle.stdin.write(options.input);
    }

    processHandle.stdin.end();

    const stdoutPromise = new Response(processHandle.stdout).text();
    const stderrPromise = new Response(processHandle.stderr).text();

    const [stdout, stderr, exitCode] = await Promise.all([
      stdoutPromise,
      stderrPromise,
      processHandle.exited,
    ]);

    if (options.signal?.aborted) {
      throw new ApplicationError(
        "agent-cancelled",
        "The coding agent draft was cancelled."
      );
    }

    if (timedOut) {
      throw new ApplicationError(
        "agent-timeout",
        "The coding agent did not finish within the allowed time."
      );
    }

    return { stdout, stderr, exitCode };
  } finally {
    clearTimeout(timer);
    options.signal?.removeEventListener("abort", killOnAbort);
  }
};

export interface JsonRpcNotification {
  method: string;
  params?: JsonObject;
}

export interface JsonRpcRequest {
  id: number | string;
  method: string;
  params?: JsonObject;
}

interface PendingRequest {
  resolve: (value: JsonValue | undefined) => void;
  reject: (reason: Error) => void;
  timer: ReturnType<typeof setTimeout>;
}

export interface JsonRpcPeerOptions {
  cwd: string;
  timeoutMs?: number;
  signal?: AbortSignal;
}

export class JsonRpcPeer {
  readonly #process: Bun.Subprocess<"pipe", "pipe", "pipe">;
  readonly #timeoutMs: number;
  readonly #signal?: AbortSignal;
  readonly #pending = new Map<number | string, PendingRequest>();
  readonly #notificationListeners = new Set<
    (notification: JsonRpcNotification) => void
  >();
  readonly #requestHandler: (
    request: JsonRpcRequest
  ) => Promise<JsonValue | undefined> | JsonValue | undefined;
  #nextId = 1;
  #closed = false;
  #readError: Error | null = null;
  readonly #stderr: Promise<string>;

  constructor(
    command: string[],
    options: JsonRpcPeerOptions,
    requestHandler: (
      request: JsonRpcRequest
    ) => Promise<JsonValue | undefined> | JsonValue | undefined
  ) {
    this.#timeoutMs = options.timeoutMs ?? DEFAULT_TIMEOUT_MS;
    this.#signal = options.signal;
    this.#requestHandler = requestHandler;

    try {
      this.#process = Bun.spawn({
        cmd: command,
        cwd: options.cwd,
        env: process.env,
        stdin: "pipe",
        stdout: "pipe",
        stderr: "pipe",
      });
    } catch (error) {
      throw new ApplicationError(
        "agent-unavailable",
        error instanceof Error
          ? error.message
          : "The selected coding agent could not be started."
      );
    }

    this.#stderr = new Response(this.#process.stderr).text();
    this.#signal?.addEventListener(
      "abort",
      () => {
        this.close();
      },
      { once: true }
    );
    void this.#readLoop();
  }

  onNotification(
    listener: (notification: JsonRpcNotification) => void
  ): () => void {
    this.#notificationListeners.add(listener);

    return () => this.#notificationListeners.delete(listener);
  }

  request(
    method: string,
    params: JsonObject,
    timeoutMs = this.#timeoutMs
  ): Promise<JsonValue | undefined> {
    if (this.#closed) {
      throw (
        this.#readError ??
        new ApplicationError(
          "agent-failed",
          "The coding agent connection closed."
        )
      );
    }

    const id = this.#nextId;
    this.#nextId += 1;

    const result = Promise.withResolvers<JsonValue | undefined>();

    const timer = setTimeout(() => {
      this.#pending.delete(id);
      result.reject(
        new ApplicationError(
          "agent-timeout",
          "The coding agent did not respond within the allowed time."
        )
      );
    }, timeoutMs);

    this.#pending.set(id, {
      resolve: result.resolve,
      reject: result.reject,
      timer,
    });

    this.#send({ jsonrpc: "2.0", id, method, params });

    return result.promise;
  }

  notify(method: string, params: JsonObject = {}): void {
    this.#send({ jsonrpc: "2.0", method, params });
  }

  close(): void {
    if (this.#closed) {
      return;
    }

    this.#closed = true;
    this.#process.kill();

    const error = new ApplicationError(
      "agent-cancelled",
      "The coding agent draft was cancelled."
    );

    this.#failPending(error);
  }

  stderr(): Promise<string> {
    return this.#stderr;
  }

  #send(message: JsonObject): void {
    try {
      this.#process.stdin.write(`${JSON.stringify(message)}\n`);
    } catch {
      if (!this.#closed) {
        this.#closed = true;
        this.#failPending(
          new ApplicationError(
            "agent-failed",
            "The coding agent connection closed unexpectedly."
          )
        );
      }
    }
  }

  async #readLoop(): Promise<void> {
    const decoder = new TextDecoder();
    let buffer = "";

    try {
      for await (const value of this.#process.stdout) {
        buffer += decoder.decode(value, { stream: true });
        let newline = buffer.indexOf("\n");

        while (newline >= 0) {
          const line = buffer.slice(0, newline).trim();
          buffer = buffer.slice(newline + 1);

          if (line) {
            this.#handleLine(line);
          }

          newline = buffer.indexOf("\n");
        }
      }

      buffer += decoder.decode();

      if (buffer.trim()) {
        this.#handleLine(buffer.trim());
      }
    } catch (error) {
      this.#readError = new ApplicationError(
        "agent-failed",
        error instanceof Error
          ? error.message
          : "The coding agent stream failed."
      );
    } finally {
      this.#closed = true;
      const exitCode = await this.#process.exited;
      const stderrOutput = await this.#stderr;
      const stderr = stderrOutput.trim();

      const error =
        this.#readError ??
        new ApplicationError(
          "agent-failed",
          stderr
            ? `${"The coding agent"} exited with code ${exitCode}: ${stderr.slice(-800)}`
            : `${"The coding agent"} exited with code ${exitCode}.`
        );

      this.#failPending(error);
    }
  }

  #handleLine(line: string): void {
    let raw: unknown;

    try {
      raw = JSON.parse(line);
    } catch {
      return;
    }

    const parsed = jsonRpcMessageSchema.safeParse(raw);

    if (!parsed.success) {
      return;
    }

    const message = parsed.data;
    const { id, method, params } = message;

    if (method && id !== undefined) {
      const handleRequest = async (): Promise<void> => {
        try {
          const result = await this.#requestHandler({
            id,
            method,
            params,
          });

          this.#send({ jsonrpc: "2.0", id, result: result ?? null });
        } catch (error) {
          this.#send({
            jsonrpc: "2.0",
            id,
            error: {
              code: -32_601,
              message:
                error instanceof Error ? error.message : "Request rejected.",
            },
          });
        }
      };

      void handleRequest();

      return;
    }

    if (id !== undefined) {
      const pending = this.#pending.get(id);

      if (!pending) {
        return;
      }

      clearTimeout(pending.timer);
      this.#pending.delete(id);

      if (message.error) {
        pending.reject(
          new ApplicationError(
            "agent-failed",
            message.error.message ?? "The coding agent rejected the request."
          )
        );
      } else {
        pending.resolve(message.result);
      }

      return;
    }

    if (method) {
      const notification: JsonRpcNotification = {
        method,
        params,
      };

      for (const listener of this.#notificationListeners) {
        listener(notification);
      }
    }
  }

  #failPending(error: Error): void {
    for (const pending of this.#pending.values()) {
      clearTimeout(pending.timer);
      pending.reject(error);
    }

    this.#pending.clear();
  }
}
