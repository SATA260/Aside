const S = 1000;
const M = 60 * S;
const H = 60 * M;

const pad = (n: number) => String(n).padStart(2, "0");

/** 计时数字：不到 1 小时显示“分:秒”，1 小时及以上显示“时:分:秒”。倒计时用 ceil，到 0 时正好显示 00:00。 */
export function clock(ms: number, round: "floor" | "ceil" = "floor"): string {
  const total = Math.max(0, round === "ceil" ? Math.ceil(ms / S) : Math.floor(ms / S));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${pad(m)}:${pad(s)}`;
}

export interface DurationPart {
  n: number;
  unit: string;
}

/** 结束页、标题里的时长：“45 分钟”“1 小时 30 分”“2 小时”。 */
export function durationParts(ms: number): DurationPart[] {
  const minutes = Math.floor(Math.max(0, ms) / M);
  if (minutes < 60) return [{ n: minutes, unit: "分钟" }];
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  return m ? [{ n: h, unit: "小时" }, { n: m, unit: "分" }] : [{ n: h, unit: "小时" }];
}

export function durationText(ms: number): string {
  return durationParts(ms)
    .map((p) => `${p.n} ${p.unit}`)
    .join(" ");
}

/** 列表里的紧凑时长：“45 分”“2 时 30 分”。 */
export function durationShort(ms: number): string {
  const minutes = Math.floor(Math.max(0, ms) / M);
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  if (!h) return `${m} 分`;
  return m ? `${h} 时 ${m} 分` : `${h} 时`;
}

/** 本地时刻 HH:MM。 */
export function hm(ts: number): string {
  const d = new Date(ts);
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export const MINUTE = M;
export const HOUR = H;
