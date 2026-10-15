import { type CSSProperties, Fragment, useEffect, useState } from "react";
import { backdrops, nextOf, soundOptions } from "../../features/atmosphere/catalog";
import { SceneChoices, SoundChoices, useLibrary } from "../../features/atmosphere/library";
import { useAtmoPreview } from "../../features/atmosphere/preview";
import { bigTime, focusChrome, horizon } from "../../features/timer/overlay";
import { quoteFor } from "../../features/atmosphere/quotes";
import { toDateStr } from "../../features/records/dates";
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
  const [atmo, setAtmo] = useState(false);
  const [notice, setNotice] = useState("");
  const { sounds, images } = useLibrary();

  useEffect(() => {
    if (!notice) return;
    const id = setTimeout(() => setNotice(""), 2000);
    return () => clearTimeout(id);
  }, [notice]);

  useEffect(() => {
    if (!confirmEnd) return;
    const id = setTimeout(() => setConfirmEnd(false), 3000);
    return () => clearTimeout(id);
  }, [confirmEnd]);

  const choose = (patch: Partial<Settings>, label: string) => {
    setNotice(`换成了 ${label}`);
    void attempt(api.settings.update(patch));
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.metaKey || e.ctrlKey || e.altKey) return;
      if (e.code === "KeyB") {
        const list = backdrops(images);
        const current = settings.scene_image != null ? `img:${settings.scene_image}` : (settings.scene ?? "sky");
        const next = nextOf(list, current);
        if (next.image != null) choose({ scene_image: next.image }, next.name);
        else choose({ scene: next.id as Settings["scene"], scene_image: null }, next.name);
      } else if (e.code === "KeyN") {
        const next = nextOf(soundOptions(sounds), settings.ambient ?? "");
        choose({ ambient: next.id }, next.name);
      } else if (e.code === "ArrowUp" || e.code === "ArrowDown") {
        e.preventDefault();
        const step = e.code === "ArrowUp" ? 0.05 : -0.05;
        const ambient_volume = Math.min(1, Math.max(0, (settings.ambient_volume ?? 0.6) + step));
        void attempt(api.settings.update({ ambient_volume }));
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

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
      className={focusChrome(idle, atmo)}
      style={{ transform: `translate(${drift.x}px, ${drift.y}px)` }}
    >
      <header className="ov-top">
        <div className="ov-subject wk">
          <Dot color={timer.subject?.color} />
          <span>{timer.plan?.mode === "mock" ? "模考" : timer.subject?.name}</span>
          {timer.plan?.task && <span className="task">{timer.plan.task}</span>}
        </div>
        <div className="ov-clock num">{hm(timer.now)}</div>
        {notice && <p className="ov-notice wk">{notice}</p>}
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
        <button type="button" className={atmo ? "on" : ""} onClick={() => setAtmo((v) => !v)}>
          氛围
        </button>
        <button type="button" className="end" onClick={end}>
          {confirmEnd ? "再点一次结束" : "结束"}
        </button>
      </nav>

      {atmo && (
        <div className="atmo-live wk">
          <div>
            <span>画面</span>
            <SceneChoices
              scene={settings.scene}
              imageId={settings.scene_image}
              onScene={(id, name) => choose({ scene: id, scene_image: null }, name)}
              onImage={(id, name) => choose({ scene_image: id }, name)}
              onEnterScene={(id) => useAtmoPreview.setState({ scene: id, picture: null })}
              onEnterImage={(src) => useAtmoPreview.setState({ scene: null, picture: src })}
              onLeave={() => useAtmoPreview.setState({ scene: null, picture: null })}
            />
          </div>
          <div>
            <span>声音</span>
            <SoundChoices ambient={settings.ambient} onPick={(id, name) => choose({ ambient: id }, name)} />
          </div>
        </div>
      )}

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
        <p className="quote wk">{quoteFor(toDateStr(new Date(timer.now)))}</p>
      </footer>
      {!idle && <p className="keys-hint">B 画面 · N 声音 · ↑↓ 音量</p>}
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
