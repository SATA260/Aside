import { describe, expect, it } from "vitest";
import { moveItem, targetIndex } from "../../src/features/subjects/reorder";

describe("moveItem", () => {
  it("moves an item down", () => {
    expect(moveItem([1, 2, 3, 4], 0, 2)).toEqual([2, 3, 1, 4]);
  });

  it("moves an item up", () => {
    expect(moveItem([1, 2, 3, 4], 3, 1)).toEqual([1, 4, 2, 3]);
  });

  it("keeps the list when the index does not change", () => {
    expect(moveItem([1, 2, 3], 1, 1)).toEqual([1, 2, 3]);
  });

  it("does not mutate the input", () => {
    const ids = [1, 2, 3];
    moveItem(ids, 0, 2);
    expect(ids).toEqual([1, 2, 3]);
  });
});

describe("targetIndex", () => {
  it("rounds the drag distance to whole rows", () => {
    expect(targetIndex(1, 5, 70, 52)).toBe(2);
    expect(targetIndex(1, 5, 20, 52)).toBe(1);
    expect(targetIndex(3, 5, -110, 52)).toBe(1);
  });

  it("stays inside the list", () => {
    expect(targetIndex(0, 5, -500, 52)).toBe(0);
    expect(targetIndex(4, 5, 500, 52)).toBe(4);
  });
});
