import { useState } from "react";
import { api } from "../../shared/api";
import { durationParts } from "../../shared/format";
import { setTimer } from "../../shared/state";
import type { Outcome, Subject } from "../../shared/types";
import { Button, Dot } from "../../shared/ui/controls";
import { attempt } from "../../shared/ui/toast";

const NOTE_MAX = 200;

export function EndScreen({ outcome, subject }: { outcome: Outcome; subject: Subject | null }) {
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

  const close = async (again: boolean) => {
    setBusy(true);
    // 写了备注没点“记下”就离开，也帮他记下
    if (await saveNote()) {
      const view = await attempt(api.timer.close(again));
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

      <div className="end-actions">
        <Button variant="primary" disabled={busy} onClick={() => void close(true)}>
          再来一段
        </Button>
        <Button disabled={busy} onClick={() => void close(false)}>
          回到电脑
        </Button>
      </div>

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

      <p className="quote wk">今天也坐下来了，这就很好。</p>
    </div>
  );
}
