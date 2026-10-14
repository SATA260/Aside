import { HOUR, MINUTE } from "../../shared/format";
import type { DisplayMode, Mode, Settings, StartArgs, Subject, TimerView } from "../../shared/types";
import { examLengthMs } from "./mock";

export const COUNTDOWN_PRESETS = [25, 45, 60, 90].map((m) => m * MINUTE);
export const MOCK_PRESETS = [120, 150, 180].map((m) => m * MINUTE);
export const MAX_LENGTH = 24 * HOUR;
export const TASK_MAX = 50;

export interface StartForm {
  subjectId: number;
  mode: Mode;
  lengthMs: number;
  display: DisplayMode;
  task: string;
}

export function initialForm(
  settings: Settings,
  subjects: Subject[],
  timer: TimerView,
  displays: DisplayMode[] = ["fullscreen", "mini", "lecture"],
): StartForm {
  const fallback = subjects.find((s) => s.is_fallback) ?? subjects[0];
  const last = timer.again ? timer.last?.plan.subject_id : undefined;
  const subject = subjects.find((s) => s.id === last) ?? fallback;
  const mode = settings.last_mode;
  return {
    subjectId: subject?.id ?? 0,
    mode,
    lengthMs: mode === "mock" ? settings.last_mock_ms : settings.last_countdown_ms,
    display: displays.includes(settings.last_display) ? settings.last_display : "fullscreen",
    task: "",
  };
}

/** 换方式时，倒计时沿用上次的长度；模考按这个科目的真题时长重新填上。 */
export function switchMode(form: StartForm, mode: Mode, settings: Settings, subjectName = "其他"): StartForm {
  if (mode === form.mode) return form;
  const lengthMs =
    mode === "mock" ? examLengthMs(subjectName) : mode === "countdown" ? settings.last_countdown_ms : form.lengthMs;
  return { ...form, mode, lengthMs };
}

/** 模考里换科目时，时长跟着换成那一科的真题时长。 */
export function pickSubject(form: StartForm, subject: Subject): StartForm {
  if (form.mode !== "mock" || subject.id === form.subjectId) return { ...form, subjectId: subject.id };
  return { ...form, subjectId: subject.id, lengthMs: examLengthMs(subject.name) };
}

export function splitHm(ms: number): { h: number; m: number } {
  const minutes = Math.round(ms / MINUTE);
  return { h: Math.floor(minutes / 60), m: minutes % 60 };
}

export function joinHm(h: number, m: number): number {
  const safe = (n: number) => (Number.isFinite(n) ? Math.max(0, Math.floor(n)) : 0);
  return (safe(h) * 60 + safe(m)) * MINUTE;
}

export function lengthError(ms: number): string | null {
  if (ms < MINUTE) return "至少 1 分钟";
  if (ms > MAX_LENGTH) return "最长 24 小时";
  return null;
}

export function toStartArgs(form: StartForm): StartArgs {
  return {
    subject_id: form.subjectId,
    mode: form.mode,
    planned_ms: form.mode === "countup" ? null : form.lengthMs,
    task: form.task.trim(),
    display: form.display,
  };
}
