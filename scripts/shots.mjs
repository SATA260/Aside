// 各种窗口大小下给界面拍照，检查布局用。需要先开着 `pnpm dev:bridge` 和 `pnpm dev`。
// 用法：node scripts/shots.mjs [输出目录] [页面...]，页面可选 start subjects settings focus end mini
import { mkdirSync } from "node:fs";
import { chromium } from "@playwright/test";

const out = process.argv[2] ?? "/tmp/aside-shots";
const only = process.argv.slice(3);
const BASE = "http://localhost:1420";

const MAIN_SIZES = [
  [880, 600],
  [1040, 680],
  [1440, 900],
  [1920, 1050],
];
const SCREEN_SIZES = [
  [1280, 720],
  [1440, 900],
  [1920, 1080],
  [2560, 1440],
  [1080, 1920],
];

const call = (cmd, args = {}) =>
  fetch(`${BASE}/__dev/call`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ cmd, args }),
  }).then((r) => r.json());

const want = (name) => only.length === 0 || only.includes(name);

mkdirSync(out, { recursive: true });
const browser = await chromium.launch({ channel: "chrome" });

async function shoot(view, sizes, name, prepare, settle = 600) {
  for (const [w, h] of sizes) {
    const page = await browser.newPage({ viewport: { width: w, height: h } });
    await page.goto(`${BASE}/?view=${view}`);
    await page.waitForTimeout(settle);
    if (prepare) await prepare(page);
    await page.screenshot({ path: `${out}/${name}-${w}x${h}.png` });
    await page.close();
  }
}

const nav = (label) => async (page) => {
  await page.getByRole("button", { name: label, exact: true }).first().click();
  await page.waitForTimeout(300);
};

if (want("start")) await shoot("main", MAIN_SIZES, "start");
if (want("subjects")) await shoot("main", MAIN_SIZES, "subjects", nav("科目"));
if (want("settings")) await shoot("main", MAIN_SIZES, "settings", nav("设置"));

if (want("focus") || want("end") || want("mini")) {
  const view = await call("timer_view");
  if (view.value?.phase !== "idle") await call("timer_finish").then(() => call("timer_close", { again: false }));
  const subjects = await call("subjects_list");
  const subject = subjects.value.find((s) => s.name === "数学") ?? subjects.value[0];
  await call("timer_start", {
    subject_id: subject.id,
    mode: "countdown",
    planned_ms: 45 * 60_000,
    display: "fullscreen",
    task: "高数基础 第 12 讲 极限的四则运算",
  });
  if (want("focus")) await shoot("overlay", SCREEN_SIZES, "focus");
  if (want("mini")) await shoot("mini", [[280, 88]], "mini");
  if (want("end")) {
    await fetch(`${BASE}/__dev/dev/advance`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ ms: 46 * 60_000 }),
    });
    await new Promise((r) => setTimeout(r, 1500));
    // 正常结束时天色先亮起来，2.4 秒后文字才出现
    await shoot("overlay", SCREEN_SIZES, "end", null, 3600);
  }
  const now = await call("timer_view");
  if (now.value?.phase === "finished") await call("timer_close", { again: false });
  else if (now.value?.phase !== "idle") await call("timer_finish").then(() => call("timer_close", { again: false }));
}

await browser.close();
console.log(`照片在 ${out}`);
