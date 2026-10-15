import { SCENES } from "../../features/atmosphere/catalog";
import { SceneBackdrop } from "../../features/atmosphere/SceneBackdrop";
import { SceneChoices, SoundChoices, imageSrc, useLibrary } from "../../features/atmosphere/library";
import { api } from "../../shared/api";
import { useApp } from "../../shared/state";
import type { Settings } from "../../shared/types";
import { Slider } from "../../shared/ui/Switch";
import { attempt } from "../../shared/ui/toast";

/** 开始页右侧：画面和声音，点一下就换。 */
export function AtmospherePanel() {
  const settings = useApp((s) => s.settings);
  const { images } = useLibrary();
  if (!settings) return null;
  const update = (patch: Partial<Settings>) => void attempt(api.settings.update(patch));
  const scene = SCENES.some((s) => s.id === settings.scene) ? settings.scene : "sky";
  const photo = images.find((picture) => picture.id === settings.scene_image);

  return (
    <aside className="atmo">
      <div className="label">氛围</div>
      <div className="atmo-preview">
        <SceneBackdrop scene={scene} photo={photo ? imageSrc(photo) : null} />
        <span className="num">32:48</span>
      </div>

      <div className="label">画面</div>
      <div className="atmo-picks">
        <SceneChoices
          scene={scene}
          imageId={settings.scene_image}
          onScene={(id) => update({ scene: id, scene_image: null })}
          onImage={(id) => update({ scene_image: id })}
        />
      </div>

      <div className="label">声音</div>
      <div className="atmo-picks">
        <SoundChoices ambient={settings.ambient} onPick={(ambient) => update({ ambient })} />
      </div>
      <Slider
        label="背景音音量"
        value={settings.ambient_volume}
        onCommit={(ambient_volume) => update({ ambient_volume })}
      />
    </aside>
  );
}
