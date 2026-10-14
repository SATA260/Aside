import { type CSSProperties, type ReactNode, useEffect, useState } from "react";
import { Penguin, useReduceMotion } from "../../features/character/Penguin";
import { pose } from "../../features/character/pose";
import { SoundChoices } from "../../features/atmosphere/library";
import { bigTime, horizon } from "../../features/timer/overlay";
import { api } from "../../shared/api";
import { clock } from "../../shared/format";
import { play } from "../../shared/sfx";
import { setTimer, useApp } from "../../shared/state";
import type { ThemeName } from "../../shared/types";
import { Dot } from "../../shared/ui/controls";
import { attempt } from "../../shared/ui/toast";
import "./mini.css";

/** 浮窗：置顶的小胶囊，拖动胶囊本身可以挪位置，悬停时出现按钮。 */
export function MiniView() {
  const timer = useApp((s) => s.timer);
  const settings = useApp((s) => s.settings);
  const [confirmEnd, setConfirmEnd] = useState(false);
  const [atmo, setAtmo] = useState(false);
  const reduce = useReduceMotion();
  const paused = timer?.phase === "paused";

  useEffect(() => {
    if (!confirmEnd) return;
    const id = setTimeout(() => setConfirmEnd(false), 3000);
    return () => clearTimeout(id);
  }, [confirmEnd]);

  const togglePause = async () => {
    play(paused ? "resume" : "pause");
    const view = await attempt(paused ? api.timer.resume() : api.timer.pause());
    if (view) setTimer(view);
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.code !== "Space") return;
      e.preventDefault();
      void togglePause();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  if (!timer || !settings || (timer.phase !== "running" && timer.phase !== "paused")) return null;

  const big = bigTime(timer, settings.time_display);
  const name = timer.plan?.mode === "mock" ? "模考" : timer.subject?.name;

  const end = () => {
    if (!confirmEnd) {
      play("tap");
      setConfirmEnd(true);
      return;
    }
    void attempt(api.timer.finish());
  };

  return (
    <div className="mini">
      <div
        className={`pill${paused ? " paused" : ""}${confirmEnd ? " confirming" : ""}`}
        style={{ "--p": horizon(timer).p } as CSSProperties}
        data-tauri-drag-region
      >
        <Dot color={timer.subject?.color} />
        <Penguin pose={pose(timer.phase, timer.remaining_ms, reduce)} size={40} />
        <span className="name wk" data-tauri-drag-region>
          {name}
        </span>
        <span className="t num" data-tauri-drag-region>
          {confirmEnd ? <span className="wk">再点一次结束</span> : clock(big.ms, big.round)}
        </span>
        <span className="ico">
          <IconButton title={paused ? "继续" : "暂停"} onClick={() => void togglePause()}>
            {paused ? <path d="M5 3.5v9l7.5-4.5z" /> : <path d="M5 3.5h2v9H5zM9 3.5h2v9H9z" />}
          </IconButton>
          <IconButton
            title="展开到全屏"
            onClick={() => {
              play("tap");
              void attempt(api.timer.switchDisplay("fullscreen"));
            }}
          >
            <path d="M3 7V3h4M13 9v4H9M3 3l4 4M13 13 9 9" fill="none" stroke="currentColor" strokeWidth="1.4" />
          </IconButton>
          <IconButton title="氛围" className={atmo ? "on" : ""} onClick={() => setAtmo((v) => !v)}>
            <circle cx="8" cy="8" r="3.2" fill="none" stroke="currentColor" strokeWidth="1.4" />
          </IconButton>
          <IconButton title="结束" className={confirmEnd ? "end on" : "end"} onClick={end}>
            <rect x="4" y="4" width="8" height="8" rx="1.5" />
          </IconButton>
        </span>
        <i className="p" data-tauri-drag-region />
      </div>
      {atmo && (
        <div className="mini-atmo wk">
          <SoundChoices
            ambient={settings.ambient}
            onPick={(ambient) => void attempt(api.settings.update({ ambient }))}
          />
          {(["night", "bamboo", "mist", "dusk", "sky"] as ThemeName[]).map((theme) => (
            <button
              type="button"
              key={theme}
              className={settings.theme === theme ? "on" : ""}
              onClick={() => void attempt(api.settings.update({ theme }))}
            >
              {theme === "night" ? "夜读" : theme === "bamboo" ? "竹影" : theme === "mist" ? "晨雾" : theme === "dusk" ? "暮色" : "天色"}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

function IconButton({
  title,
  className,
  onClick,
  children,
}: {
  title: string;
  className?: string;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <button type="button" title={title} aria-label={title} className={className} onClick={onClick}>
      <svg viewBox="0 0 16 16" width="14" height="14" fill="currentColor" aria-hidden>
        {children}
      </svg>
    </button>
  );
}
