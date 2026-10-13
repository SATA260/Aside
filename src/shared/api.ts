import { call } from "./ipc";
import type { Outcome, Settings, StartArgs, Subject, TimerView } from "./types";

export const api = {
  timer: {
    view: () => call<TimerView>("timer_view"),
    start: (args: StartArgs) => call<TimerView>("timer_start", args),
    pause: () => call<TimerView>("timer_pause"),
    resume: () => call<TimerView>("timer_resume"),
    finish: () => call<Outcome>("timer_finish"),
    close: (again: boolean) => call<TimerView>("timer_close", { again }),
    dismissRecovered: () => call<null>("timer_dismiss_recovered"),
  },
  settings: {
    get: () => call<Settings>("settings_get"),
    update: (patch: Partial<Settings>) => call<Settings>("settings_update", patch),
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
