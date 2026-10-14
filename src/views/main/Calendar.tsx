import { useEffect, useState } from "react";
import { monthGrid, weekday } from "../../features/records/dates";
import type { DateStr } from "../../shared/types";

const HEADS = ["一", "二", "三", "四", "五", "六", "日"];

export function Calendar({
  value,
  today,
  onPick,
  onClose,
}: {
  value: DateStr;
  today: DateStr;
  onPick: (d: DateStr) => void;
  onClose: () => void;
}) {
  const [cursor, setCursor] = useState(value.slice(0, 7));
  useEffect(() => setCursor(value.slice(0, 7)), [value]);

  const [year, month] = cursor.split("-").map(Number);
  const cells = monthGrid(year, month);
  const title = `${year} 年 ${month} 月`;
  const shift = (n: number) => {
    const d = new Date(year, month - 1 + n, 1);
    setCursor(`${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`);
  };
  const nextMonth = `${month === 12 ? year + 1 : year}-${String(month === 12 ? 1 : month + 1).padStart(2, "0")}`;
  const nextIsFuture = `${nextMonth}-01` > today;

  return (
    <>
      <button type="button" className="cal-backdrop" aria-label="关闭日历" onClick={onClose} />
      <div className="cal" role="dialog" aria-label="选择日期">
        <div className="cal-head">
          <button type="button" aria-label="上个月" onClick={() => shift(-1)}>
            ‹
          </button>
          <span>{title}</span>
          <button type="button" aria-label="下个月" disabled={nextIsFuture} onClick={() => shift(1)}>
            ›
          </button>
        </div>
        <div className="cal-grid heads">
          {HEADS.map((h) => (
            <span key={h}>{h}</span>
          ))}
        </div>
        <div className="cal-grid">
          {cells.map((d, i) =>
            d ? (
              <button
                type="button"
                key={d}
                disabled={d > today}
                className={[d === value ? "on" : "", d === today ? "today" : ""].filter(Boolean).join(" ")}
                aria-label={`${d} ${weekday(d)}`}
                onClick={() => onPick(d)}
              >
                {Number(d.slice(8))}
              </button>
            ) : (
              <span key={`e${i}`} />
            ),
          )}
        </div>
      </div>
    </>
  );
}
