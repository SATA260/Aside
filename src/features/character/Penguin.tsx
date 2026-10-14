import blink from "../../assets/characters/gaga/blink.png";
import done from "../../assets/characters/gaga/done.png";
import focus from "../../assets/characters/gaga/focus.png";
import last5 from "../../assets/characters/gaga/last5.png";
import pause from "../../assets/characters/gaga/pause.png";
import { useEffect, useState } from "react";
import type { Pose } from "./pose";
import "./penguin.css";

const SHEETS: Record<Pose, string> = { focus, pause, last5, done, blink };

/** 嘎嘎。序列图按帧横排，显示时缩小。 */
export function Penguin({ pose, size = 64 }: { pose: Pose; size?: number }) {
  const frames = pose === "blink" ? 2 : 4;
  const frameW = Math.round((size * 180) / 220);
  return (
    <i
      className={`penguin frames-${frames}`}
      aria-hidden
      style={{
        width: frameW,
        height: size,
        backgroundImage: `url(${SHEETS[pose]})`,
        backgroundSize: `${frameW * frames}px ${size}px`,
        ["--shift" as string]: `-${frameW * frames}px`,
      }}
    />
  );
}

export function useReduceMotion() {
  const [on, setOn] = useState(false);
  useEffect(() => {
    const media = window.matchMedia("(prefers-reduced-motion: reduce)");
    const apply = () => setOn(media.matches);
    apply();
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, []);
  return on;
}
