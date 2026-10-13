// 每个窗口各有一份：当前计时、设置、科目，都跟着后端广播的事件同步。

import { create } from "zustand";
import { api } from "./api";
import { on } from "./ipc";
import type { Settings, Subject, TimerView } from "./types";

interface AppState {
  timer: TimerView | null;
  settings: Settings | null;
  subjects: Subject[];
  /** 记录或科目有变化时加一，需要重新拉数据的页面监听它 */
  dataVersion: number;
}

export const useApp = create<AppState>(() => ({
  timer: null,
  settings: null,
  subjects: [],
  dataVersion: 0,
}));

let started = false;

export function startSync() {
  if (started) return;
  started = true;
  const refresh = () =>
    Promise.all([api.timer.view(), api.subjects.list()]).then(([timer, subjects]) =>
      useApp.setState((s) => ({ timer, subjects, dataVersion: s.dataVersion + 1 })),
    );
  on<TimerView>("timer:tick", (timer) => useApp.setState({ timer }));
  on<Settings>("settings:changed", (settings) => useApp.setState({ settings }));
  on("data:changed", () => void refresh());
  void Promise.all([api.timer.view(), api.settings.get(), api.subjects.list()]).then(
    ([timer, settings, subjects]) => useApp.setState({ timer, settings, subjects }),
  );
}

export function setTimer(timer: TimerView) {
  useApp.setState({ timer });
}
