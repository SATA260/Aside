import { describe, expect, it } from "vitest";
import { pose } from "../../src/features/character/pose";

describe("penguin pose", () => {
  it("follows the timer and the last five minutes", () => {
    expect(pose("running", 10 * 60_000, false)).toBe("focus");
    expect(pose("paused", 10 * 60_000, false)).toBe("pause");
    expect(pose("running", 4 * 60_000, false)).toBe("last5");
    expect(pose("running", 0, false)).toBe("focus");
    expect(pose("finished", null, false)).toBe("done");
    expect(pose("running", 4 * 60_000, true)).toBe("blink");
  });
});
