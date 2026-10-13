import { type CSSProperties, Fragment, useEffect, useState } from "react";
import { bigTime, horizon } from "../../features/timer/overlay";
import { api } from "../../shared/api";
import { clock, durationText, hm, MINUTE } from "../../shared/format";
import { useDrift, useIdle } from "../../shared/hooks";
import { play } from "../../shared/sfx";
import { setTimer } from "../../shared/state";
import type { Settings, TimerView } from "../../shared/types";
import { Dot } from "../../shared/ui/controls";
import { attempt } from "../../shared/ui/toast";

export function FocusScreen({ timer, settings }: { timer: TimerView; settings: Settings }) {
  const idle = useIdle(3000);
  const drift = useDrift();
  const paused = timer.phase === "paused";
  const [confirmEnd, setConfirmEnd] = useState(false);

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

  const end = () => {
    if (!confirmEnd) {
      play("tap");
      setConfirmEnd(true);
      return;
    }
    void attempt(api.timer.finish());
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.code === "Space" && !(e.target instanceof HTMLInputElement)) {
        e.preventDefault();
        void togglePause();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  const big = bigTime(timer, settings.time_display);
  const line = horizon(timer);
  const canFlip = timer.remaining_ms !== null;
  const verb = timer.plan?.mode === "mock" ? "已作答" : "已学";
  const learned = timer.focused_ms < MINUTE ? "刚开始" : `${verb} ${durationText(timer.focused_ms)}`;

  return (
    <div
      className={idle ? "focus idle" : "focus"}
      style={{ transform: `translate(${drift.x}px, ${drift.y}px)` }}
    >
      <header className="ov-top">
        <div className="ov-subject wk">
          <Dot color={timer.subject?.color} />
          <span>{timer.plan?.mode === "mock" ? "模考" : timer.subject?.name}</span>
          {timer.plan?.task && <span className="task">{timer.plan.task}</span>}
        </div>
        <div className="ov-clock num">{hm(timer.now)}</div>
      </header>

      <main className="ov-center">
        <button
          type="button"
          className={paused ? "ov-time num paused" : "ov-time num"}
          title={canFlip ? "点一下切换剩余 / 已学" : undefined}
          onClick={() => {
            if (!canFlip) return;
            play("toggle");
            void attempt(
              api.settings.update({ time_display: settings.time_display === "remaining" ? "elapsed" : "remaining" }),
            );
          }}
        >
          <BigClock text={clock(big.ms, big.round)} />
        </button>
        <div className="ov-note wk">{paused ? `暂停中 · ${learned}` : learned}</div>
      </main>

      <footer className="ov-bottom">
        <div className="horizon" style={{ "--p": line.p } as CSSProperties}>
          <div className="track" />
          <div className="fill" />
          <div className="glow" />
        </div>
        <div className="h-labels num">
          <span>{hm(line.from)}</span>
          <span>{hm(line.to)}</span>
        </div>
        <p className="quote wk">慢慢来，比较快。</p>
      </footer>

      <nav className="textctl wk">
        <button type="button" className={paused ? "on" : ""} onClick={() => void togglePause()}>
          {paused ? "继续" : "暂停"}
        </button>
        <button
          type="button"
          onClick={() => {
            play("tap");
            void attempt(api.timer.switchDisplay("mini"));
          }}
        >
          缩成浮窗
        </button>
        <button type="button" className="end" onClick={end}>
          {confirmEnd ? "再点一次结束" : "结束"}
        </button>
      </nav>
    </div>
  );
}

/** 冒号画成两个圆点，比字体里的冒号更轻。 */
function BigClock({ text }: { text: string }) {
  const parts = text.split(":");
  return (
    <>
      {parts.map((p, i) => (
        <Fragment key={i}>
          {i > 0 && <span className="colon">:</span>}
          {p}
        </Fragment>
      ))}
    </>
  );
}
