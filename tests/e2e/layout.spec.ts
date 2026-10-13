import { type APIRequestContext, expect, type Page, test } from "@playwright/test";

async function call<T = unknown>(request: APIRequestContext, cmd: string, args: object = {}): Promise<T> {
  const res = await request.post("/__dev/call", { data: { cmd, args } });
  const body = await res.json();
  if (!body.ok) throw new Error(`${cmd}: ${body.error}`);
  return body.value as T;
}

async function resetTimer(request: APIRequestContext) {
  const view = await call<{ phase: string }>(request, "timer_view");
  if (view.phase === "running" || view.phase === "paused") await call(request, "timer_finish");
  if (view.phase !== "idle") await call(request, "timer_close", { again: false });
}

async function startMath(request: APIRequestContext) {
  const subjects = await call<{ id: number; name: string }[]>(request, "subjects_list");
  await call(request, "timer_start", {
    subject_id: subjects.find((s) => s.name === "数学")!.id,
    mode: "countdown",
    planned_ms: 45 * 60_000,
    display: "fullscreen",
    task: "高数基础 第 12 讲 极限的四则运算，顺便把课后习题一到十五题都做完再对答案",
  });
}

const box = async (page: Page, selector: string) => {
  const b = await page.locator(selector).first().boundingBox();
  if (!b) throw new Error(`${selector} 不在页面上`);
  return b;
};

async function openMain(page: Page, nav?: string) {
  await page.goto("/?view=main");
  await page.locator(".go .btn.primary").waitFor();
  if (nav) await page.getByRole("button", { name: nav, exact: true }).click();
}

test.beforeEach(async ({ request }) => resetTimer(request));

test.describe("主窗口", () => {
  test("最小窗口里开始按钮不用滚动就能点到", async ({ page }) => {
    await page.setViewportSize({ width: 880, height: 600 });
    await openMain(page);
    const go = await box(page, ".go .btn.primary");
    expect(go.y + go.height).toBeLessThanOrEqual(600);
    await expect(page.locator(".page")).toHaveJSProperty("scrollLeft", 0);
  });

  test("大窗口里开始页的表单在侧栏右边居中，不挤在左上角", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1050 });
    await openMain(page);
    const [left, right] = await page
      .locator(".start-page")
      .evaluate((e) => [getComputedStyle(e).paddingLeft, getComputedStyle(e).paddingRight].map(Number.parseFloat));
    expect(left).toBeGreaterThan(300);
    expect(Math.abs(left - right)).toBeLessThan(2);
    const area = await box(page, ".main-content");
    const hello = await box(page, ".hello");
    const go = await box(page, ".go");
    expect(hello.y - area.y).toBeGreaterThan(150);
    expect(area.y + area.height - (go.y + go.height)).toBeGreaterThan(150);
  });

  test("设置在宽窗口分两栏，窄一些是一栏", async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1050 });
    await openMain(page, "设置");
    const groups = page.locator(".settings-group");
    const first = (await groups.first().boundingBox())!;
    const last = (await groups.last().boundingBox())!;
    expect(last.x).toBeGreaterThan(first.x + first.width);

    await page.setViewportSize({ width: 1440, height: 900 });
    const first2 = (await groups.first().boundingBox())!;
    const last2 = (await groups.last().boundingBox())!;
    expect(last2.x).toBeCloseTo(first2.x, 0);
  });

  for (const [w, h] of [
    [880, 600],
    [1440, 900],
    [1920, 1050],
  ]) {
    test(`${w}×${h} 下风格名都在一行、页面不横向滚动`, async ({ page }) => {
      await page.setViewportSize({ width: w, height: h });
      await openMain(page, "设置");
      const heights = await page.locator(".theme-pick").evaluateAll((els) => els.map((e) => e.clientHeight));
      expect(new Set(heights).size).toBe(1);
      for (const name of ["开始", "科目", "设置"]) {
        await page.getByRole("button", { name, exact: true }).click();
        const overflow = await page.locator(".page").evaluate((e) => e.scrollWidth - e.clientWidth);
        expect(overflow, name).toBeLessThanOrEqual(0);
      }
    });
  }
});

test.describe("全屏页", () => {
  const SCREENS = [
    [1024, 640],
    [1280, 720],
    [2560, 1440],
    [1080, 1920],
  ] as const;

  for (const [w, h] of SCREENS) {
    test(`${w}×${h} 下各部分都在屏幕里、互不重叠`, async ({ page, request }) => {
      await startMath(request);
      await page.setViewportSize({ width: w, height: h });
      await page.goto("/?view=overlay");
      await page.locator(".ov-time").waitFor();

      const top = await box(page, ".ov-top");
      const time = await box(page, ".ov-time");
      const note = await box(page, ".ov-note");
      const ctl = await box(page, ".textctl");
      const bottom = await box(page, ".ov-bottom");
      expect(top.y).toBeGreaterThanOrEqual(0);
      expect(top.x + top.width).toBeLessThanOrEqual(w);
      expect(top.y + top.height).toBeLessThan(time.y);
      expect(time.x).toBeGreaterThanOrEqual(0);
      expect(time.x + time.width).toBeLessThanOrEqual(w);
      expect(note.y + note.height).toBeLessThan(ctl.y);
      expect(ctl.y + ctl.height).toBeLessThan(bottom.y);
      expect(bottom.y + bottom.height).toBeLessThanOrEqual(h);
    });
  }

  test("屏幕越大字越大，远处也看得清", async ({ page, request }) => {
    await startMath(request);
    const sizes: Record<string, number[]> = {};
    for (const [w, h] of [
      [1280, 720],
      [2560, 1440],
    ]) {
      await page.setViewportSize({ width: w, height: h });
      await page.goto("/?view=overlay");
      await page.locator(".ov-time").waitFor();
      sizes[w] = await page.evaluate(() =>
        [".ov-time", ".ov-subject", ".textctl button"].map((s) =>
          Number.parseFloat(getComputedStyle(document.querySelector(s)!).fontSize),
        ),
      );
    }
    sizes[2560].forEach((big, i) => expect(big / sizes[1280][i]).toBeGreaterThan(1.5));
  });

  test("竖屏上结束页的字和备注框都不出屏幕", async ({ page, request }) => {
    await startMath(request);
    await request.post("/__dev/dev/advance", { data: { ms: 10 * 60_000 } });
    await call(request, "timer_finish");
    await page.setViewportSize({ width: 1080, height: 1920 });
    await page.goto("/?view=overlay");
    await page.locator(".end-title").waitFor();
    for (const s of [".end-title", ".end-sub", ".end-actions", ".note-box"]) {
      const b = await box(page, s);
      expect(b.x, s).toBeGreaterThanOrEqual(0);
      expect(b.x + b.width, s).toBeLessThanOrEqual(1080);
    }
  });
});
