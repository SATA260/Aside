/** 快捷键的录制和显示。组合键用后端认得的写法存，例如 `Alt+Super+S`。 */

export const isMac = /Mac/i.test(navigator.userAgent);

export interface KeyLike {
  code: string;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

export type Recorded =
  | { kind: "done"; accel: string }
  /** 只按住了修饰键，还在等普通键 */
  | { kind: "partial"; accel: string }
  /** 没按 ⌘ / Ctrl / Alt，打字时会被抢走，不收 */
  | { kind: "bare" }
  | { kind: "cancel" };

const MODIFIER_CODES = /^(Meta|Control|Alt|Shift|OS)(Left|Right)?$/;

export function recordKey(e: KeyLike): Recorded {
  const mods = [e.ctrlKey && "Control", e.altKey && "Alt", e.shiftKey && "Shift", e.metaKey && "Super"].filter(
    Boolean,
  ) as string[];
  if (MODIFIER_CODES.test(e.code)) return { kind: "partial", accel: mods.join("+") };
  if (!e.ctrlKey && !e.altKey && !e.metaKey) {
    return e.code === "Escape" && !e.shiftKey ? { kind: "cancel" } : { kind: "bare" };
  }
  const keyName = e.code.replace(/^(Key|Digit)(?=.$)/, "");
  return { kind: "done", accel: [...mods, keyName].join("+") };
}

const SYMBOLS: Record<string, string> = {
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  Comma: ",",
  Period: ".",
  Slash: "/",
  Backslash: "\\",
  Semicolon: ";",
  Quote: "'",
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Backquote: "`",
};

type Mod = "control" | "alt" | "shift" | "super";
const MOD_ORDER: Mod[] = ["control", "alt", "shift", "super"];
const MAC_MOD: Record<Mod, string> = { control: "⌃", alt: "⌥", shift: "⇧", super: "⌘" };
const WIN_MOD: Record<Mod, string> = { control: "Ctrl", alt: "Alt", shift: "Shift", super: "Win" };

function modOf(token: string, mac: boolean): Mod | null {
  switch (token.toLowerCase()) {
    case "commandorcontrol":
    case "commandorctrl":
    case "cmdorctrl":
    case "cmdorcontrol":
      return mac ? "super" : "control";
    case "super":
    case "cmd":
    case "command":
      return "super";
    case "control":
    case "ctrl":
      return "control";
    case "alt":
    case "option":
      return "alt";
    case "shift":
      return "shift";
    default:
      return null;
  }
}

/** 界面上显示的样子：macOS 用 ⌃⌥⇧⌘ 符号，Windows 写成 Ctrl+Alt+S。 */
export function formatAccel(accel: string, mac: boolean = isMac): string {
  const mods = new Set<Mod>();
  let key = "";
  for (const token of accel.split("+").map((t) => t.trim())) {
    const mod = modOf(token, mac);
    if (mod) mods.add(mod);
    else key = token.replace(/^(Key|Digit)(?=.$)/, "");
  }
  key = SYMBOLS[key] ?? key;
  const names = MOD_ORDER.filter((m) => mods.has(m)).map((m) => (mac ? MAC_MOD : WIN_MOD)[m]);
  return mac ? names.join("") + key : [...names, key].filter(Boolean).join("+");
}
