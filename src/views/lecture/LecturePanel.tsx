import { useEffect, useState } from "react";
import { SoundChoices } from "../../features/atmosphere/library";
import { Penguin, useReduceMotion } from "../../features/character/Penguin";
import { pose } from "../../features/character/pose";
import { bigTime, horizon } from "../../features/timer/overlay";
import { api } from "../../shared/api";
import { on } from "../../shared/ipc";
import { clock, durationText } from "../../shared/format";
import { play } from "../../shared/sfx";
import { setTimer, useApp } from "../../shared/state";
import type { Bookmark, ThemeName } from "../../shared/types";
import { Dot } from "../../shared/ui/controls";
import { attempt } from "../../shared/ui/toast";
import "./lecture.css";

/** 网课窗口的侧栏和地址栏。课程网页由后端嵌在左边。 */
export function LecturePanel() {
  const timer = useApp((s) => s.timer);
  const settings = useApp((s) => s.settings);
  const [draft, setDraft] = useState("");
  const [bookmarks, setBookmarks] = useState<Bookmark[]>([]);
  const [atmo, setAtmo] = useState(false);
  const [only, setOnly] = useState(false);
  const [confirmEnd, setConfirmEnd] = useState(false);
  const reduce = useReduceMotion();

  useEffect(() => on<{ on: boolean }>("lecture:video", (payload) => setOnly(payload.on)), []);
  useEffect(() => {
    void api.lecture.bookmarks().then(setBookmarks);
  }, []);
  useEffect(() => {
    if (settings) setDraft(settings.lecture_url);
  }, [settings?.lecture_url]);

  const go = (url: string) => {
    play("tap");
    void attempt(api.lecture.go(url)).then((saved) => saved && setDraft(saved));
  };

  const active = timer && (timer.phase === "running" || timer.phase === "paused");
  const paused = timer?.phase === "paused";
  const big = timer && settings ? bigTime(timer, settings.time_display) : null;
  const line = timer ? horizon(timer) : null;

  const togglePause = async () => {
    if (!timer) return;
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

  return (
    <div className="lecture">
      <header className="lecture-bar">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            go(draft);
          }}
        >
          <input
            className="input"
            aria-label="课程地址"
            placeholder="课程网址"
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
          />
          <button type="submit">打开</button>
        </form>
        <div className="lecture-marks">
          {bookmarks.map((mark) => (
            <button type="button" key={mark.url} onClick={() => go(mark.url)}>
              {mark.name}
            </button>
          ))}
        </div>
      </header>

      <div className="lecture-stage">
        {!settings?.lecture_url && (
          <div className="lecture-home">
            <p>选一个常看的网站，登录会留在这台电脑上。</p>
            {bookmarks.map((mark) => (
              <button type="button" key={mark.url} onClick={() => go(mark.url)}>
                {mark.name}
              </button>
            ))}
          </div>
        )}
      </div>

      <aside className="lecture-side">
        {active && timer && big && line ? (
          <>
            <div className="who">
              <Dot color={timer.subject?.color} />
              <span className="wk">{timer.plan?.mode === "mock" ? "模考" : timer.subject?.name}</span>
            </div>
            {timer.plan?.task && <p className="task wk">{timer.plan.task}</p>}
            <div className="lecture-time num">{clock(big.ms, big.round)}</div>
            <div className="horizon">
              <i style={{ width: `${line.p * 100}%` }} />
            </div>
            <p className="today">今天 {durationText(timer.today_ms)}</p>
            <Penguin pose={pose(timer.phase, timer.remaining_ms, reduce)} size={120} />
            <div className="lecture-actions">
              <button type="button" onClick={() => void togglePause()}>
                {paused ? "继续" : "暂停"}
              </button>
              <button type="button" onClick={() => setAtmo((v) => !v)}>
                氛围
              </button>
              <button
                type="button"
                className={only ? "on" : ""}
                onClick={() => {
                  play("tap");
                  const time = big ? clock(big.ms, big.round) : "";
                  void attempt(api.lecture.video(!only, time));
                }}
              >
                {only ? "退出只看视频" : "只看视频"}
              </button>
              <button type="button" className="end" onClick={end}>
                {confirmEnd ? "再点一次结束" : "结束"}
              </button>
            </div>
            {atmo && settings && (
              <div className="lecture-atmo">
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
          </>
        ) : (
          <p className="hint">开始一段专注，并把显示方式选成网课，侧栏会显示时间。</p>
        )}
      </aside>
    </div>
  );
}
