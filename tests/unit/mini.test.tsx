import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { EndScreen } from "../../src/views/overlay/EndScreen";
import { MiniView } from "../../src/views/mini/MiniView";
import { useApp } from "../../src/shared/state";
import type { Outcome, Settings, TimerView } from "../../src/shared/types";

function media(matches: boolean) {
  return {
    matches,
    media: "(prefers-reduced-motion: reduce)",
    addEventListener: () => {},
    removeEventListener: () => {},
    addListener: () => {},
    removeListener: () => {},
    dispatchEvent: () => false,
  } as unknown as MediaQueryList;
}

const settings = {
  theme: "night",
  time_display: "remaining",
  character: "gaga",
  ambient: "",
} as Settings;

function running(): TimerView {
  return {
    phase: "running",
    plan: { subject_id: 1, mode: "countdown", planned_ms: 45 * 60_000, task: "", display: "mini" },
    focused_ms: 12 * 60_000,
    planned_ms: 45 * 60_000,
    remaining_ms: 32 * 60_000 + 48 * 1000,
    started_at: 0,
    subject: { id: 1, name: "数学", color: "#8DB59A", sort: 0, archived: false, is_fallback: false },
    last: null,
    recovered: null,
    again: false,
    today_ms: 0,
    now: 1,
  };
}

beforeEach(() => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => ({ ok: true, json: async () => ({ ok: true, value: [], next: 0, events: [] }) })),
  );
  vi.stubGlobal("matchMedia", () => media(false));
  useApp.setState({ timer: running(), settings, subjects: [], dataVersion: 0 });
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  useApp.setState({ timer: null, settings: null, subjects: [], dataVersion: 0 });
});

it("keeps the time on the character window and can still open the full screen", () => {
  const { container } = render(<MiniView />);
  const time = screen.getByText("32:48");
  expect(time.closest(".pet-bubble")).toBeTruthy();
  expect(screen.getByRole("button", { name: "展开到全屏", hidden: true })).toBeTruthy();
  expect(container.querySelector("[data-tauri-drag-region]")).toBeTruthy();
  expect(container.querySelector(".pet-bubble .p")).toBeTruthy();
});

it("blinks instead of flapping when motion is reduced", async () => {
  vi.stubGlobal("matchMedia", () => media(true));
  const outcome: Outcome = {
    session_id: 1,
    recorded: true,
    focused_ms: 45 * 60_000,
    reason: "completed",
    plan: { subject_id: 1, mode: "countdown", planned_ms: 45 * 60_000, task: "", display: "mini" },
    started_at: 0,
    ended_at: 1,
  };
  const { container } = render(<EndScreen outcome={outcome} subject={null} />);
  await waitFor(() => {
    const image = container.querySelector(".penguin")?.getAttribute("style") ?? "";
    expect(image).toContain("blink");
  });
});
