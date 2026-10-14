import { useState } from "react";
import { Penguin, usePack } from "../../features/character/Penguin";
import { fullScore, scoreError } from "../../features/timer/mock";
import { quoteFor } from "../../features/atmosphere/quotes";
import { toDateStr } from "../../features/records/dates";
import { api } from "../../shared/api";
import { durationParts } from "../../shared/format";
import { setTimer } from "../../shared/state";
import type { Outcome, Subject } from "../../shared/types";
import { Button, Dot } from "../../shared/ui/controls";
import { attempt } from "../../shared/ui/toast";

const NOTE_MAX = 200;

export function EndScreen({ outcome, subject }: { outcome: Outcome; subject: Subject | null }) {
  const pack = usePack();
  const [note, setNote] = useState("");
  const [saved, setSaved] = useState("");
  const [busy, setBusy] = useState(false);

  const saveNote = async () => {
    if (!outcome.session_id || note.trim() === saved) return true;
    const ok = await attempt(api.sessions.setNote(outcome.session_id, note.trim()));
    if (ok === undefined) return false;
    setSaved(note.trim());
    return true;
  };

  const close = async () => {
    setBusy(true);
    // 写了备注没点“记下”就离开，也帮他记下
    if (await saveNote()) {
      const view = await attempt(api.timer.close(false));
      if (view) setTimer(view);
    }
    setBusy(false);
  };

  const task = outcome.plan.task;
  return (
    <div className="end-wrap">
      {outcome.recorded ? (
        <p className="end-title wk">
          这一段学了
          {durationParts(outcome.focused_ms).map((p) => (
            <span key={p.unit}>
              <em className="num">{p.n}</em>
              {p.unit}
            </span>
          ))}
          ，已经记下了。
        </p>
      ) : (
        <p className="end-title wk">这一段太短，没有记下。</p>
      )}
      <p className="end-sub wk">
        <Dot color={subject?.color} />
        {[outcome.plan.mode === "mock" ? `模考 · ${subject?.name ?? ""}` : subject?.name, task]
          .filter(Boolean)
          .join(" · ")}
      </p>

      {pack.show && outcome.reason === "completed" && (
        <div className="end-pet">
          <span className="bubble">嘎嘎！</span>
          <Penguin pose="done" size={160} pack={pack.id} dir={pack.dir} clip={pack.clips?.done} />
        </div>
      )}

      <div className="end-actions">
        <Button variant="primary" disabled={busy} onClick={() => void close()}>
          回到电脑
        </Button>
      </div>

      {outcome.recorded && outcome.session_id && outcome.plan.mode === "mock" && (
        <ScoreBox sessionId={outcome.session_id} subjectName={subject?.name ?? ""} />
      )}

      {outcome.recorded && outcome.session_id && (
        <div className="note-box">
          <input
            className="wk"
            placeholder="加个备注，比如：第 3 题没想通，明天问"
            value={note}
            maxLength={NOTE_MAX}
            onChange={(e) => setNote(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && void saveNote()}
          />
          <Button disabled={!note.trim() || note.trim() === saved} onClick={() => void saveNote()}>
            {saved && note.trim() === saved ? "已记下" : "记下"}
          </Button>
        </div>
      )}

      <p className="quote wk">{quoteFor(toDateStr(new Date(outcome.ended_at || Date.now())))}</p>
    </div>
  );
}

function ScoreBox({ sessionId, subjectName }: { sessionId: number; subjectName: string }) {
  const [score, setScore] = useState("");
  const [full, setFull] = useState(String(fullScore(subjectName)));
  const [saved, setSaved] = useState(false);
  const [busy, setBusy] = useState(false);
  const parsedScore = score.trim() === "" ? null : Number(score);
  const parsedFull = Number(full);
  const error = parsedScore == null ? null : scoreError(parsedScore, parsedFull);

  const save = async () => {
    if (parsedScore == null || error) return;
    setBusy(true);
    const row = await attempt(api.sessions.setScore(sessionId, parsedScore, parsedFull));
    setBusy(false);
    if (row) setSaved(true);
  };

  return (
    <div className="note-box score-box">
      <input
        className="num"
        inputMode="numeric"
        aria-label="分数"
        placeholder="分数，可以不填"
        value={score}
        onChange={(e) => {
          setSaved(false);
          setScore(e.target.value.replace(/[^\d]/g, ""));
        }}
      />
      <span className="score-full">
        /
        <input
          className="num"
          inputMode="numeric"
          aria-label="满分"
          value={full}
          onChange={(e) => {
            setSaved(false);
            setFull(e.target.value.replace(/[^\d]/g, ""));
          }}
        />
      </span>
      <Button size="small" disabled={busy || parsedScore == null || !!error || saved} onClick={() => void save()}>
        {saved ? "已记下" : "记下分数"}
      </Button>
      {error && <p className="field-error">{error}</p>}
    </div>
  );
}
