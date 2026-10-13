// 和 Rust 那边序列化出来的结构一一对应。

export type Phase = "idle" | "running" | "paused" | "finished";
export type Mode = "countdown" | "countup" | "mock";
export type DisplayMode = "fullscreen" | "mini" | "lecture";
export type ThemeName = "night" | "bamboo" | "mist" | "dusk" | "sky";
export type TimeDisplay = "remaining" | "elapsed";
export type EndReason = "completed" | "manual" | "recovered";

export interface Plan {
  subject_id: number;
  mode: Mode;
  planned_ms: number | null;
  task: string;
  display: DisplayMode;
}

export interface Subject {
  id: number;
  name: string;
  color: string;
  sort: number;
  archived: boolean;
  is_fallback: boolean;
}

export interface Outcome {
  session_id: number | null;
  recorded: boolean;
  focused_ms: number;
  reason: EndReason;
  plan: Plan;
  started_at: number;
  ended_at: number;
}

export interface TimerView {
  phase: Phase;
  plan: Plan | null;
  focused_ms: number;
  planned_ms: number | null;
  remaining_ms: number | null;
  started_at: number | null;
  subject: Subject | null;
  last: Outcome | null;
  recovered: Outcome | null;
  again: boolean;
  today_ms: number;
  now: number;
}

export interface Settings {
  theme: ThemeName;
  sfx_enabled: boolean;
  sfx_volume: number;
  time_display: TimeDisplay;
  last_mode: Mode;
  last_countdown_ms: number;
  last_mock_ms: number;
  last_display: DisplayMode;
  last_subject_id: number | null;
  mini_pos: { x: number; y: number } | null;
  shortcut_toggle: string;
  shortcut_display: string;
}

export type ShortcutAction = "toggle" | "display";

export interface StartArgs {
  subject_id: number;
  mode: Mode;
  planned_ms: number | null;
  task: string;
  display: DisplayMode;
}
