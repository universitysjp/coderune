import { ApplicationError } from "@coderune/core";

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
  params?: Record<string, unknown>;
}

export interface JsonRpcRequest {
  id: number | string;
  method: string;
  params?: Record<string, unknown>;
}

interface PendingRequest {
  resolve: (value: unknown) => void;
  reject: (reason: Error) => void;
  timer: ReturnType<typeof setTimeout>;
}

export interface JsonRpcPeerOptions {
  cwd: string;
  timeoutMs?: number;
  signal?: AbortSignal;
}

type Message = Record<string, unknown>;

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
  ) => Promise<unknown> | unknown;
  #nextId = 1;
  #closed = false;
  #readError: Error | null = null;
  readonly #stderr: Promise<string>;

  constructor(
    command: string[],
    options: JsonRpcPeerOptions,
    requestHandler: (request: JsonRpcRequest) => Promise<unknown> | unknown
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

  async request(
    method: string,
    params: Record<string, unknown>,
    timeoutMs = this.#timeoutMs
  ): Promise<unknown> {
    if (this.#closed) {
      throw (
        this.#readError ??
        new ApplicationError(
          "agent-failed",
          "The coding agent connection closed."
        )
      );
    }

    const id = this.#nextId++;

    const result = new Promise<unknown>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.#pending.delete(id);
        reject(
          new ApplicationError(
            "agent-timeout",
            "The coding agent did not respond within the allowed time."
          )
        );
      }, timeoutMs);

      this.#pending.set(id, { resolve, reject, timer });
    });

    this.#send({ jsonrpc: "2.0", id, method, params });

    return result;
  }

  notify(method: string, params: Record<string, unknown> = {}): void {
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

  async stderr(): Promise<string> {
    return this.#stderr;
  }

  #send(message: Record<string, unknown>): void {
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
    const reader = this.#process.stdout.getReader();
    const decoder = new TextDecoder();
    let buffer = "";

    try {
      while (true) {
        const { value, done } = await reader.read();

        if (done) {
          break;
        }

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
      const stderr = (await this.#stderr).trim();

      const error =
        this.#readError ??
        new ApplicationError(
          "agent-failed",
          stderr
            ? `${this.#name()} exited with code ${exitCode}: ${stderr.slice(-800)}`
            : `${this.#name()} exited with code ${exitCode}.`
        );

      this.#failPending(error);
    }
  }

  #name(): string {
    return "The coding agent";
  }

  #handleLine(line: string): void {
    let message: Message;

    try {
      message = JSON.parse(line) as Message;
    } catch {
      return;
    }

    const id = message.id as number | string | undefined;
    const method = typeof message.method === "string" ? message.method : null;

    if (method && id !== undefined && id !== null) {
      Promise.resolve(
        this.#requestHandler({
          id,
          method,
          params: message.params as Record<string, unknown> | undefined,
        })
      )
        .then((result) => {
          this.#send({ jsonrpc: "2.0", id, result });
        })
        .catch((error) => {
          this.#send({
            jsonrpc: "2.0",
            id,
            error: {
              code: -32_601,
              message:
                error instanceof Error ? error.message : "Request rejected.",
            },
          });
        });

      return;
    }

    if (id !== undefined && id !== null) {
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
            String(
              (message.error as { message?: unknown }).message ??
                "The coding agent rejected the request."
            )
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
        params: message.params as Record<string, unknown> | undefined,
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
