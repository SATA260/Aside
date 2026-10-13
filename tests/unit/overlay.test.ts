import { describe, expect, it } from "vitest";
import { bigTime, horizon } from "../../src/features/timer/overlay";
import { HOUR, MINUTE } from "../../src/shared/format";
import type { Mode, TimerView } from "../../src/shared/types";

const START = new Date(2026, 9, 13, 21, 2).getTime();

function view(mode: Mode, focused: number, planned: number | null, patch: Partial<TimerView> = {}): TimerView {
  return {
    phase: "running",
    plan: { subject_id: 1, mode, planned_ms: planned, task: "", display: "fullscreen" },
    focused_ms: focused,
    planned_ms: planned,
    remaining_ms: planned === null ? null : planned - focused,
    started_at: START,
    subject: null,
    last: null,
    recovered: null,
    again: false,
    today_ms: 0,
    now: START + focused,
    ...patch,
  };
}

describe("bigTime", () => {
  it("shows the remaining time of a countdown, rounded up", () => {
    expect(bigTime(view("countdown", 12 * MINUTE + 300, 45 * MINUTE), "remaining")).toEqual({
      ms: 33 * MINUTE - 300,
      round: "ceil",
    });
  });

  it("shows the focused time when the user prefers it", () => {
    expect(bigTime(view("countdown", 12 * MINUTE, 45 * MINUTE), "elapsed")).toEqual({ ms: 12 * MINUTE, round: "floor" });
  });

  it("always shows the focused time for count-up", () => {
    expect(bigTime(view("countup", 5 * MINUTE, null), "remaining")).toEqual({ ms: 5 * MINUTE, round: "floor" });
  });
});

describe("horizon", () => {
  it("runs from the start time to the expected end of a countdown", () => {
    const h = horizon(view("countdown", 12 * MINUTE, 45 * MINUTE));
    expect(h.p).toBeCloseTo(12 / 45);
    expect(h.from).toBe(START);
    expect(h.to).toBe(START + 45 * MINUTE);
  });

  it("moves the expected end later after a pause", () => {
    const paused = view("countdown", 12 * MINUTE, 45 * MINUTE, { now: START + 20 * MINUTE, phase: "paused" });
    expect(horizon(paused).to).toBe(START + 53 * MINUTE);
  });

  it("measures count-up against the current hour", () => {
    expect(horizon(view("countup", 20 * MINUTE, null)).p).toBeCloseTo(1 / 3);
    const h = horizon(view("countup", 80 * MINUTE, null));
    expect(h.p).toBeCloseTo(80 / 120);
    expect(h.to).toBe(START + 2 * HOUR);
  });

  it("is full when the time is up", () => {
    expect(horizon(view("countdown", 45 * MINUTE, 45 * MINUTE)).p).toBe(1);
  });
});
