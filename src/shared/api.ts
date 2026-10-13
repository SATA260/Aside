import { call } from "./ipc";
import type { TimerSnapshot } from "../features/timer/types";

export const api = {
  timer: {
    snapshot: () => call<TimerSnapshot>("timer_snapshot"),
  },
};
