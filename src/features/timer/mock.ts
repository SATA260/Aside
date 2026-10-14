// 考研真题的默认时长和满分。四科现在都是 180 分钟，满分按科目不同。

import { MINUTE } from "../../shared/format";

export const EXAM_MS = 180 * MINUTE;

export function examLengthMs(_name: string): number {
  return EXAM_MS;
}

export function fullScore(name: string): number {
  if (name === "政治" || name === "英语") return 100;
  return 150;
}

export function scoreError(score: number, full: number): string | null {
  if (!Number.isInteger(full) || full <= 0) return "满分要大于 0";
  if (!Number.isInteger(score) || score < 0 || score > full) return `分数要在 0 到 ${full} 之间`;
  return null;
}
