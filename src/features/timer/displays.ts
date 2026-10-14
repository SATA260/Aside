import type { DisplayMode } from "../../shared/types";

export const DISPLAYS: { value: DisplayMode; label: string }[] = [
  { value: "fullscreen", label: "全屏" },
  { value: "mini", label: "浮窗" },
  { value: "lecture", label: "网课" },
];
