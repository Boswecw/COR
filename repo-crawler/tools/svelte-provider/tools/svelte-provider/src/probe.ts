import { readFile, stat } from "node:fs/promises";
import path from "node:path";
import { createHash } from "node:crypto";
import { parse } from "svelte/compiler";

type JsonPrimitive = string | number | boolean | null;
type JsonValue = JsonPrimitive | JsonValue[] | { [key: string]: JsonValue };

type ProbeSuccess = {
  ok: true;
  provider: "svelte-probe";
  version: 1;
  file: string;
  absolutePath: string;
  ext: string;
  bytes: number;
  sha256: string;
  parsedWith: {
    engine: "svelte/compiler";
    mode: "modern";
  };
  heuristics: {
    hasSvelte5Runes: boolean;
    runes: {
      state: boolean;
      derived: boolean;
      effect: boolean;
      props: boolean;
    };
    template: {
      snippet: boolean;
      render: boolean;
      legacyEventDirective: boolean;
      eventAttributes: boolean;
      styleBlock: boolean;
      scriptInstance: boolean;
      scriptModule: boolean;
    };
  };
};

type ProbeFailure = {
  ok: false;
  provider: "svelte-probe";
  version: 1;
  file: string | null;
  absolutePath: string | null;
  error: {
    kind: "usage" | "fs" | "parse" | "unknown";
    message: string;
    code?: string | null;
    details?: JsonValue;
  };
};

function printJson(value: ProbeSuccess | ProbeFailure, exitCode = 0): never {
  process.stdout.write(`${JSON.stringify(value, null, 2)}\n`);
  process.exit(exitCode);
}

function detectHeuristics(source: string) {
  const runes = {
    state: /\$state\s*\(/.test(source),
    derived: /\$derived\s*\(/.test(source),
    effect: /\$effect\s*\(/.test(source),
    props: /\$props\s*\(/.test(source),
  };

  const template = {
    snippet: /\{#snippet\b/.test(source),
    render: /\{@render\b/.test(source),
    legacyEventDirective: /\bon:[\w-]+\s*=/.test(source),
    eventAttributes: /\bon[a-z][\w-]*\s*=/.test(source),
    styleBlock: /<style(\s|>)/i.test(source),
    scriptInstance: /<script(?:(?!\bmodule\b)[^>])*?>/i.test(source),
    scriptModule: /<script\b[^>]*\bmodule\b[^>]*>/i.test(source),
  };

  return {
    hasSvelte5Runes:
      runes.state || runes.derived || runes.effect || runes.props,
    runes,
    template,
  };
}

function normalizeParseError(error: unknown): ProbeFailure["error"] {
  if (error && typeof error === "object") {
    const err = error as {
      name?: string;
      message?: string;
      code?: string;
      start?: { line?: number; column?: number; character?: number };
      end?: { line?: number; column?: number; character?: number };
      position?: [number, number] | number;
      frame?: string;
      stack?: string;
    };

    return {
      kind: "parse",
      message: err.message ?? "Unknown Svelte parse error",
      code: err.code ?? err.name ?? null,
      details: {
        start: err.start
          ? {
              line: err.start.line ?? null,
              column: err.start.column ?? null,
              character: err.start.character ?? null,
            }
          : null,
        end: err.end
          ? {
              line: err.end.line ?? null,
              column: err.end.column ?? null,
              character: err.end.character ?? null,
            }
          : null,
        position: Array.isArray(err.position)
          ? err.position
          : err.position ?? null,
        frame: err.frame ?? null,
      },
    };
  }

  return {
    kind: "unknown",
    message: error instanceof Error ? error.message : "Unknown error",
    code: null,
  };
}

async function main() {
  const rawArg = process.argv[2];

  if (!rawArg) {
    printJson(
      {
        ok: false,
        provider: "svelte-probe",
        version: 1,
        file: null,
        absolutePath: null,
        error: {
          kind: "usage",
          message: "Usage: bun run src/probe.ts <path-to-file.svelte>",
          code: "USAGE",
        },
      },
      2,
    );
  }

  const absolutePath = path.resolve(rawArg);
  const file = path.basename(absolutePath);
  const ext = path.extname(absolutePath).toLowerCase();

  if (ext !== ".svelte") {
    printJson(
      {
        ok: false,
        provider: "svelte-probe",
        version: 1,
        file,
        absolutePath,
        error: {
          kind: "usage",
          message: "Probe target must be a .svelte file",
          code: "BAD_EXTENSION",
          details: { ext },
        },
      },
      2,
    );
  }

  let bytes = 0;
  let source = "";

  try {
    const fileStat = await stat(absolutePath);
    bytes = fileStat.size;
    source = await readFile(absolutePath, "utf8");
  } catch (error) {
    const err = error as NodeJS.ErrnoException;
    printJson(
      {
        ok: false,
        provider: "svelte-probe",
        version: 1,
        file,
        absolutePath,
        error: {
          kind: "fs",
          message: err.message,
          code: err.code ?? null,
        },
      },
      1,
    );
  }

  const sha256 = createHash("sha256").update(source).digest("hex");

  try {
    parse(source, {
      filename: absolutePath,
      modern: true,
    });

    const result: ProbeSuccess = {
      ok: true,
      provider: "svelte-probe",
      version: 1,
      file,
      absolutePath,
      ext,
      bytes,
      sha256,
      parsedWith: {
        engine: "svelte/compiler",
        mode: "modern",
      },
      heuristics: detectHeuristics(source),
    };

    printJson(result, 0);
  } catch (error) {
    printJson(
      {
        ok: false,
        provider: "svelte-probe",
        version: 1,
        file,
        absolutePath,
        error: normalizeParseError(error),
      },
      1,
    );
  }
}

void main();
