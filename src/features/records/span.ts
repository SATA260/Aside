// 把“日期 + 开始时刻 + 时长”换成一条记录的起止时刻。时长按整分钟计。

import type { DateStr } from "../../shared/types";
import { parseDate } from "./dates";

const MINUTE = 60_000;
export const MIN_MINUTES = 1;
export const MAX_MINUTES = 24 * 60;

export function atLocal(date: DateStr, time: string): number | null {
  const match = /^(\d{1,2}):(\d{2})(?::\d{2})?$/.exec(time.trim());
  if (!match) return null;
  const h = Number(match[1]);
  const m = Number(match[2]);
  if (h > 23 || m > 59) return null;
  const d = parseDate(date);
  d.setHours(h, m, 0, 0);
  return d.getTime();
}

/** 开始时刻加上整数分钟。分钟不在 1 到 24 小时之间时返回 null。 */
export function rangeFrom(
  date: DateStr,
  time: string,
  minutes: number,
): { started_at: number; ended_at: number } | null {
  const started_at = atLocal(date, time);
  if (started_at == null || !Number.isInteger(minutes)) return null;
  if (minutes < MIN_MINUTES || minutes > MAX_MINUTES) return null;
  return { started_at, ended_at: started_at + minutes * MINUTE };
}

export function wallMinutes(started_at: number, ended_at: number): number {
  return Math.max(0, Math.round((ended_at - started_at) / MINUTE));
}
