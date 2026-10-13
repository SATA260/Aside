import type { ButtonHTMLAttributes, CSSProperties, ReactNode } from "react";
import { type Cue, play } from "../sfx";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "plain";
  size?: "small";
  cue?: Cue | null;
};

/** 所有能点的地方都带一声轻轻的反馈音。 */
export function Button({ variant = "plain", size, cue = "tap", className = "", onClick, ...rest }: ButtonProps) {
  const cls = ["btn", variant === "primary" ? "primary" : "", size ?? "", className].filter(Boolean).join(" ");
  return (
    <button
      type="button"
      className={cls}
      onClick={(e) => {
        if (cue) play(cue);
        onClick?.(e);
      }}
      {...rest}
    />
  );
}

export function TextButton({ cue = "tap", className = "link", onClick, ...rest }: ButtonProps) {
  return (
    <button
      type="button"
      className={className}
      onClick={(e) => {
        if (cue) play(cue);
        onClick?.(e);
      }}
      {...rest}
    />
  );
}

export function Chip({
  on,
  color,
  className = "",
  children,
  onClick,
  title,
}: {
  on?: boolean;
  color?: string;
  className?: string;
  children: ReactNode;
  onClick?: () => void;
  title?: string;
}) {
  return (
    <button
      type="button"
      title={title}
      className={`chip ${on ? "on" : ""} ${className}`}
      style={color ? ({ "--c": color } as CSSProperties) : undefined}
      onClick={() => {
        play("tap");
        onClick?.();
      }}
    >
      {children}
    </button>
  );
}

export function Segmented<T extends string>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (v: T) => void;
}) {
  return (
    <div className="seg" role="radiogroup">
      {options.map((o) => (
        <button
          type="button"
          role="radio"
          aria-checked={o.value === value}
          key={o.value}
          className={o.value === value ? "on" : ""}
          onClick={() => {
            if (o.value !== value) play("toggle");
            onChange(o.value);
          }}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function Dot({ color }: { color?: string }) {
  return <i className="dot" style={color ? ({ "--c": color } as CSSProperties) : undefined} />;
}
