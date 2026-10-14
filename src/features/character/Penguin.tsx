import { convertFileSrc } from "@tauri-apps/api/core";
import blink from "../../assets/characters/gaga/blink.png";
import done from "../../assets/characters/gaga/done.png";
import focus from "../../assets/characters/gaga/focus.png";
import last5 from "../../assets/characters/gaga/last5.png";
import pause from "../../assets/characters/gaga/pause.png";
import { useEffect, useState } from "react";
import { api } from "../../shared/api";
import { inTauri } from "../../shared/ipc";
import { useApp } from "../../shared/state";
import type { Pose } from "./pose";
import "./penguin.css";

const SHEETS: Record<Pose, string> = { focus, pause, last5, done, blink };

function sheetUrl(pack: string, dir: string, pose: Pose): string {
  if (!pack || pack === "gaga" || !dir) return SHEETS[pose];
  if (!inTauri) return `/__dev/characters/${encodeURIComponent(pack)}/${pose}.png`;
  return convertFileSrc(`${dir}/${pose}.png`);
}

/** 嘎嘎。序列图按帧横排，显示时缩小。导入的形象把文件夹传进来。 */
export function Penguin({ pose, size = 64, pack = "gaga", dir = "" }: { pose: Pose; size?: number; pack?: string; dir?: string }) {
  const frames = pose === "blink" ? 2 : 4;
  const frameW = Math.round((size * 180) / 220);
  return (
    <i
      className={`penguin frames-${frames}`}
      aria-hidden
      style={{
        width: frameW,
        height: size,
        backgroundImage: `url(${sheetUrl(pack, dir, pose)})`,
        backgroundSize: `${frameW * frames}px ${size}px`,
        ["--shift" as string]: `-${frameW * frames}px`,
      }}
    />
  );
}

export function usePack() {
  const id = useApp((s) => s.settings?.character) || "gaga";
  const version = useApp((s) => s.dataVersion);
  const [dir, setDir] = useState("");
  useEffect(() => {
    if (id === "gaga" || id === "capsule") {
      setDir("");
      return;
    }
    void api.character.list().then((rows) => setDir(rows.find((row) => row.id === id)?.dir ?? ""));
  }, [id, version]);
  return { id, dir, show: id !== "capsule" };
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
