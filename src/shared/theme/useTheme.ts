import { useEffect } from "react";

/** 界面固定用夜读配色。全屏页的字色跟着画面走，不再单独选风格。 */
export function useTheme() {
  useEffect(() => {
    document.documentElement.dataset.theme = "night";
  }, []);
}
