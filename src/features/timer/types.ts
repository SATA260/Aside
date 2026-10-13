export type TimerPhase = "idle" | "running" | "paused" | "finished";

export interface TimerSnapshot {
  phase: TimerPhase;
  focused_ms: number;
  planned_ms: number | null;
}
