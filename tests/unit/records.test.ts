import { describe, expect, it } from "vitest";
import { addDays, dayLabel, dayTitle, monthGrid, toDateStr } from "../../src/features/records/dates";
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
