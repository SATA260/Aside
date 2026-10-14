// 某一天 0 点到 24 点的时间轴：每个运行段画成一块，位置和宽度按占这一天的比例算。

export interface TimelineEntry {
  id: number;
  subject_color: string;
  segments: { start: number; end: number }[];
}

export interface Block {
  id: number;
  color: string;
  /** 百分比 */
  left: number;
  width: number;
}

export function blocks(entries: TimelineEntry[], dayStart: number, dayEnd: number): Block[] {
  const span = dayEnd - dayStart;
  return entries.flatMap((e) =>
    e.segments.flatMap((g) => {
      const start = Math.max(g.start, dayStart);
      const end = Math.min(g.end, dayEnd);
      if (end <= start) return [];
      return [{ id: e.id, color: e.subject_color, left: ((start - dayStart) * 100) / span, width: ((end - start) * 100) / span }];
    }),
  );
}

export function crossesMidnight(
  entry: { started_at: number; ended_at: number },
  dayStart: number,
  dayEnd: number,
): boolean {
  return entry.started_at < dayStart || entry.ended_at > dayEnd;
}
