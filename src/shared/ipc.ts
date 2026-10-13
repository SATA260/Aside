import { invoke } from "@tauri-apps/api/core";
import type { TimerSnapshot } from "../features/timer/types";

export const ipc = {
  timerSnapshot: () => invoke<TimerSnapshot>("timer_snapshot"),
};
