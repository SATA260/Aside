import { describe, expect, it } from "vitest";
import { SCENES, SOUNDS, backdrops, nextOf, soundOptions } from "../../src/features/atmosphere/catalog";

describe("atmosphere catalog", () => {
  it("starts from the sky scene and an off sound", () => {
    expect(SCENES[0]).toEqual({ id: "sky", name: "天色" });
    expect(SOUNDS[0]).toEqual({ id: "", name: "关" });
    expect(SCENES.map((s) => s.name)).toEqual(["天色", "雨窗", "台灯", "星夜", "晨雾"]);
  });

  it("cycles to the next item and wraps around", () => {
    expect(nextOf(SCENES, "sky").id).toBe("rain");
    expect(nextOf(SCENES, "fog").id).toBe("sky");
    expect(nextOf(SOUNDS, "").id).toBe("rain");
    expect(nextOf(SOUNDS, "noise").id).toBe("");
  });

  it("stays on the first item when the current id is unknown", () => {
    expect(nextOf(SCENES, "missing").id).toBe("sky");
  });

  it("puts imported pictures and sounds after the built-in ones", () => {
    const scenes = backdrops([{ id: 3, name: "教室" }]);
    expect(scenes.at(-1)).toEqual({ id: "img:3", name: "教室", image: 3 });
    expect(nextOf(scenes, "fog").id).toBe("img:3");
    expect(nextOf(scenes, "img:3").id).toBe("sky");
    const sounds = soundOptions([{ id: 8, name: "老电扇" }]);
    expect(nextOf(sounds, "noise").name).toBe("老电扇");
    expect(nextOf(sounds, "s:8").id).toBe("");
  });
});
