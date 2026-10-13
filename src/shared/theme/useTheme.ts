import { useEffect } from "react";
import type { ThemeName } from "../types";
import { resolveTheme } from "./resolve";

/** 把主题写到 <html data-theme>；跟随天色时每分钟重新判断一次。 */
export function useTheme(theme: ThemeName | undefined) {
  useEffect(() => {
    if (!theme) return;
    const apply = () => {
      document.documentElement.dataset.theme = resolveTheme(theme, new Date().getHours());
    };
    apply();
    if (theme !== "sky") return;
    const id = setInterval(apply, 60_000);
    return () => clearInterval(id);
  }, [theme]);
}
