import { useEffect } from "react";
import { configureSfx } from "../shared/sfx";
import { startSync, useApp } from "../shared/state";
import { useTheme } from "../shared/theme/useTheme";
import { Toaster } from "../shared/ui/toast";
import { MainView } from "../views/main/MainView";
import { MiniView } from "../views/mini/MiniView";
import { OverlayView } from "../views/overlay/OverlayView";

const VIEWS = {
  main: MainView,
  overlay: OverlayView,
  mini: MiniView,
} as const;

type ViewName = keyof typeof VIEWS;

// 每个窗口都加载同一个 index.html，后端创建窗口时用 ?view= 指定显示哪个界面
export function currentView(): ViewName {
  const view = new URLSearchParams(window.location.search).get("view");
  return view && view in VIEWS ? (view as ViewName) : "main";
}

export function App() {
  const settings = useApp((s) => s.settings);
  useEffect(() => startSync(), []);
  useTheme();
  useEffect(() => {
    if (settings) configureSfx({ enabled: settings.sfx_enabled, volume: settings.sfx_volume });
  }, [settings]);

  const view = currentView();
  const View = VIEWS[view];
  useEffect(() => {
    document.documentElement.dataset.view = view;
  }, [view]);

  return (
    <>
      <View />
      <Toaster />
    </>
  );
}
