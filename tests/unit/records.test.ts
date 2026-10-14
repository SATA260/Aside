import { describe, expect, it } from "vitest";
import { addDays, dayLabel, dayTitle, monthGrid, toDateStr } from "../../src/features/records/dates";
import { axisMax, barPercent, heatLevel } from "../../src/features/records/charts";
import { atLocal, rangeFrom, wallMinutes } from "../../src/features/records/span";
import { blocks, crossesMidnight } from "../../src/features/records/timeline";

const at = (d: number, h: number, m = 0) => new Date(2026, 9, d, h, m).getTime();
const MIN = 60_000;

describe("dates", () => {
  it("turns a local time into its date string", () => {
    expect(toDateStr(new Date(2026, 9, 4, 23, 59))).toBe("2026-10-04");
  });

  it("steps across month and year ends", () => {
    expect(addDays("2026-10-31", 1)).toBe("2026-11-01");
    expect(addDays("2026-01-01", -1)).toBe("2025-12-31");
  });

  it("titles a day by month and day", () => {
    expect(dayTitle("2026-10-13")).toBe("10 月 13 日");
  });

  it("labels the weekday and today or yesterday", () => {
    expect(dayLabel("2026-10-14", "2026-10-14")).toBe("周三 · 今天");
    expect(dayLabel("2026-10-13", "2026-10-14")).toBe("周二 · 昨天");
    expect(dayLabel("2026-10-11", "2026-10-14")).toBe("周日");
  });

  it("lays a month out in weeks starting on Monday", () => {
    const grid = monthGrid(2026, 10);
    // 2026-10-01 是周四，前面空三格
    expect(grid.slice(0, 4)).toEqual([null, null, null, "2026-10-01"]);
    expect(grid).toHaveLength(35);
    expect(grid.filter(Boolean)).toHaveLength(31);
    expect(grid.slice(-2)).toEqual(["2026-10-31", null]);
  });
});

describe("span", () => {
  it("builds a range from a local date, a clock time and whole minutes", () => {
    const started = new Date(2026, 9, 13, 19, 30).getTime();
    expect(atLocal("2026-10-13", "19:30")).toBe(started);
    expect(atLocal("2026-10-13", "19:30:00")).toBe(started);
    expect(rangeFrom("2026-10-13", "19:30", 90)).toEqual({ started_at: started, ended_at: started + 90 * MIN });
    expect(wallMinutes(started, started + 90 * MIN)).toBe(90);
  });

  it("rejects a bad clock time or a length outside 1 minute to 24 hours", () => {
    expect(atLocal("2026-10-13", "24:00")).toBeNull();
    expect(atLocal("2026-10-13", "7")).toBeNull();
    expect(rangeFrom("2026-10-13", "19:30", 0)).toBeNull();
    expect(rangeFrom("2026-10-13", "19:30", 24 * 60)).toEqual({
      started_at: new Date(2026, 9, 13, 19, 30).getTime(),
      ended_at: new Date(2026, 9, 14, 19, 30).getTime(),
    });
    expect(rangeFrom("2026-10-13", "19:30", 24 * 60 + 1)).toBeNull();
    expect(rangeFrom("2026-10-13", "19:30", 1.5)).toBeNull();
  });
});

describe("charts", () => {
  it("leaves an empty day blank and steps the rest by how full the busiest day was", () => {
    expect(heatLevel(0, 4 * 60 * MIN)).toBe(0);
    expect(heatLevel(30 * MIN, 0)).toBe(0);
    expect(heatLevel(60 * MIN, 4 * 60 * MIN)).toBe(1);
    expect(heatLevel(2 * 60 * MIN, 4 * 60 * MIN)).toBe(2);
    expect(heatLevel(3 * 60 * MIN, 4 * 60 * MIN)).toBe(3);
    expect(heatLevel(4 * 60 * MIN, 4 * 60 * MIN)).toBe(4);
  });

  it("sizes the axis to the taller of the days and the daily goal", () => {
    expect(axisMax([60 * MIN, 2 * 60 * MIN], 6 * 60 * MIN)).toBe(6 * 60 * MIN);
    expect(axisMax([8 * 60 * MIN], 6 * 60 * MIN)).toBe(8 * 60 * MIN);
    expect(barPercent(3 * 60 * MIN, 6 * 60 * MIN)).toBe(50);
    expect(barPercent(0, 6 * 60 * MIN)).toBe(0);
  });
});

describe("timeline", () => {
  const dayStart = at(13, 0);
  const dayEnd = at(14, 0);

  it("places each running segment by its share of the day", () => {
    const entry = {
      id: 1,
      subject_color: "#8DB59A",
      segments: [
        { start: at(13, 6), end: at(13, 12) },
        { start: at(13, 18), end: at(13, 18, 36) },
      ],
    };
    expect(blocks([entry], dayStart, dayEnd)).toEqual([
      { id: 1, color: "#8DB59A", left: 25, width: 25 },
      { id: 1, color: "#8DB59A", left: 75, width: 2.5 },
    ]);
  });

  it("clips segments to the day and drops the ones outside it", () => {
    const entry = {
      id: 2,
      subject_color: "#7FA3C7",
      segments: [
        { start: at(12, 20), end: at(12, 21) },
        { start: at(12, 23), end: at(13, 3) },
      ],
    };
    expect(blocks([entry], dayStart, dayEnd)).toEqual([{ id: 2, color: "#7FA3C7", left: 0, width: 12.5 }]);
  });

  it("tells when an entry runs past either midnight", () => {
    expect(crossesMidnight({ started_at: at(12, 23, 30), ended_at: at(13, 0, 30) }, dayStart, dayEnd)).toBe(true);
    expect(crossesMidnight({ started_at: at(13, 23, 30), ended_at: at(14, 0, 30) }, dayStart, dayEnd)).toBe(true);
    expect(crossesMidnight({ started_at: at(13, 8), ended_at: at(13, 8) + 45 * MIN }, dayStart, dayEnd)).toBe(false);
  });
});
