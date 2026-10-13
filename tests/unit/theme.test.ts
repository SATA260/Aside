import { describe, expect, it } from "vitest";
import { resolveTheme } from "../../src/shared/theme/resolve";

describe("resolveTheme", () => {
  it("keeps a fixed theme as is", () => {
    expect(resolveTheme("bamboo", 3)).toBe("bamboo");
    expect(resolveTheme("mist", 23)).toBe("mist");
  });

  it("follows the sky: misty dawn, clear day, dusk, night", () => {
    expect(resolveTheme("sky", 5)).toBe("mist");
    expect(resolveTheme("sky", 7)).toBe("mist");
    expect(resolveTheme("sky", 8)).toBe("clear");
    expect(resolveTheme("sky", 16)).toBe("clear");
    expect(resolveTheme("sky", 17)).toBe("dusk");
    expect(resolveTheme("sky", 18)).toBe("dusk");
    expect(resolveTheme("sky", 19)).toBe("night");
    expect(resolveTheme("sky", 0)).toBe("night");
    expect(resolveTheme("sky", 4)).toBe("night");
  });
});
