import { useEffect, useState } from "react";

/** 鼠标和键盘静止 ms 毫秒后返回 true，一动就变回 false。 */
export function useIdle(ms: number): boolean {
  const [idle, setIdle] = useState(false);
  useEffect(() => {
    let timer = setTimeout(() => setIdle(true), ms);
    const wake = () => {
      setIdle(false);
      clearTimeout(timer);
      timer = setTimeout(() => setIdle(true), ms);
    };
    const events = ["mousemove", "mousedown", "keydown", "wheel"] as const;
    events.forEach((e) => window.addEventListener(e, wake));
    return () => {
      clearTimeout(timer);
      events.forEach((e) => window.removeEventListener(e, wake));
    };
  }, [ms]);
  return idle;
}

/** 防烧屏：每隔 everyMs 把内容整体挪到一个新的随机偏移，最多 range 像素。 */
export function useDrift(everyMs = 3 * 60_000, range = 6): { x: number; y: number } {
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  useEffect(() => {
    const id = setInterval(() => {
      const r = () => Math.round((Math.random() * 2 - 1) * range);
      setOffset({ x: r(), y: r() });
    }, everyMs);
    return () => clearInterval(id);
  }, [everyMs, range]);
  return offset;
}
