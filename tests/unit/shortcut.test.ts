import { describe, expect, it } from "vitest";
import { formatAccel, recordKey } from "../../src/features/shortcuts/keys";

const key = (code: string, mods: Partial<Record<"metaKey" | "ctrlKey" | "altKey" | "shiftKey", boolean>> = {}) => ({
  code,
  metaKey: false,
  ctrlKey: false,
  altKey: false,
  shiftKey: false,
  ...mods,
});

describe("recordKey", () => {
  it("turns a combination into an accelerator with modifiers in a fixed order", () => {
    expect(recordKey(key("KeyS", { metaKey: true, altKey: true }))).toEqual({ kind: "done", accel: "Alt+Super+S" });
    expect(recordKey(key("Digit1", { shiftKey: true, ctrlKey: true }))).toEqual({ kind: "done", accel: "Control+Shift+1" });
    expect(recordKey(key("ArrowUp", { altKey: true }))).toEqual({ kind: "done", accel: "Alt+ArrowUp" });
    expect(recordKey(key("F5", { ctrlKey: true }))).toEqual({ kind: "done", accel: "Control+F5" });
  });

  it("waits while only modifiers are held", () => {
    expect(recordKey(key("MetaLeft", { metaKey: true, altKey: true }))).toEqual({ kind: "partial", accel: "Alt+Super" });
    expect(recordKey(key("ShiftRight", { shiftKey: true }))).toEqual({ kind: "partial", accel: "Shift" });
  });

  it("asks for a modifier when a plain key is pressed", () => {
    expect(recordKey(key("KeyS"))).toEqual({ kind: "bare" });
    // Shift 单独不算，否则打大写字母会被抢走
    expect(recordKey(key("KeyS", { shiftKey: true }))).toEqual({ kind: "bare" });
  });

  it("cancels on a bare Escape", () => {
    expect(recordKey(key("Escape"))).toEqual({ kind: "cancel" });
    expect(recordKey(key("Escape", { ctrlKey: true }))).toEqual({ kind: "done", accel: "Control+Escape" });
  });
});

describe("formatAccel", () => {
  it("uses symbols in the macOS order", () => {
    expect(formatAccel("CommandOrControl+Alt+S", true)).toBe("⌥⌘S");
    expect(formatAccel("Alt+Super+S", true)).toBe("⌥⌘S");
    expect(formatAccel("Shift+Control+KeyM", true)).toBe("⌃⇧M");
    expect(formatAccel("Alt+ArrowUp", true)).toBe("⌥↑");
  });

  it("spells keys out on Windows", () => {
    expect(formatAccel("CommandOrControl+Alt+S", false)).toBe("Ctrl+Alt+S");
    expect(formatAccel("Super+Shift+Digit2", false)).toBe("Shift+Win+2");
    expect(formatAccel("Control+Comma", false)).toBe("Ctrl+,");
    expect(formatAccel("Control+Space", false)).toBe("Ctrl+Space");
  });

  it("shows held modifiers before the key is pressed", () => {
    expect(formatAccel("Alt+Super", true)).toBe("⌥⌘");
    expect(formatAccel("Control+Alt", false)).toBe("Ctrl+Alt");
  });
});
