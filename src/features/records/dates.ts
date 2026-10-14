// 记录页用到的本地日期，统一用 "2026-10-14" 这样的字符串传来传去。

import type { DateStr } from "../../shared/types";

const WEEKDAYS = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

const pad = (n: number) => String(n).padStart(2, "0");

export function toDateStr(d: Date): DateStr {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

/** 这一天的本地 0 点。 */
export function parseDate(s: DateStr): Date {
  const [y, m, d] = s.split("-").map(Number);
  return new Date(y, m - 1, d);
}

export function addDays(s: DateStr, n: number): DateStr {
  const d = parseDate(s);
  d.setDate(d.getDate() + n);
  return toDateStr(d);
}

export function dayTitle(s: DateStr): string {
  const d = parseDate(s);
  return `${d.getMonth() + 1} 月 ${d.getDate()} 日`;
}

export function weekday(s: DateStr): string {
  return WEEKDAYS[parseDate(s).getDay()];
}

/** “周二 · 今天”“周一 · 昨天”，更早的只写周几。 */
export function dayLabel(s: DateStr, today: DateStr): string {
  if (s === today) return `${weekday(s)} · 今天`;
  if (s === addDays(today, -1)) return `${weekday(s)} · 昨天`;
  return weekday(s);
}

/** 一个月按周排开，每周从周一开始，前后不足的格子是 null。`month` 从 1 开始。 */
export function monthGrid(year: number, month: number): (DateStr | null)[] {
  const first = new Date(year, month - 1, 1);
  const days = new Date(year, month, 0).getDate();
  const lead = (first.getDay() + 6) % 7;
  const cells: (DateStr | null)[] = Array(lead).fill(null);
  for (let d = 1; d <= days; d++) cells.push(toDateStr(new Date(year, month - 1, d)));
  while (cells.length % 7) cells.push(null);
  return cells;
}
