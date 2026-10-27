#!/usr/bin/env node

import { execSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = resolve(__dirname, "..");

const target = "wasm32-wasip1";
const wasmSource = resolve(rootDir, "target", target, "release", "chinenshichanaka.wasm");
const distDir = resolve(rootDir, "dist");
const wasmDest = resolve(distDir, "chinenshichanaka.wasm");

console.log(`Building Rust project for target ${target}...`);
execSync(`cargo build --target ${target} --release`, {
  cwd: rootDir,
  stdio: "inherit",
});

if (!existsSync(wasmSource)) {
  console.error(`Build failed: Expected WASM output at ${wasmSource}`);
  process.exit(1);
}

mkdirSync(distDir, { recursive: true });
copyFileSync(wasmSource, wasmDest);
console.log(`Successfully staged WASM binary to ${wasmDest}`);
