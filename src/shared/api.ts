import { call } from "./ipc";
import type { DisplayMode, Outcome, Settings, ShortcutAction, StartArgs, Subject, TimerView } from "./types";

export const api = {
  timer: {
    view: () => call<TimerView>("timer_view"),
    start: (args: StartArgs) => call<TimerView>("timer_start", args),
    pause: () => call<TimerView>("timer_pause"),
    resume: () => call<TimerView>("timer_resume"),
    finish: () => call<Outcome>("timer_finish"),
    close: (again: boolean) => call<TimerView>("timer_close", { again }),
    dismissRecovered: () => call<TimerView>("timer_dismiss_recovered"),
    switchDisplay: (display: DisplayMode) => call<TimerView>("timer_switch_display", { display }),
  },
  settings: {
    get: () => call<Settings>("settings_get"),
    update: (patch: Partial<Settings>) => call<Settings>("settings_update", patch),
    setShortcut: (action: ShortcutAction, accel: string) => call<Settings>("shortcut_set", { action, accel }),
    pauseShortcuts: (paused: boolean) => call<null>("shortcuts_pause", { paused }),
  },
  subjects: {
    list: (includeArchived = false) => call<Subject[]>("subjects_list", { include_archived: includeArchived }),
    create: (name: string, color: string) => call<Subject>("subject_create", { name, color }),
    update: (id: number, patch: { name?: string; color?: string }) => call<Subject>("subject_update", { id, ...patch }),
    archive: (id: number) => call<null>("subject_archive", { id }),
    restore: (id: number) => call<null>("subject_restore", { id }),
    reorder: (ids: number[]) => call<null>("subjects_reorder", { ids }),
  },
  sessions: {
    setNote: (id: number, note: string) => call<null>("session_set_note", { id, note }),
  },
};
