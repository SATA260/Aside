import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { defineConfig } from "@playwright/test";

// 端到端测试自己起一个开发桥和前端，数据放在临时目录，不碰 .devdata，也不占 1420 / 1430
const VITE_PORT = 1520;
const BRIDGE_PORT = 1530;
const dataDir = process.env.ASIDE_E2E_DATA ?? mkdtempSync(join(tmpdir(), "aside-e2e-"));
process.env.ASIDE_E2E_DATA = dataDir;

export default defineConfig({
  testDir: "tests/e2e",
  workers: 1,
  fullyParallel: false,
  timeout: 30_000,
  use: {
    baseURL: `http://localhost:${VITE_PORT}`,
    channel: process.env.CI ? undefined : "chrome",
  },
  webServer: [
    {
      command: "cargo run --quiet --manifest-path src-tauri/Cargo.toml --features devserver --bin devserver",
      port: BRIDGE_PORT,
      env: { ASIDE_DATA_DIR: dataDir, ASIDE_BRIDGE_PORT: String(BRIDGE_PORT) },
      timeout: 600_000,
      reuseExistingServer: false,
    },
    {
      command: "pnpm exec vite",
      port: VITE_PORT,
      env: { ASIDE_VITE_PORT: String(VITE_PORT), ASIDE_BRIDGE_PORT: String(BRIDGE_PORT) },
      reuseExistingServer: false,
    },
  ],
});
