import type { Phase } from "../../shared/types";

export type Pose = "focus" | "pause" | "last5" | "done" | "blink";

const FIVE_MIN = 5 * 60 * 1000;

/** 专注中看书，暂停伸懒腰，最后五分钟看钟，结束扑腾。减少动态时只眨眼。 */
export function pose(phase: Phase, remainingMs: number | null, reduceMotion: boolean): Pose {
  if (reduceMotion) return "blink";
  if (phase === "finished") return "done";
  if (phase === "paused") return "pause";
  if (remainingMs !== null && remainingMs <= FIVE_MIN && remainingMs > 0) return "last5";
  return "focus";
}
