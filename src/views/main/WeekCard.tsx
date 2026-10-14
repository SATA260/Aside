import { useEffect, useRef, useState } from "react";
import { toPng } from "html-to-image";
import { QUOTES } from "../../features/atmosphere/quotes";
import { addDays, toDateStr } from "../../features/records/dates";
import { api } from "../../shared/api";
import { durationParts, durationShort } from "../../shared/format";
import type { DateStr, Subject, WeekReport } from "../../shared/types";
import { Button } from "../../shared/ui/controls";
import { attempt } from "../../shared/ui/toast";

/** 一张可以保存的周报。不含任务和备注。`from` 是这一周里的任意一天。 */
export function WeekCard({ from, onClose }: { from: DateStr; onClose: () => void }) {
  const card = useRef<HTMLDivElement>(null);
  const [anchor, setAnchor] = useState(from);
  const [data, setData] = useState<WeekReport | null>(null);
  const [subjects, setSubjects] = useState<Subject[]>([]);
  const [line, setLine] = useState(0);

  useEffect(() => {
    let live = true;
    void attempt(api.stats.week(anchor)).then((report) => live && report && setData(report));
    void attempt(api.subjects.list(true)).then((rows) => live && rows && setSubjects(rows));
    return () => {
      live = false;
    };
  }, [anchor]);

  if (!data) return null;
  const days = Array.from({ length: 7 }, (_, i) => addDays(data.week.from, i));
  const msOf = (date: string) => data.days.find((d) => d.date === date)?.total_ms ?? 0;
  const peak = Math.max(...data.week.subjects.map((s) => s.ms), 1);
  const name = (id: number) => subjects.find((s) => s.id === id)?.name ?? "";
  const color = (id: number) => subjects.find((s) => s.id === id)?.color ?? "#A3A9B4";
  const today = toDateStr(new Date());
  const next = addDays(data.week.from, 7);

  const save = async () => {
    if (!card.current) return;
    const url = await toPng(card.current, { pixelRatio: 2 });
    const a = document.createElement("a");
    a.href = url;
    a.download = `aside-${data.week.from}.png`;
    a.click();
  };

  return (
    <div className="week-stage">
      <div className="week" ref={card}>
        <div className="range">
          {data.week.from.slice(5).replace("-", ".")} — {data.week.to.slice(5).replace("-", ".")}
        </div>
        <h2>这一周，你坐下来了 {data.week.days_studied} 天</h2>
        <div className="total">
          {durationParts(data.week.total_ms).map((p) => (
            <span key={p.unit}>
              {p.n}
              <small>{p.unit}</small>
            </span>
          ))}
        </div>
        <div className="rows">
          {data.week.subjects.map((s) => (
            <div className="r" key={s.subject_id}>
              <span className="n">{name(s.subject_id)}</span>
              <span className="b">
                <i style={{ width: `${(s.ms / peak) * 100}%`, background: color(s.subject_id) }} />
              </span>
              <span className="v">{durationShort(s.ms)}</span>
            </div>
          ))}
        </div>
        <div className="seven">
          {days.map((d) => (
            <div key={d}>
              <i style={{ opacity: msOf(d) > 0 ? 0.35 + 0.65 * Math.min(1, msOf(d) / (6 * 3600_000)) : 0.08 }} />
              {"日一二三四五六"[new Date(d + "T00:00:00").getDay()]}
            </div>
          ))}
        </div>
        <p className="q">{QUOTES[line % QUOTES.length]}</p>
      </div>
      <div className="week-actions">
        <Button cue={null} onClick={() => setAnchor(addDays(data.week.from, -7))}>
          上一周
        </Button>
        <Button cue={null} disabled={next > today} onClick={() => setAnchor(next)}>
          下一周
        </Button>
        <Button onClick={() => setLine((n) => n + 1)}>换一句话</Button>
        <Button variant="primary" onClick={() => void save()}>
          保存为图片
        </Button>
        <Button cue={null} onClick={onClose}>
          关闭
        </Button>
      </div>
    </div>
  );
}
