export function moveItem<T>(items: T[], from: number, to: number): T[] {
  const next = items.slice();
  const [item] = next.splice(from, 1);
  next.splice(to, 0, item);
  return next;
}

/** 拖动 dy 像素之后落在第几行。 */
export function targetIndex(from: number, count: number, dy: number, rowHeight: number): number {
  return Math.min(count - 1, Math.max(0, from + Math.round(dy / rowHeight)));
}
