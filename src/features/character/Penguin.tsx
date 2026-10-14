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
import type { CharacterClip, CharacterInfo } from "../../shared/types";
import type { Pose } from "./pose";
import "./penguin.css";

const SHEETS: Record<Pose, string> = { focus, pause, last5, done, blink };

function sheetUrl(pack: string, dir: string, pose: Pose): string {
  if (!pack || pack === "gaga" || !dir) return SHEETS[pose];
  if (!inTauri) return `/__dev/characters/${encodeURIComponent(pack)}/${pose}.png`;
  return convertFileSrc(`${dir}/${pose}.png`);
}

/** 嘎嘎。序列图按帧横排，显示时缩小。导入的形象把文件夹和帧数传进来。 */
export function Penguin({
  pose,
  size = 64,
  pack = "gaga",
  dir = "",
  clip,
}: {
  pose: Pose;
  size?: number;
  pack?: string;
  dir?: string;
  clip?: CharacterClip;
}) {
  const frames = clip?.frames || (pose === "blink" ? 2 : 4);
  const ms = clip?.ms || (pose === "done" || pose === "blink" ? 160 : 180);
  const frameW = Math.round((size * 180) / 220);
  return (
    <i
      className="penguin"
      aria-hidden
      style={{
        width: frameW,
        height: size,
        backgroundImage: `url(${sheetUrl(pack, dir, pose)})`,
        backgroundSize: `${frameW * frames}px ${size}px`,
        animation: `penguin-shift ${frames * ms}ms steps(${frames}) infinite`,
        ["--shift" as string]: `-${frameW * frames}px`,
      }}
    />
  );
}

export function usePack() {
  const id = useApp((s) => s.settings?.character) || "gaga";
  const version = useApp((s) => s.dataVersion);
  const [rows, setRows] = useState<CharacterInfo[]>([]);
  useEffect(() => {
    void api.character.list().then(setRows).catch(() => setRows([]));
  }, [version]);
  const current = rows.find((row) => row.id === id);
  return {
    id,
    dir: current?.dir ?? "",
    clips: current?.clips,
    rows,
    show: id !== "capsule",
  };
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
