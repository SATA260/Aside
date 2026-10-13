import { useState } from "react";
import { DISPLAYS } from "../../features/timer/displays";
import {
  COUNTDOWN_PRESETS,
  initialForm,
  joinHm,
  lengthError,
  MOCK_PRESETS,
  type StartForm,
  splitHm,
  switchMode,
  TASK_MAX,
  toStartArgs,
} from "../../features/timer/startForm";
import { api } from "../../shared/api";
import { clock, durationParts, durationText, hm, MINUTE } from "../../shared/format";
import { play } from "../../shared/sfx";
import { setTimer, useApp } from "../../shared/state";
import type { Mode, Outcome, Settings, TimerView } from "../../shared/types";
import { Button, Chip, Dot, Segmented, TextButton } from "../../shared/ui/controls";
import { attempt } from "../../shared/ui/toast";
import "./start.css";

const MODES: { value: Mode; label: string }[] = [
  { value: "countdown", label: "倒计时" },
  { value: "countup", label: "正计时" },
  { value: "mock", label: "模考" },
];

export function StartPage() {
  const settings = useApp((s) => s.settings);
  const subjects = useApp((s) => s.subjects);
  const timer = useApp((s) => s.timer);
  if (!settings || !timer || subjects.length === 0) return <div className="page" />;
  const init = initialForm(
    settings,
    subjects,
    timer,
    DISPLAYS.map((d) => d.value),
  );
  // 每结束一段都重新按上一次的选择填好表单
  return (
    <StartFormView
      key={`${timer.again}-${timer.last?.ended_at ?? 0}`}
      init={init}
      timer={timer}
      settings={settings}
    />
  );
}

function StartFormView({ init, timer, settings }: { init: StartForm; timer: TimerView; settings: Settings }) {
  const subjects = useApp((s) => s.subjects);
  const [form, setForm] = useState(init);
  const [busy, setBusy] = useState(false);
  const set = (patch: Partial<StartForm>) => setForm((f) => ({ ...f, ...patch }));

  const fallback = subjects.find((s) => s.is_fallback) ?? subjects[0];
  const subjectId = subjects.some((s) => s.id === form.subjectId) ? form.subjectId : fallback.id;
  const active = timer.phase === "running" || timer.phase === "paused";
  const error = form.mode === "countup" ? null : lengthError(form.lengthMs);

  const start = async () => {
    play("start");
    setBusy(true);
    const view = await attempt(api.timer.start(toStartArgs({ ...form, subjectId })));
    setBusy(false);
    if (view) setTimer(view);
  };

  return (
    <div className="page start-page">
      <Greeting todayMs={timer.today_ms} />
      {timer.recovered && <RecoveredNotice outcome={timer.recovered} />}
      {active && <ActiveNotice timer={timer} />}

      <div className="field">
        <div className="label">科目</div>
        <div className="chips">
          {subjects.map((s) => (
            <Chip key={s.id} on={s.id === subjectId} color={s.color} onClick={() => set({ subjectId: s.id })}>
              <Dot color={s.color} />
              <span className="wk">{s.name}</span>
            </Chip>
          ))}
        </div>
      </div>

      <div className="row">
        <div className="field">
          <div className="label">方式</div>
          <Segmented
            value={form.mode}
            options={MODES}
            onChange={(mode) => setForm((f) => switchMode(f, mode, settings))}
          />
        </div>
        {DISPLAYS.length > 1 && (
          <div className="field">
            <div className="label">显示</div>
            <Segmented value={form.display} options={DISPLAYS} onChange={(display) => set({ display })} />
          </div>
        )}
      </div>

      <div className="field">
        <div className="label">{form.mode === "countup" ? "时长" : "时长（最长 24 小时）"}</div>
        {form.mode === "countup" ? (
          <p className="countup-hint">从 0 开始往上数，想停就停；连续 24 小时会自动结束。</p>
        ) : (
          <LengthPicker
            key={form.mode}
            presets={form.mode === "mock" ? MOCK_PRESETS : COUNTDOWN_PRESETS}
            value={form.lengthMs}
            onChange={(lengthMs) => set({ lengthMs })}
          />
        )}
        {error && <p className="field-error">{error}</p>}
      </div>

      <div className="field">
        <div className="label">这一段要做什么（可以不填）</div>
        <input
          className="input"
          placeholder={form.mode === "mock" ? "比如：数学一 · 2023 真题" : "比如：高数基础 第 12 讲"}
          value={form.task}
          maxLength={TASK_MAX}
          onChange={(e) => set({ task: e.target.value })}
          onKeyDown={(e) => e.key === "Enter" && !error && !active && !busy && void start()}
        />
      </div>

      <div className="go">
        <Button variant="primary" cue={null} disabled={!!error || active || busy} onClick={() => void start()}>
          开始专注
        </Button>
      </div>
    </div>
  );
}

function Greeting({ todayMs }: { todayMs: number }) {
  if (todayMs < MINUTE) {
    return (
      <div className="hello">
        <h1>今天也坐下来了</h1>
      </div>
    );
  }
  return (
    <div className="hello">
      <h1>
        今天学了
        {durationParts(todayMs).map((p) => (
          <span key={p.unit}>
            {" "}
            <b>{p.n}</b> {p.unit}
          </span>
        ))}
      </h1>
    </div>
  );
}

function RecoveredNotice({ outcome }: { outcome: Outcome }) {
  const text = outcome.recorded
    ? `上次有一段专注已记下：${hm(outcome.started_at)}–${hm(outcome.ended_at)}，${durationText(outcome.focused_ms)}。`
    : "上次那一段不到 1 分钟，就不记了。";
  return (
    <div className="notice">
      <span>{text}</span>
      <TextButton onClick={() => void attempt(api.timer.dismissRecovered()).then((v) => v && setTimer(v))}>
        知道了
      </TextButton>
    </div>
  );
}

function ActiveNotice({ timer }: { timer: TimerView }) {
  const shown = timer.remaining_ms ?? timer.focused_ms;
  return (
    <div className="notice">
      <span>
        {timer.phase === "paused" ? "暂停中" : "正在专注"}：<span className="wk">{timer.subject?.name}</span>{" "}
        <span className="num">{clock(shown, timer.remaining_ms === null ? "floor" : "ceil")}</span>
      </span>
    </div>
  );
}

function LengthPicker({ presets, value, onChange }: { presets: number[]; value: number; onChange: (ms: number) => void }) {
  const [custom, setCustom] = useState(!presets.includes(value));
  const current = splitHm(value);
  const [hText, setH] = useState(String(current.h));
  const [mText, setM] = useState(String(current.m));
  // 没选“自定义”时，输入框里显示当前选中的时长，点进去就从这个数开始改
  const h = custom ? hText : String(current.h);
  const m = custom ? mText : String(current.m);

  const edit = (nh: string, nm: string) => {
    const hours = Math.min(24, Number(nh.replace(/\D/g, "") || 0));
    const minutes = Math.min(59, Number(nm.replace(/\D/g, "") || 0));
    setH(nh === "" ? "" : String(hours));
    setM(nm === "" ? "" : String(minutes));
    setCustom(true);
    onChange(joinHm(hours, minutes));
  };

  return (
    <div className="chips tight">
      {presets.map((ms) => (
        <Chip
          key={ms}
          className="num"
          on={!custom && value === ms}
          onClick={() => {
            setCustom(false);
            onChange(ms);
          }}
        >
          {presetLabel(ms)}
        </Chip>
      ))}
      <span
        className={custom ? "chip custom on" : "chip custom"}
        onClick={() => {
          if (custom) return;
          play("tap");
          setH(h);
          setM(m);
          setCustom(true);
        }}
      >
        <input
          aria-label="小时"
          inputMode="numeric"
          maxLength={2}
          value={h}
          onChange={(e) => edit(e.target.value, m)}
          onFocus={(e) => e.target.select()}
        />
        <small>小时</small>
        <input
          aria-label="分钟"
          inputMode="numeric"
          maxLength={2}
          value={m}
          onChange={(e) => edit(h, e.target.value)}
          onFocus={(e) => e.target.select()}
        />
        <small>分</small>
      </span>
    </div>
  );
}

function presetLabel(ms: number): string {
  const minutes = ms / MINUTE;
  if (minutes < 120) return String(minutes);
  return `${minutes / 60} 小时`;
}
