import { type CSSProperties, useEffect, useState } from "react";
import { addDays, dayLabel, dayTitle, toDateStr } from "../../features/records/dates";
import { blocks, crossesMidnight } from "../../features/records/timeline";
import { api } from "../../shared/api";
import { durationParts, durationShort, hm } from "../../shared/format";
import { play } from "../../shared/sfx";
import { useApp } from "../../shared/state";
import type { DateStr, DayEntry, DayView } from "../../shared/types";
import { Dot } from "../../shared/ui/controls";
import { attempt } from "../../shared/ui/toast";
import { Calendar } from "./Calendar";
import { EntryEditor } from "./EntryEditor";
import { ManualForm } from "./ManualForm";
import { Overview } from "./Overview";
import "./records.css";

const TICKS = [0, 6, 12, 18, 24];

/** 后端时钟的今天。开发时时钟可以快进，所以不直接用 `new Date()`。 */
export function useToday(): DateStr {
  return useApp((s) => toDateStr(new Date(s.timer?.now ?? Date.now())));
}

export function RecordsPage() {
  const today = useToday();
  const [tab, setTab] = useState<"day" | "overview">("day");
  const [date, setDate] = useState<DateStr>(today);
  const openDay = (d: DateStr) => {
    setDate(d);
    setTab("day");
  };
  return (
    <div className="page records-page">
      <div className="tabs" role="tablist">
        {(
          [
            ["day", "按天"],
            ["overview", "总览"],
          ] as const
        ).map(([name, label]) => (
          <button
            type="button"
            key={name}
            className={tab === name ? "on" : ""}
            onClick={() => {
              if (tab !== name) play("tap");
              setTab(name);
            }}
          >
            {label}
          </button>
        ))}
      </div>
      {tab === "day" ? (
        <DayPanel date={date} today={today} onDate={setDate} />
      ) : (
        <Overview today={today} onOpen={openDay} />
      )}
    </div>
  );
}

function DayPanel({ date, today, onDate }: { date: DateStr; today: DateStr; onDate: (d: DateStr) => void }) {
  const dataVersion = useApp((s) => s.dataVersion);
  const [day, setDay] = useState<DayView | null>(null);
  const [editing, setEditing] = useState<number | null>(null);
  const [picking, setPicking] = useState(false);
  const [adding, setAdding] = useState(false);

  useEffect(() => {
    let live = true;
    void attempt(api.sessions.day(date)).then((d) => live && d && setDay(d));
    return () => {
      live = false;
    };
  }, [date, dataVersion]);

  useEffect(() => {
    setEditing(null);
    setAdding(false);
  }, [date]);

  const go = (d: DateStr) => {
    if (d > today) return;
    play("tap");
    setPicking(false);
    onDate(d);
  };

  return (
    <>
      <div className="dayhead">
        <div className="nav-date">
          <button type="button" className="arrow" aria-label="前一天" onClick={() => go(addDays(date, -1))}>
            ‹
          </button>
          <button type="button" className="date-title" onClick={() => setPicking((p) => !p)}>
            <h2>
              {dayTitle(date)}
              <small>{dayLabel(date, today)}</small>
            </h2>
          </button>
          <button
            type="button"
            className="arrow"
            aria-label="后一天"
            disabled={date >= today}
            onClick={() => go(addDays(date, 1))}
          >
            ›
          </button>
          {picking && (
            <Calendar
              value={date}
              today={today}
              onPick={(d) => {
                setPicking(false);
                go(d);
              }}
              onClose={() => setPicking(false)}
            />
          )}
        </div>
        <div className="sum">
          {day && day.entries.length > 0 && (
            <span>
              学了
              {durationParts(day.total_ms).map((p) => (
                <span key={p.unit}>
                  {" "}
                  <b>{p.n}</b> {p.unit}
                </span>
              ))}{" "}
              · {day.entries.length} 段
            </span>
          )}
          <button
            type="button"
            className="link"
            onClick={() => {
              play("tap");
              setAdding((v) => !v);
            }}
          >
            ＋ 补记一段
          </button>
        </div>
      </div>

      {adding && <ManualForm date={date} onDone={() => setAdding(false)} />}

      <div className="timeline">
        {day &&
          blocks(day.entries, day.start, day.end).map((b, i) => (
            <button
              type="button"
              key={i}
              aria-label="这一段"
              style={{ left: `${b.left}%`, width: `${b.width}%`, background: b.color } as CSSProperties}
              onClick={() => setEditing(b.id)}
            />
          ))}
      </div>
      <div className="ticks">
        {TICKS.map((t) => (
          <span key={t} style={{ left: `${(t / 24) * 100}%` }}>
            {t}
          </span>
        ))}
      </div>

      {day && day.entries.length === 0 && <p className="day-empty">这一天没有专注记录。</p>}
      {day && day.entries.length > 0 && (
        <div className="sessions">
          {day.entries.map((e) =>
            editing === e.id ? (
              <EntryEditor key={e.id} entry={e} onDone={() => setEditing(null)} />
            ) : (
              <EntryRow key={e.id} entry={e} day={day} onEdit={() => setEditing(e.id)} />
            ),
          )}
        </div>
      )}
    </>
  );
}

function EntryRow({ entry, day, onEdit }: { entry: DayEntry; day: DayView; onEdit: () => void }) {
  return (
    <div
      className="sess"
      role="button"
      tabIndex={0}
      onClick={() => {
        play("tap");
        onEdit();
      }}
      onKeyDown={(ev) => ev.key === "Enter" && onEdit()}
    >
      <span className="time">
        {hm(entry.started_at)} – {hm(entry.ended_at)}
      </span>
      <Dot color={entry.subject_color} />
      <div className="what">
        <span className="wk">{entry.subject_name}</span>
        {entry.task && <span className="sub"> · {entry.task}</span>}
        {crossesMidnight(entry, day.start, day.end) && <span className="tag">跨零点</span>}
        {entry.source === "manual" && <span className="tag">补记</span>}
        {entry.mode === "mock" && <span className="tag">模考</span>}
        {entry.mock && (
          <span className="tag">
            {entry.mock.score}/{entry.mock.full_score}
          </span>
        )}
        {entry.note ? <div className="note">{entry.note}</div> : <div className="add">＋ 加个备注</div>}
      </div>
      <span className="dur">{durationShort(entry.focused_ms)}</span>
    </div>
  );
}
