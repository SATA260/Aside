import { useEffect } from "react";
import { SCENES, type SceneId } from "../../features/atmosphere/catalog";
import { SceneBackdrop } from "../../features/atmosphere/SceneBackdrop";
import { imageSrc, useLibrary } from "../../features/atmosphere/library";
import { useAtmoPreview } from "../../features/atmosphere/preview";
import { api } from "../../shared/api";
import { on } from "../../shared/ipc";
import { play } from "../../shared/sfx";
import { useApp } from "../../shared/state";
import { EndScreen } from "./EndScreen";
import { FocusScreen } from "./FocusScreen";
import "./overlay.css";

/** 副屏上的全屏窗口只铺底色，计时内容和结束音效只在主屏。 */
const blank = new URLSearchParams(location.search).has("blank");

export function OverlayView() {
  const timer = useApp((s) => s.timer);
  const settings = useApp((s) => s.settings);

  useEffect(() => (blank ? undefined : on("timer:finished", () => play("end"))), []);

  useEffect(() => {
    const stop = () => void api.alarm.stop();
    window.addEventListener("pointerdown", stop);
    window.addEventListener("keydown", stop);
    return () => {
      window.removeEventListener("pointerdown", stop);
      window.removeEventListener("keydown", stop);
    };
  }, []);

  const ended = timer?.phase === "finished" && timer.last;
  const previewScene = useAtmoPreview((s) => s.scene);
  const previewPicture = useAtmoPreview((s) => s.picture);
  const { images } = useLibrary();
  const scene = (previewScene ?? SCENES.find((s) => s.id === settings?.scene)?.id ?? "sky") as SceneId;
  const hovering = previewScene !== null || previewPicture !== null;
  const saved = images.find((picture) => picture.id === settings?.scene_image);
  const photo = previewPicture ?? (hovering || !saved ? null : imageSrc(saved));
  return (
    <div className={ended ? `overlay ended ${timer.last?.reason}` : "overlay"}>
      <SceneBackdrop scene={scene} photo={photo} />
      <div className="sky" />
      {!blank && timer && settings && (timer.phase === "running" || timer.phase === "paused") && (
        <FocusScreen timer={timer} settings={settings} />
      )}
      {!blank && ended && timer.last && <EndScreen outcome={timer.last} subject={timer.subject} />}
    </div>
  );
}
