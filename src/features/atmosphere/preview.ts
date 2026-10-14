import { create } from "zustand";
import type { ThemeName } from "../../shared/types";
import type { SceneId } from "./catalog";

/** 全屏页悬停时的临时预览。移开鼠标就清空，点下去才写进设置。 */
export const useAtmoPreview = create<{
  scene: SceneId | null;
  picture: string | null;
  theme: ThemeName | null;
}>(() => ({
  scene: null,
  picture: null,
  theme: null,
}));
