import { type APIRequestContext, expect, test } from "@playwright/test";

async function call<T = unknown>(request: APIRequestContext, cmd: string, args: object = {}): Promise<T> {
  const res = await request.post("/__dev/call", { data: { cmd, args } });
  const body = await res.json();
  if (!body.ok) throw new Error(`${cmd}: ${body.error}`);
  return body.value as T;
}

test.beforeEach(async ({ request }) => {
  const view = await call<{ phase: string }>(request, "timer_view");
  if (view.phase === "running" || view.phase === "paused") await call(request, "timer_finish");
  if (view.phase !== "idle") await call(request, "timer_close", { again: false });
});

test("开始一段，结束并加备注，在记录页看到这一段", async ({ page, request }) => {
  const task = "二重积分 第三章习题";
  const note = "第 3 题没想通，明天问";

  await page.goto("/?view=main");
  await page.locator(".go .btn.primary").waitFor();
  await page.getByRole("button", { name: "数学" }).click();
  await page.getByPlaceholder("比如：高数基础").fill(task);
  await page.getByRole("button", { name: "开始专注" }).click();

  await page.waitForURL(/view=overlay/);
  await page.locator(".ov-time").waitFor();
  await page.mouse.move(240, 240);
  await request.post("/__dev/dev/advance", { data: { ms: 2 * 60_000 } });
  await page.getByRole("button", { name: "结束", exact: true }).click();
  await page.getByRole("button", { name: "再点一次结束" }).click();

  await expect(page.locator(".end-title")).toContainText("已经记下了");
  await page.getByPlaceholder("加个备注").fill(note);
  await page.getByRole("button", { name: "记下", exact: true }).click();
  await page.getByRole("button", { name: "回到电脑" }).click();

  await page.waitForURL(/view=main/);
  await page.getByRole("button", { name: "记录", exact: true }).click();
  const row = page.locator(".sess", { hasText: task });
  await expect(row).toBeVisible();
  await expect(row).toContainText("数学");
  await expect(row.locator(".note")).toHaveText(note);
});

test("模考结束时记下分数，记录页显示模考和分数", async ({ page, request }) => {
  await page.goto("/?view=main");
  await page.locator(".go .btn.primary").waitFor();
  await page.getByRole("radio", { name: "模考" }).click();
  await page.getByRole("button", { name: "数学" }).click();
  await page.getByPlaceholder("比如：数学一").fill("2023 数学一");
  await page.getByRole("button", { name: "开始专注" }).click();

  await page.waitForURL(/view=overlay/);
  await page.locator(".ov-time").waitFor();
  await page.mouse.move(240, 240);
  await request.post("/__dev/dev/advance", { data: { ms: 2 * 60_000 } });
  await page.getByRole("button", { name: "结束", exact: true }).click();
  await page.getByRole("button", { name: "再点一次结束" }).click();

  await page.getByLabel("分数").fill("120");
  await expect(page.getByLabel("满分")).toHaveValue("150");
  await page.getByRole("button", { name: "记下分数" }).click();
  await expect(page.getByRole("button", { name: "已记下" })).toBeVisible();
  await page.getByRole("button", { name: "回到电脑" }).click();

  await page.waitForURL(/view=main/);
  await page.getByRole("button", { name: "记录", exact: true }).click();
  const row = page.locator(".sess", { hasText: "2023 数学一" });
  await expect(row).toContainText("模考");
  await expect(row).toContainText("120/150");
});
