import { describe, expect, it } from "vitest";
import { QUOTES, daysUntilExam, quoteFor } from "../../src/features/atmosphere/quotes";

describe("quoteFor", () => {
  it("keeps the same sentence all day and has a full set", () => {
    expect(QUOTES.length).toBeGreaterThanOrEqual(120);
    expect(quoteFor("2026-10-14")).toBe(quoteFor("2026-10-14"));
    expect(QUOTES).toContain(quoteFor("2026-10-14"));
  });

  it("moves to another sentence on another day", () => {
    expect(quoteFor("2026-10-15")).not.toBe(quoteFor("2026-10-14"));
  });
});

describe("daysUntilExam", () => {
  it("counts whole days and hides a date that has passed", () => {
    expect(daysUntilExam("2026-10-14", "2026-12-21")).toBe(68);
    expect(daysUntilExam("2026-10-14", "2026-10-14")).toBe(0);
    expect(daysUntilExam("2026-10-14", "2026-10-13")).toBeNull();
  });
});
