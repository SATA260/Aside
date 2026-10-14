import { describe, expect, it } from "vitest";
import {
  COUNTDOWN_PRESETS,
  initialForm,
  joinHm,
  lengthError,
  splitHm,
  switchMode,
  toStartArgs,
} from "../../src/features/timer/startForm";
import { HOUR, MINUTE } from "../../src/shared/format";
import type { Outcome, Settings, Subject, TimerView } from "../../src/shared/types";

const subjects: Subject[] = [
  { id: 1, name: "政治", color: "#C98B7B", sort: 0, archived: false, is_fallback: false },
  { id: 3, name: "数学", color: "#8DB59A", sort: 1, archived: false, is_fallback: false },
  { id: 5, name: "其他", color: "#A3A9B4", sort: 2, archived: false, is_fallback: true },
];

const settings: Settings = {
  theme: "night",
  sfx_enabled: true,
  sfx_volume: 0.4,
  time_display: "remaining",
  last_mode: "countdown",
  last_countdown_ms: 45 * MINUTE,
  last_mock_ms: 180 * MINUTE,
  last_display: "fullscreen",
  daily_goal_ms: 0,
};

function view(patch: Partial<TimerView> = {}): TimerView {
  return {
    phase: "idle",
    plan: null,
    focused_ms: 0,
    planned_ms: null,
    remaining_ms: null,
    started_at: null,
    subject: null,
    last: null,
    recovered: null,
    again: false,
    today_ms: 0,
    now: 0,
    ...patch,
  };
}

function outcome(subjectId: number): Outcome {
  return {
    session_id: 1,
    recorded: true,
    focused_ms: 30 * MINUTE,
    reason: "manual",
    plan: { subject_id: subjectId, mode: "countdown", planned_ms: 45 * MINUTE, task: "", display: "fullscreen" },
    started_at: 0,
    ended_at: 0,
  };
}

describe("initialForm", () => {
  it("defaults to 其他 and the last mode, length and display", () => {
    const f = initialForm(settings, subjects, view());
    expect(f).toMatchObject({ subjectId: 5, mode: "countdown", lengthMs: 45 * MINUTE, display: "fullscreen", task: "" });
  });

  it("keeps the last subject after 再来一段", () => {
    const f = initialForm(settings, subjects, view({ again: true, last: outcome(3) }));
    expect(f.subjectId).toBe(3);
  });

  it("falls back to 其他 when the last subject was archived since", () => {
    const f = initialForm(settings, subjects, view({ again: true, last: outcome(2) }));
    expect(f.subjectId).toBe(5);
  });

  it("does not keep the last subject without 再来一段", () => {
    const f = initialForm(settings, subjects, view({ again: false, last: outcome(3) }));
    expect(f.subjectId).toBe(5);
  });

  it("uses the mock length for mock exams", () => {
    const f = initialForm({ ...settings, last_mode: "mock" }, subjects, view());
    expect(f.lengthMs).toBe(180 * MINUTE);
  });

  it("falls back to fullscreen when the remembered display is not available", () => {
    const f = initialForm({ ...settings, last_display: "lecture" }, subjects, view(), ["fullscreen", "mini"]);
    expect(f.display).toBe("fullscreen");
  });
});

describe("switchMode", () => {
  const base = initialForm(settings, subjects, view());

  it("uses the remembered mock length when switching to mock", () => {
    expect(switchMode(base, "mock", settings)).toMatchObject({ mode: "mock", lengthMs: 180 * MINUTE });
  });

  it("uses the remembered countdown length when switching back", () => {
    const mock = switchMode(base, "mock", settings);
    expect(switchMode(mock, "countdown", settings)).toMatchObject({ mode: "countdown", lengthMs: 45 * MINUTE });
  });

  it("keeps the other choices", () => {
    const f = switchMode({ ...base, subjectId: 3, task: "真题" }, "countup", settings);
    expect(f).toMatchObject({ mode: "countup", subjectId: 3, task: "真题" });
  });
});

describe("hours and minutes", () => {
  it("splits and joins", () => {
    expect(splitHm(150 * MINUTE)).toEqual({ h: 2, m: 30 });
    expect(joinHm(2, 30)).toBe(150 * MINUTE);
  });

  it("treats empty or broken input as zero", () => {
    expect(joinHm(Number.NaN, 5)).toBe(5 * MINUTE);
  });
});

describe("lengthError", () => {
  it("needs at least one minute", () => {
    expect(lengthError(0)).toBe("至少 1 分钟");
    expect(lengthError(MINUTE)).toBeNull();
  });

  it("allows up to exactly 24 hours", () => {
    expect(lengthError(24 * HOUR)).toBeNull();
    expect(lengthError(24 * HOUR + MINUTE)).toBe("最长 24 小时");
  });
});

describe("toStartArgs", () => {
  const base = initialForm(settings, subjects, view());

  it("sends the planned length for countdown and mock", () => {
    expect(toStartArgs({ ...base, task: "  高数第 12 讲  " })).toEqual({
      subject_id: 5,
      mode: "countdown",
      planned_ms: 45 * MINUTE,
      task: "高数第 12 讲",
      display: "fullscreen",
    });
  });

  it("sends no length for count-up", () => {
    expect(toStartArgs({ ...base, mode: "countup" }).planned_ms).toBeNull();
  });
});

it("offers the usual countdown presets", () => {
  expect(COUNTDOWN_PRESETS.map((ms) => ms / MINUTE)).toEqual([25, 45, 60, 90]);
});
