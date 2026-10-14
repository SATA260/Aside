// 总览图用到的纯计算：热力图深浅、柱子高度、坐标最大值。

export const HEAT_ALPHA = [0, 0.32, 0.52, 0.74, 1] as const;

/** 0 是没学，1 到 4 越来越深。没学的日子不涂颜色。 */
export function heatLevel(ms: number, peak: number): 0 | 1 | 2 | 3 | 4 {
  if (ms <= 0 || peak <= 0) return 0;
  const ratio = ms / peak;
  if (ratio <= 0.25) return 1;
  if (ratio <= 0.5) return 2;
  if (ratio <= 0.75) return 3;
  return 4;
}

/** 柱子占坐标轴的百分比。坐标轴至少能放下每日目标。 */
export function axisMax(totals: number[], goalMs: number): number {
  return Math.max(1, goalMs, ...totals);
}

export function barPercent(ms: number, max: number): number {
  if (ms <= 0 || max <= 0) return 0;
  return (ms / max) * 100;
}
