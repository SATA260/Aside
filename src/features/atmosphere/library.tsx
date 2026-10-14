import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { api } from "../../shared/api";
import { inTauri } from "../../shared/ipc";
import { useApp } from "../../shared/state";
import type { ImportedImage, ImportedSound } from "../../shared/types";
import { attempt } from "../../shared/ui/toast";
import { SCENES, SOUNDS, type SceneId } from "./catalog";
import "./library.css";

export function useLibrary() {
  const dataVersion = useApp((s) => s.dataVersion);
  const [sounds, setSounds] = useState<ImportedSound[]>([]);
  const [images, setImages] = useState<ImportedImage[]>([]);
  useEffect(() => {
    let live = true;
    void api.media.sounds().then((rows) => live && setSounds(rows));
    void api.media.images().then((rows) => live && setImages(rows));
    return () => {
      live = false;
    };
  }, [dataVersion]);
  return { sounds, images };
}

export function imageSrc(picture: ImportedImage): string {
  if (!inTauri) return `/__dev/media/image/${picture.id}`;
  return convertFileSrc(picture.path);
}

function pickFile(accept: string): Promise<File | null> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = accept;
    input.onchange = () => resolve(input.files?.[0] ?? null);
    input.click();
  });
}

export async function importKind(kind: "sound" | "image") {
  if (inTauri) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const picked = await open({
      multiple: false,
      filters:
        kind === "sound"
          ? [{ name: "声音", extensions: ["mp3", "m4a", "wav", "flac"] }]
          : [{ name: "图片", extensions: ["jpg", "jpeg", "png", "webp"] }],
    });
    if (typeof picked !== "string") return;
    if (kind === "sound") await api.media.importSound(picked);
    else await api.media.importImage(picked);
    return;
  }
  const file = await pickFile(kind === "sound" ? ".mp3,.m4a,.wav,.flac" : ".jpg,.jpeg,.png,.webp");
  if (!file) return;
  const res = await fetch("/__dev/import", {
    method: "POST",
    headers: { "x-kind": kind, "x-name": encodeURIComponent(file.name) },
    body: file,
  });
  const body = (await res.json()) as { ok: boolean; error?: string };
  if (!body.ok) throw new Error(body.error ?? "没能导入");
}

function MineMark({
  label,
  on,
  onPick,
  onRename,
  onDelete,
  onEnter,
  onLeave,
}: {
  label: string;
  on: boolean;
  onPick: () => void;
  onRename: (name: string) => void;
  onDelete: () => void;
  onEnter?: () => void;
  onLeave?: () => void;
}) {
  const [menu, setMenu] = useState(false);
  const [name, setName] = useState(label);
  useEffect(() => setName(label), [label]);

  return (
    <span
      className="mine-wrap"
      onMouseEnter={onEnter}
      onMouseLeave={onLeave}
      onContextMenu={(e) => {
        e.preventDefault();
        setMenu(true);
      }}
    >
      {menu ? (
        <input
          className="input small"
          aria-label="改名"
          value={name}
          autoFocus
          onChange={(e) => setName(e.target.value)}
          onBlur={() => {
            setMenu(false);
            if (name.trim() && name.trim() !== label) onRename(name.trim());
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter") (e.target as HTMLInputElement).blur();
            if (e.key === "Escape") {
              setName(label);
              setMenu(false);
            }
          }}
        />
      ) : (
        <button
          type="button"
          className={on ? "on" : ""}
          onClick={onPick}
          onPointerDown={() => {
            const timer = window.setTimeout(() => setMenu(true), 500);
            const up = () => window.clearTimeout(timer);
            window.addEventListener("pointerup", up, { once: true });
            window.addEventListener("pointercancel", up, { once: true });
          }}
        >
          {label}
          <em className="mine">我的</em>
        </button>
      )}
      {menu && (
        <button
          type="button"
          className="mine-del"
          onMouseDown={(e) => {
            e.preventDefault();
            setMenu(false);
            onDelete();
          }}
        >
          删除
        </button>
      )}
    </span>
  );
}

export function SceneChoices({
  scene,
  imageId,
  onScene,
  onImage,
  onEnterScene,
  onEnterImage,
  onLeave,
}: {
  scene: SceneId;
  imageId: number | null;
  onScene: (id: SceneId, name: string) => void;
  onImage: (id: number, name: string) => void;
  onEnterScene?: (id: SceneId) => void;
  onEnterImage?: (src: string) => void;
  onLeave?: () => void;
}) {
  const { images } = useLibrary();
  return (
    <>
      {SCENES.map((item) => (
        <button
          type="button"
          key={item.id}
          className={imageId == null && scene === item.id ? "on" : ""}
          onMouseEnter={() => onEnterScene?.(item.id)}
          onMouseLeave={onLeave}
          onClick={() => onScene(item.id, item.name)}
        >
          {item.name}
        </button>
      ))}
      {images.map((picture) => (
        <MineMark
          key={picture.id}
          label={picture.name}
          on={imageId === picture.id}
          onPick={() => onImage(picture.id, picture.name)}
          onRename={(name) => void attempt(api.media.renameImage(picture.id, name))}
          onDelete={() => void attempt(api.media.deleteImage(picture.id))}
          onEnter={() => onEnterImage?.(imageSrc(picture))}
          onLeave={onLeave}
        />
      ))}
      <button type="button" onClick={() => void attempt(importKind("image"))}>
        ＋ 导入
      </button>
    </>
  );
}

export function SoundChoices({
  ambient,
  onPick,
}: {
  ambient: string;
  onPick: (id: string, name: string) => void;
}) {
  const { sounds } = useLibrary();
  return (
    <>
      {SOUNDS.map((item) => (
        <button type="button" key={item.id || "off"} className={ambient === item.id ? "on" : ""} onClick={() => onPick(item.id, item.name)}>
          {item.name}
        </button>
      ))}
      {sounds.map((sound) => (
        <MineMark
          key={sound.id}
          label={sound.name}
          on={ambient === `s:${sound.id}`}
          onPick={() => onPick(`s:${sound.id}`, sound.name)}
          onRename={(name) => void attempt(api.media.renameSound(sound.id, name))}
          onDelete={() => void attempt(api.media.deleteSound(sound.id))}
        />
      ))}
      <button type="button" onClick={() => void attempt(importKind("sound"))}>
        ＋ 导入
      </button>
    </>
  );
}
