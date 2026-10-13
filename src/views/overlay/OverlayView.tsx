import { useEffect } from "react";
import { on } from "../../shared/ipc";
import { play } from "../../shared/sfx";
import { useApp } from "../../shared/state";
import { EndScreen } from "./EndScreen";
import { FocusScreen } from "./FocusScreen";
import "./overlay.css";

export function OverlayView() {
  const timer = useApp((s) => s.timer);
  const settings = useApp((s) => s.settings);

  useEffect(() => on("timer:finished", () => play("end")), []);

  const ended = timer?.phase === "finished" && timer.last;
  return (
    <div className={ended ? `overlay ended ${timer.last?.reason}` : "overlay"}>
      <div className="sky" />
      {timer && settings && (timer.phase === "running" || timer.phase === "paused") && (
        <FocusScreen timer={timer} settings={settings} />
      )}
      {ended && timer.last && <EndScreen outcome={timer.last} subject={timer.subject} />}
    </div>
  );
}
