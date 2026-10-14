import { useState } from "react";
import { MAX_MINUTES, MIN_MINUTES, rangeFrom } from "../../features/records/span";
import { api } from "../../shared/api";
import { useApp } from "../../shared/state";
import type { DateStr } from "../../shared/types";
import { Button, Chip, Dot } from "../../shared/ui/controls";
import { attempt } from "../../shared/ui/toast";
import { toDateStr } from "../../features/records/dates";

const NOTE_MAX = 200;

export function ManualForm({ date, onDone }: { date: DateStr; onDone: () => void }) {
  const subjects = useApp((s) => s.subjects);
  const now = useApp((s) => s.timer?.now ?? Date.now());
  const [subjectId, setSubjectId] = useState(subjects.find((s) => s.is_fallback)?.id ?? subjects[0]?.id ?? 0);
  const [day, setDay] = useState(date);
  const [time, setTime] = useState("19:30");
  const [minutes, setMinutes] = useState(90);
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);

  const range = rangeFrom(day, time, minutes);
  const localError =
    !Number.isInteger(minutes) || minutes < MIN_MINUTES
      ? "至少 1 分钟"
      : minutes > MAX_MINUTES
        ? "最长 24 小时"
        : !range
          ? "开始时间写成 19:30 这样"
          : range.ended_at > now
            ? "不能晚于当前时刻"
            : null;

  const save = async () => {
    if (!range || localError) return;
    setBusy(true);
    const saved = await attempt(api.sessions.add({ subject_id: subjectId, ...range, note }));
    setBusy(false);
    if (saved) onDone();
  };

  return (
    <div className="entry-editor">
      <div className="label">补记一段</div>
      <div className="chips">
        {subjects.map((s) => (
          <Chip key={s.id} on={s.id === subjectId} color={s.color} onClick={() => setSubjectId(s.id)}>
            <Dot color={s.color} />
            <span className="wk">{s.name}</span>
          </Chip>
        ))}
      </div>
      <div className="edit-when">
        <input className="input small" type="date" aria-label="日期" value={day} max={toDateStr(new Date(now))} onChange={(e) => setDay(e.target.value)} />
        <input className="input small" type="time" aria-label="开始时间" value={time} onChange={(e) => setTime(e.target.value)} />
        <label className="minutes">
          <input
            className="input small"
            type="number"
            aria-label="时长"
            min={MIN_MINUTES}
            max={MAX_MINUTES}
            value={minutes}
            onChange={(e) => setMinutes(Number(e.target.value))}
          />
          分钟
        </label>
      </div>
      <input className="input small" aria-label="备注" placeholder="备注，可以不填" maxLength={NOTE_MAX} value={note} onChange={(e) => setNote(e.target.value)} />
      {localError && <p className="field-error">{localError}</p>}
      <div className="edit-actions">
        <Button variant="primary" size="small" disabled={busy || !!localError || !subjectId} onClick={() => void save()}>
          补上
        </Button>
        <Button size="small" cue={null} disabled={busy} onClick={onDone}>
          取消
        </Button>
      </div>
    </div>
  );
}
