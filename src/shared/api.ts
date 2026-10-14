import { call } from "./ipc";
import type {
  DateStr,
  DayTotal,
  DayView,
  DisplayMode,
  ImportedImage,
  ImportedSound,
  Outcome,
  Overview,
  WeekPrompt,
  WeekReport,
  Bookmark,
  CharacterInfo,
  Session,
  SessionPatch,
  Settings,
  ShortcutAction,
  StartArgs,
  Subject,
  TimerView,
} from "./types";

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
  autostart: {
    get: () => call<boolean>("autostart_get"),
    set: (on: boolean) => call<boolean>("autostart_set", { on }),
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
    setScore: (id: number, score: number, fullScore: number) =>
      call<Session>("session_set_score", { id, score, full_score: fullScore }),
    day: (date: DateStr) => call<DayView>("day_view", { date }),
    add: (entry: { subject_id: number; started_at: number; ended_at: number; task?: string; note?: string }) =>
      call<Session>("session_add", entry),
    update: (id: number, patch: SessionPatch) => call<Session>("session_update", { id, ...patch }),
    remove: (id: number) => call<null>("session_delete", { id }),
  },
  stats: {
    overview: () => call<Overview>("stats_overview"),
    days: (from: DateStr, to: DateStr) => call<DayTotal[]>("stats_days", { from, to }),
    week: (date: DateStr) => call<WeekReport>("stats_week", { date }),
    prompt: () => call<WeekPrompt | null>("week_prompt"),
  },
  alarm: {
    stop: () => call<null>("alarm_stop"),
  },
  media: {
    sounds: () => call<ImportedSound[]>("media_sounds"),
    images: () => call<ImportedImage[]>("media_images"),
    importSound: (path: string, name?: string) => call<ImportedSound>("media_import_sound", { path, name }),
    importImage: (path: string, name?: string) => call<ImportedImage>("media_import_image", { path, name }),
    renameSound: (id: number, name: string) => call<null>("media_rename_sound", { id, name }),
    deleteSound: (id: number) => call<null>("media_delete_sound", { id }),
    renameImage: (id: number, name: string) => call<null>("media_rename_image", { id, name }),
    deleteImage: (id: number) => call<null>("media_delete_image", { id }),
  },
  character: {
    list: () => call<CharacterInfo[]>("characters_list"),
    import: (path: string) => call<CharacterInfo>("character_import", { url: path }),
  },
  lecture: {
    bookmarks: () => call<Bookmark[]>("lecture_bookmarks"),
    go: (url: string) => call<string>("lecture_go", { url }),
    video: (on: boolean, time: string) => call<null>("lecture_video", { on, time }),
  },
};
