import { useEffect, useState } from "react";
import { axisMax, barPercent, HEAT_ALPHA, heatLevel } from "../../features/records/charts";
import { weekday } from "../../features/records/dates";
import { api } from "../../shared/api";
import { durationParts, durationShort, HOUR } from "../../shared/format";
import { play } from "../../shared/sfx";
import { useApp } from "../../shared/state";
import type { DateStr, Overview as OverviewData, Subject } from "../../shared/types";
import { Dot } from "../../shared/ui/controls";
import { attempt } from "../../shared/ui/toast";

const GOAL_HOURS = [0, 1, 2, 3, 4, 5, 6, 8, 10, 12];

export function Overview({ today, onOpen }: { today: DateStr; onOpen: (date: DateStr) => void }) {
  const dataVersion = useApp((s) => s.dataVersion);
  const goalMs = useApp((s) => s.settings?.daily_goal_ms ?? 0);
  const [data, setData] = useState<OverviewData | null>(null);
  const [subjects, setSubjects] = useState<Subject[]>([]);
  const [span, setSpan] = useState<"week" | "month">("week");

  useEffect(() => {
    let live = true;
    void attempt(api.stats.overview()).then((d) => live && d && setData(d));
    void attempt(api.subjects.list(true)).then((s) => live && s && setSubjects(s));
    return () => {
      live = false;
    };
  }, [dataVersion]);

  if (!data) return null;
  const name = (id: number) => subjects.find((s) => s.id === id)?.name ?? "";
  const color = (id: number) => subjects.find((s) => s.id === id)?.color;
  const todayRow = data.days.find((d) => d.date === today);
  const trend = span === "week" ? data.days.slice(-7) : data.days.filter((d) => d.date.slice(0, 7) === today.slice(0, 7));
  const max = axisMax(trend.map((d) => d.total_ms), goalMs);
  const peak = Math.max(...data.days.map((d) => d.total_ms), 0);
  const hours = Math.round(goalMs / HOUR);
  const goalChoices = GOAL_HOURS.includes(hours) ? GOAL_HOURS : [...GOAL_HOURS, hours].sort((a, b) => a - b);

  const open = (date: DateStr) => {
    play("tap");
    onOpen(date);
  };

  return (
    <div className="stats-grid">
      <section className="card">
        <h3>今天</h3>
        <Big ms={todayRow?.total_ms ?? 0} />
        {todayRow && todayRow.total_ms > 0 && (
          <>
            <div className="split">
              {todayRow.subjects.map((s) => (
                <i key={s.subject_id} style={{ flex: s.ms, background: color(s.subject_id) }} />
              ))}
            </div>
            <div className="legend">
              {todayRow.subjects.map((s) => (
                <span key={s.subject_id}>
                  <Dot color={color(s.subject_id)} />
                  {name(s.subject_id)} {durationShort(s.ms)}
                </span>
              ))}
            </div>
          </>
        )}
      </section>

      <section className="card">
        <h3>这一周</h3>
        <Big ms={data.week.total_ms} />
        <p className="balance">
          {data.week.days_studied > 0 && `学了 ${data.week.days_studied} 天。`}
          {data.balance}
        </p>
      </section>

      <section className="card">
        <h3>
          {span === "week" ? "最近 7 天" : "本月"}
          <span className="card-tools">
            <button type="button" className={span === "week" ? "on" : ""} onClick={() => setSpan("week")}>
              7 天
            </button>
            <button type="button" className={span === "month" ? "on" : ""} onClick={() => setSpan("month")}>
              本月
            </button>
            <select
              aria-label="每日目标"
              value={hours}
              onChange={(e) => void attempt(api.settings.update({ daily_goal_ms: Number(e.target.value) * HOUR }))}
            >
              {goalChoices.map((h) => (
                <option key={h} value={h}>
                  {h === 0 ? "不设目标" : `每日 ${h} 小时`}
                </option>
              ))}
            </select>
          </span>
        </h3>
        <div className="bars">
          {trend.map((d) => (
            <button type="button" key={d.date} className="col" aria-label={d.date} onClick={() => open(d.date)}>
              {d.subjects.map((s) => (
                <i key={s.subject_id} style={{ height: `${barPercent(s.ms, max)}%`, background: color(s.subject_id) }} />
              ))}
            </button>
          ))}
          {goalMs > 0 && <div className="goal" style={{ bottom: `${barPercent(goalMs, max)}%` }} />}
        </div>
        <div className="days">
          {trend.map((d) => (
            <span key={d.date}>{d.date === today ? "今天" : span === "week" ? weekday(d.date).slice(1) : Number(d.date.slice(8))}</span>
          ))}
        </div>
      </section>

      <section className="card">
        <h3>最近 16 周</h3>
        <div className="heat">
          {data.days.map((d) => {
            const level = heatLevel(d.total_ms, peak);
            return (
              <button
                type="button"
                key={d.date}
                className={level ? "lit" : ""}
                style={{ ["--a" as string]: HEAT_ALPHA[level] }}
                aria-label={`${d.date} ${d.total_ms > 0 ? durationShort(d.total_ms) : "没有记录"}`}
                onClick={() => open(d.date)}
              />
            );
          })}
        </div>
        <p className="balance">这个月学了 {data.month.days_studied} 天</p>
      </section>
    </div>
  );
}

function Big({ ms }: { ms: number }) {
  return (
    <div className="big">
      {durationParts(ms).map((p) => (
        <span key={p.unit}>
          {p.n}
          <small>{p.unit}</small>
        </span>
      ))}
    </div>
  );
}
