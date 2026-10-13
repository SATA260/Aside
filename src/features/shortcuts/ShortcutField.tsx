import { type KeyboardEvent, useState } from "react";
import { api } from "../../shared/api";
import { play } from "../../shared/sfx";
import type { ShortcutAction } from "../../shared/types";
import { attempt } from "../../shared/ui/toast";
import { formatAccel, isMac, recordKey } from "./keys";

/** 点一下开始录，按下新的组合键就换上；Esc 或点别处取消。 */
export function ShortcutField({ action, accel, label }: { action: ShortcutAction; accel: string; label: string }) {
  const [recording, setRecording] = useState(false);
  const [held, setHeld] = useState("");
  const [nudge, setNudge] = useState(false);

  const reset = () => {
    setRecording(false);
    setHeld("");
    setNudge(false);
  };
  const begin = () => {
    if (recording) return;
    play("tap");
    setRecording(true);
    void attempt(api.settings.pauseShortcuts(true));
  };
  const cancel = () => {
    if (!recording) return;
    reset();
    void attempt(api.settings.pauseShortcuts(false));
  };

  const onKey = (e: KeyboardEvent<HTMLButtonElement>, down: boolean) => {
    if (!recording) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.repeat) return;
    const r = recordKey(e);
    if (r.kind === "partial") {
      setHeld(r.accel);
      return;
    }
    if (!down) return;
    if (r.kind === "cancel") cancel();
    else if (r.kind === "bare") {
      setHeld("");
      setNudge(true);
    } else {
      reset();
      void attempt(api.settings.setShortcut(action, r.accel)).then((saved) => saved && play("tap"));
    }
  };

  const shown = recording ? (held ? `${formatAccel(held)}${isMac ? "" : "+"}…` : "按下新的组合键") : formatAccel(accel);
  return (
    <div className="key-field-wrap">
      <button
        type="button"
        aria-label={`${label}的快捷键`}
        className={recording ? "key-field recording" : "key-field"}
        onClick={begin}
        onBlur={cancel}
        onKeyDown={(e) => onKey(e, true)}
        onKeyUp={(e) => onKey(e, false)}
      >
        {shown}
      </button>
      {recording && (
        <div className={nudge ? "key-field-tip nudge" : "key-field-tip"}>
          {nudge ? `要同时按住 ${isMac ? "⌘、⌥ 或 ⌃" : "Ctrl、Alt 或 Win"}` : "Esc 取消"}
        </div>
      )}
    </div>
  );
}
