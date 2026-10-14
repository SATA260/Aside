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
  /** 0 表示不设每日目标 */
  daily_goal_ms: number;
  /** 全屏背景：天色、雨窗、台灯、星夜、晨雾 */
  scene: "sky" | "rain" | "lamp" | "stars" | "fog";
  /** 背景音 id，空字符串表示关掉 */
  ambient: string;
  ambient_volume: number;
  exam_on: boolean;
  /** 考试日期，形如 2026-12-21。空着表示还没填 */
  exam_date: string;
  /** 用导入的图片当背景。有值时盖过 scene */
  scene_image: number | null;
  /** 导入声音时自动对齐响度 */
  loudness_match: boolean;
  /** macOS 快捷指令名。空着表示不调用 */
  dnd_on: string;
  dnd_off: string;
  /** Windows 开始专注时还要不要提示 Win+N */
  dnd_hint: boolean;
  /** 已经提示过周报的那个周一 */
  week_prompted: string;
}

export type ShortcutAction = "toggle" | "display";

export interface ImportedSound {
  id: number;
  name: string;
  path: string;
  gain_db: number;
  duration_ms: number;
}

export interface ImportedImage {
  id: number;
  name: string;
  path: string;
}

/** 本地日期，形如 2026-10-14 */
export type DateStr = string;

export interface Segment {
  start: number;
  end: number;
}

export interface MockScore {
  score: number;
  full_score: number;
}

export interface Session {
  id: number;
  subject_id: number;
  mode: Mode;
  planned_ms: number | null;
  started_at: number;
  ended_at: number;
  focused_ms: number;
  task: string;
  note: string;
  source: "timer" | "manual";
  segments: Segment[];
  mock: MockScore | null;
}

export interface DayEntry extends Session {
  subject_name: string;
  subject_color: string;
  /** 落在这一天里的部分 */
  day_ms: number;
}

export interface DayView {
  date: DateStr;
  start: number;
  end: number;
  total_ms: number;
  entries: DayEntry[];
}

export interface SessionPatch {
  subject_id?: number;
  task?: string;
  note?: string;
  started_at?: number;
  ended_at?: number;
}

export interface SubjectMs {
  subject_id: number;
  ms: number;
}

export interface DayTotal {
  date: DateStr;
  /** 这一天 0 点的时刻 */
  start: number;
  total_ms: number;
  subjects: SubjectMs[];
}

export interface Period {
  from: DateStr;
  to: DateStr;
  total_ms: number;
  days_studied: number;
  subjects: SubjectMs[];
}

export interface WeekPrompt {
  from: DateStr;
  to: DateStr;
}

export interface WeekReport {
  week: Period;
  days: DayTotal[];
}

export interface Overview {
  today: DateStr;
  /** 热力图第一天（16 周前的周一）到今天 */
  days: DayTotal[];
  week: Period;
  month: Period;
  balance: string;
}

export interface StartArgs {
  subject_id: number;
  mode: Mode;
  planned_ms: number | null;
  task: string;
  display: DisplayMode;
}
