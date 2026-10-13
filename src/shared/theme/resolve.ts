import type { ThemeName } from "../types";

/** 实际用到的配色。“晴灰”只在跟随天色时出现。 */
export type Palette = "night" | "bamboo" | "mist" | "dusk" | "clear";

/** 跟随天色：清晨雾蓝，白天晴灰，傍晚暮紫，夜里夜读。 */
export function resolveTheme(theme: ThemeName, hour: number): Palette {
  if (theme !== "sky") return theme;
  if (hour >= 5 && hour < 8) return "mist";
  if (hour >= 8 && hour < 17) return "clear";
  if (hour >= 17 && hour < 19) return "dusk";
  return "night";
}
