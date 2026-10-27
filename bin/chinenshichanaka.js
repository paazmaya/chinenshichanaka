#!/usr/bin/env node

import { WASI } from "node:wasi";
import { readFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { resolve, dirname, parse, isAbsolute } from "node:path";
import { fileURLToPath } from "node:url";
import { tmpdir } from "node:os";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const wasmPath = resolve(__dirname, "..", "dist", "chinenshichanaka.wasm");

if (!existsSync(wasmPath)) {
  console.error(
    `Error: Compiled WebAssembly binary not found at ${wasmPath}.\nPlease run "npm run build" first.`,
  );
  process.exit(1);
}

const cwd = process.cwd();

// Forward arguments to WASI: resolve paths to absolute so the WASI filesystem can access them
const root = parse(cwd).root;
const preopens = {
  [root]: root,
};

// Process arguments:
// clap expects [program_name, ...args]
// If the user specified an input file without an output file, clap defaults output to "favicon.ico".
// In WASI, relative "favicon.ico" defaults to the root directory which is read-only.
// We explicitly resolve input and output paths to absolute paths so relative paths work seamlessly from cwd.
const rawArgs = process.argv.slice(2);
const positionalIndices = [];
const translatedArgs = [];

for (let i = 0; i < rawArgs.length; i++) {
  const arg = rawArgs[i];
  if (arg === "--help" || arg === "-h" || arg === "--version" || arg === "-V") {
    translatedArgs.push(arg);
  } else if (arg.startsWith("-")) {
    translatedArgs.push(arg);
  } else {
    positionalIndices.push(translatedArgs.length);
    translatedArgs.push(resolve(cwd, arg));
  }
}

// If 1 positional argument was given (input only), add explicit resolved output path for favicon.ico
if (positionalIndices.length === 1) {
  translatedArgs.push(resolve(cwd, "favicon.ico"));
}

const wasi = new WASI({
  version: "preview1",
  args: ["chinenshichanaka", ...translatedArgs],
  env: process.env,
  preopens,
});

try {
  const wasmBuffer = await readFile(wasmPath);
  const wasmModule = await WebAssembly.compile(wasmBuffer);
  const instance = await WebAssembly.instantiate(wasmModule, wasi.getImportObject());

  const exitCode = wasi.start(instance);
  if (typeof exitCode === "number") {
    process.exit(exitCode);
  }
} catch (error) {
  if (
    error &&
    typeof error === "object" &&
    "code" in error &&
    error.code === "ERR_WASI_NOT_STARTED"
  ) {
    // WASI process exited cleanly via proc_exit
  } else if (error && typeof error.exitCode === "number") {
    process.exit(error.exitCode);
  } else {
    // Check if error is WebAssembly.RuntimeError caused by proc_exit
    // Typically in Node.js, wasi.start throws a system error or returns exit code
    console.error(error);
    process.exit(1);
  }
}
