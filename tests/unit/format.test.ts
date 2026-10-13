import { describe, expect, it } from "vitest";
import { clock, durationParts, durationShort, hm } from "../../src/shared/format";

const S = 1000;
const M = 60 * S;
const H = 60 * M;

describe("clock", () => {
  it("shows minutes and seconds under an hour", () => {
    expect(clock(32 * M + 48 * S)).toBe("32:48");
    expect(clock(5 * S)).toBe("00:05");
    expect(clock(0)).toBe("00:00");
  });

  it("shows hours from one hour on", () => {
    expect(clock(H)).toBe("1:00:00");
    expect(clock(2 * H + 30 * M)).toBe("2:30:00");
    expect(clock(24 * H)).toBe("24:00:00");
  });

  it("rounds down by default and up for countdowns", () => {
    expect(clock(59 * M + 59 * S + 400)).toBe("59:59");
    expect(clock(44 * M + 59 * S + 300, "ceil")).toBe("45:00");
    expect(clock(59 * M + 59 * S + 300, "ceil")).toBe("1:00:00");
  });

  it("never shows negative time", () => {
    expect(clock(-5 * S)).toBe("00:00");
  });
});

describe("durationParts", () => {
  it("uses 分钟 under an hour", () => {
    expect(durationParts(45 * M)).toEqual([{ n: 45, unit: "分钟" }]);
    expect(durationParts(M + 59 * S)).toEqual([{ n: 1, unit: "分钟" }]);
  });

  it("uses 小时 and 分 from an hour on, dropping zero minutes", () => {
    expect(durationParts(H + 30 * M)).toEqual([
      { n: 1, unit: "小时" },
      { n: 30, unit: "分" },
    ]);
    expect(durationParts(2 * H)).toEqual([{ n: 2, unit: "小时" }]);
  });

  it("shows 0 分钟 for less than a minute", () => {
    expect(durationParts(20 * S)).toEqual([{ n: 0, unit: "分钟" }]);
  });
});

describe("durationShort", () => {
  it("is compact for lists", () => {
    expect(durationShort(45 * M)).toBe("45 分");
    expect(durationShort(2 * H + 30 * M)).toBe("2 时 30 分");
    expect(durationShort(3 * H)).toBe("3 时");
  });
});

describe("hm", () => {
  it("formats a local time as HH:MM", () => {
    const t = new Date(2026, 9, 13, 7, 5).getTime();
    expect(hm(t)).toBe("07:05");
  });
});
