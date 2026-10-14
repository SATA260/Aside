import { HOUR } from "../../shared/format";
import type { TimeDisplay, TimerView } from "../../shared/types";

/** 大号时间显示哪个数：倒计时默认显示剩余（向上取整，到 0 时正好 00:00），也可以改成始终显示已学。 */
export function bigTime(view: TimerView, display: TimeDisplay): { ms: number; round: "floor" | "ceil" } {
  if (view.remaining_ms !== null && display === "remaining") return { ms: view.remaining_ms, round: "ceil" };
  return { ms: view.focused_ms, round: "floor" };
}

/**
 * 天际线：p 是已经亮起的比例，from / to 是两端标的时刻。
 * 倒计时和模考按计划时长算，暂停后终点会顺延；正计时按“当前这一个小时”算，满一小时换到下一格。
 */
export function horizon(view: TimerView): { p: number; from: number; to: number } {
  const from = view.started_at ?? view.now;
  const span = view.planned_ms ?? (Math.floor(view.focused_ms / HOUR) + 1) * HOUR;
  const p = span > 0 ? Math.min(1, view.focused_ms / span) : 0;
  return { p, from, to: view.now + Math.max(0, span - view.focused_ms) };
}

/** 氛围面板打开时，画面和声音要留在可点的状态，不能跟控制条一起淡成点不了。 */
export function focusChrome(idle: boolean, atmo: boolean): string {
  return ["focus", atmo && "atmo-open", idle && !atmo && "idle"].filter(Boolean).join(" ");
}
