import { SCENES, SOUNDS } from "../../features/atmosphere/catalog";
import { SceneBackdrop } from "../../features/atmosphere/SceneBackdrop";
import { api } from "../../shared/api";
import { useApp } from "../../shared/state";
import type { Settings, ThemeName } from "../../shared/types";
import { Slider } from "../../shared/ui/Switch";
import { attempt } from "../../shared/ui/toast";

const THEMES: { value: ThemeName; label: string }[] = [
  { value: "night", label: "夜读" },
  { value: "bamboo", label: "竹影" },
  { value: "mist", label: "晨雾" },
  { value: "dusk", label: "暮色" },
  { value: "sky", label: "跟随天色" },
];

/** 开始页右侧：画面、声音、风格，点一下就换。 */
export function AtmospherePanel() {
  const settings = useApp((s) => s.settings);
  if (!settings) return null;
  const update = (patch: Partial<Settings>) => void attempt(api.settings.update(patch));
  const scene = SCENES.some((s) => s.id === settings.scene) ? settings.scene : "sky";

  return (
    <aside className="atmo">
      <div className="label">氛围</div>
      <div className="atmo-preview">
        <SceneBackdrop scene={scene} />
        <span className="num">32:48</span>
      </div>

      <div className="label">画面</div>
      <div className="atmo-picks">
        {SCENES.map((s) => (
          <button
            type="button"
            key={s.id}
            className={scene === s.id ? "on" : ""}
            onClick={() => update({ scene: s.id })}
          >
            {s.name}
          </button>
        ))}
      </div>

      <div className="label">声音</div>
      <div className="atmo-picks">
        {SOUNDS.map((s) => (
          <button
            type="button"
            key={s.id || "off"}
            className={settings.ambient === s.id ? "on" : ""}
            onClick={() => update({ ambient: s.id })}
          >
            {s.name}
          </button>
        ))}
      </div>
      <Slider
        label="背景音音量"
        value={settings.ambient_volume}
        onCommit={(ambient_volume) => update({ ambient_volume })}
      />

      <div className="label">风格</div>
      <div className="atmo-picks">
        {THEMES.map((t) => (
          <button
            type="button"
            key={t.value}
            className={settings.theme === t.value ? "on" : ""}
            onClick={() => update({ theme: t.value })}
          >
            {t.label}
          </button>
        ))}
      </div>
    </aside>
  );
}
