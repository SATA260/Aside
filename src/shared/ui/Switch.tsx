import type { CSSProperties } from "react";
import { play } from "../sfx";

export function Switch({ on, onChange, label }: { on: boolean; onChange: (on: boolean) => void; label: string }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={label}
      className={on ? "switch on" : "switch"}
      onClick={() => {
        play("toggle");
        onChange(!on);
      }}
    />
  );
}

/** 0–1 的滑块，松手时才提交，避免拖动时连续写设置。 */
export function Slider({
  value,
  onCommit,
  onInput,
  label,
}: {
  value: number;
  onCommit: (v: number) => void;
  onInput?: (v: number) => void;
  label: string;
}) {
  return (
    <input
      type="range"
      className="slider"
      aria-label={label}
      min={0}
      max={1}
      step={0.01}
      defaultValue={value}
      key={value}
      style={{ "--v": `${value * 100}%` } as CSSProperties}
      onInput={(e) => {
        const v = Number(e.currentTarget.value);
        e.currentTarget.style.setProperty("--v", `${v * 100}%`);
        onInput?.(v);
      }}
      onPointerUp={(e) => onCommit(Number(e.currentTarget.value))}
      onKeyUp={(e) => onCommit(Number(e.currentTarget.value))}
    />
  );
}
