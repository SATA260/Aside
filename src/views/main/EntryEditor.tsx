import { useState } from "react";
import { toDateStr } from "../../features/records/dates";
import { MAX_MINUTES, MIN_MINUTES, rangeFrom, wallMinutes } from "../../features/records/span";
import { api } from "../../shared/api";
import { hm } from "../../shared/format";
import { useApp } from "../../shared/state";
import type { DayEntry, Subject } from "../../shared/types";
import { Button, Chip, Dot } from "../../shared/ui/controls";
import { attempt } from "../../shared/ui/toast";

const TASK_MAX = 50;
const NOTE_MAX = 200;

export function EntryEditor({ entry, onDone }: { entry: DayEntry; onDone: () => void }) {
  const live = useApp((s) => s.subjects);
  const now = useApp((s) => s.timer?.now ?? Date.now());
  const subjects = subjectsFor(live, entry);

  const [subjectId, setSubjectId] = useState(entry.subject_id);
  const [date, setDate] = useState(toDateStr(new Date(entry.started_at)));
  const [time, setTime] = useState(hm(entry.started_at));
  const [minutes, setMinutes] = useState(wallMinutes(entry.started_at, entry.ended_at));
  const [task, setTask] = useState(entry.task);
  const [note, setNote] = useState(entry.note);
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState(false);

  const initialMinutes = wallMinutes(entry.started_at, entry.ended_at);
  const timeChanged = date !== toDateStr(new Date(entry.started_at)) || time !== hm(entry.started_at) || minutes !== initialMinutes;
  const range = rangeFrom(date, time, minutes);
  const localError =
    !Number.isInteger(minutes) || minutes < MIN_MINUTES
      ? "至少 1 分钟"
      : minutes > MAX_MINUTES
        ? "最长 24 小时"
        : !range
          ? "开始时间写成 07:40 这样"
          : range.ended_at > now
            ? "不能晚于当前时刻"
            : null;

  const save = async () => {
    if (!range || localError) return;
    setBusy(true);
    const saved = await attempt(
      api.sessions.update(entry.id, {
        subject_id: subjectId === entry.subject_id ? undefined : subjectId,
        task: task.trim() === entry.task ? undefined : task,
        note: note.trim() === entry.note ? undefined : note,
        ...(timeChanged ? range : {}),
      }),
    );
    setBusy(false);
    if (saved) onDone();
  };

  const remove = async () => {
    setBusy(true);
    const ok = await attempt(api.sessions.remove(entry.id));
    setBusy(false);
    if (ok !== undefined) onDone();
  };

  return (
    <div className="entry-editor">
      <div className="chips">
        {subjects.map((s) => (
          <Chip key={s.id} on={s.id === subjectId} color={s.color} onClick={() => setSubjectId(s.id)}>
            <Dot color={s.color} />
            <span className="wk">{s.name}</span>
            {s.archived && <span className="tag">已归档</span>}
          </Chip>
        ))}
      </div>
      <div className="edit-when">
        <input className="input small" type="date" aria-label="日期" value={date} max={toDateStr(new Date(now))} onChange={(e) => setDate(e.target.value)} />
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
      {timeChanged && entry.focused_ms !== entry.ended_at - entry.started_at && (
        <p className="edit-hint">这段里有暂停，改时间后按连续学习计算。</p>
      )}
      <input className="input small" aria-label="任务" placeholder="任务，可以不填" maxLength={TASK_MAX} value={task} onChange={(e) => setTask(e.target.value)} />
      <input className="input small" aria-label="备注" placeholder="备注，可以不填" maxLength={NOTE_MAX} value={note} onChange={(e) => setNote(e.target.value)} />
      {localError && timeChanged && <p className="field-error">{localError}</p>}
      <div className="edit-actions">
        <Button variant="primary" size="small" disabled={busy || (timeChanged && !!localError)} onClick={() => void save()}>
          保存
        </Button>
        <Button size="small" cue={null} disabled={busy} onClick={onDone}>
          取消
        </Button>
        {confirming ? (
          <span className="confirm-delete">
            删除这条记录？
            <Button size="small" cue="delete" disabled={busy} onClick={() => void remove()}>
              删除
            </Button>
            <Button size="small" cue={null} onClick={() => setConfirming(false)}>
              留下
            </Button>
          </span>
        ) : (
          <button type="button" className="link muted" onClick={() => setConfirming(true)}>
            删除
          </button>
        )}
      </div>
    </div>
  );
}

function subjectsFor(live: Subject[], entry: DayEntry): Subject[] {
  if (live.some((s) => s.id === entry.subject_id)) return live;
  return [
    {
      id: entry.subject_id,
      name: entry.subject_name,
      color: entry.subject_color,
      sort: -1,
      archived: true,
      is_fallback: false,
    },
    ...live,
  ];
}
