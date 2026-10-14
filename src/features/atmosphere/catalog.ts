export const SCENES = [
  { id: "sky", name: "天色" },
  { id: "rain", name: "雨窗" },
  { id: "lamp", name: "台灯" },
  { id: "stars", name: "星夜" },
  { id: "fog", name: "晨雾" },
] as const;

export const SOUNDS = [
  { id: "", name: "关" },
  { id: "rain", name: "雨声" },
  { id: "library", name: "图书馆" },
  { id: "cafe", name: "咖啡馆" },
  { id: "fire", name: "篝火" },
  { id: "noise", name: "白噪音" },
] as const;

export type SceneId = (typeof SCENES)[number]["id"];
export type SoundId = (typeof SOUNDS)[number]["id"];

export function nextOf<T extends { id: string }>(list: readonly T[], id: string): T {
  const index = list.findIndex((item) => item.id === id);
  if (index < 0) return list[0];
  return list[(index + 1) % list.length];
}
