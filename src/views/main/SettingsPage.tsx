import type { CSSProperties } from "react";
import { ShortcutField } from "../../features/shortcuts/ShortcutField";
import { api } from "../../shared/api";
import { configureSfx, play } from "../../shared/sfx";
import { useApp } from "../../shared/state";
import type { Settings, ThemeName } from "../../shared/types";
import { Segmented } from "../../shared/ui/controls";
import { Slider, Switch } from "../../shared/ui/Switch";
import { attempt } from "../../shared/ui/toast";

const THEMES: { value: ThemeName; label: string; swatch: string }[] = [
  { value: "night", label: "夜读", swatch: "#10151f" },
  { value: "bamboo", label: "竹影", swatch: "#121b17" },
  { value: "mist", label: "晨雾", swatch: "#e4e9ef" },
  { value: "dusk", label: "暮色", swatch: "#e9e4ec" },
  { value: "sky", label: "跟随天色", swatch: "linear-gradient(135deg, #e4e9ef 0 50%, #10151f 50%)" },
];

export function SettingsPage() {
  const settings = useApp((s) => s.settings);
  if (!settings) return <div className="page" />;
  const update = (patch: Partial<Settings>) => attempt(api.settings.update(patch));

  return (
    <div className="page">
      <h1 className="page-title">设置</h1>

      <section className="settings-group">
        <h3>外观</h3>
        <div className="settings-row">
          <div>
            <div>风格</div>
            <div className="hint">跟随天色：清晨晨雾，白天晴灰，傍晚暮色，夜里夜读</div>
          </div>
          <div className="themes">
            {THEMES.map((t) => (
              <button
                type="button"
                key={t.value}
                className={settings.theme === t.value ? "theme-pick on" : "theme-pick"}
                onClick={() => {
                  play("tap");
                  void update({ theme: t.value });
                }}
              >
                <i style={{ background: t.swatch } as CSSProperties} />
                {t.label}
              </button>
            ))}
          </div>
        </div>
        <div className="settings-row">
          <div>
            <div>倒计时显示</div>
            <div className="hint">专注中也可以点一下大数字切换</div>
          </div>
          <Segmented
            value={settings.time_display}
            options={[
              { value: "remaining", label: "还剩多少" },
              { value: "elapsed", label: "已经多少" },
            ]}
            onChange={(v) => void update({ time_display: v })}
          />
        </div>
      </section>

      <section className="settings-group">
        <h3>声音</h3>
        <div className="settings-row">
          <div>
            <div>界面音效</div>
            <div className="hint">点按、开始、暂停、结束时的轻响</div>
          </div>
          <Switch label="界面音效" on={settings.sfx_enabled} onChange={(on) => void update({ sfx_enabled: on })} />
        </div>
        <div className="settings-row">
          <div>音效音量</div>
          <Slider
            label="音效音量"
            value={settings.sfx_volume}
            onCommit={(v) => {
              configureSfx({ enabled: settings.sfx_enabled, volume: v });
              play("tap");
              void update({ sfx_volume: v });
            }}
          />
        </div>
      </section>

      <section className="settings-group">
        <h3>快捷键</h3>
        <div className="settings-row">
          <div>
            <div>开始或结束</div>
            <div className="hint">在别的软件里也能按，沿用上一次的选择</div>
          </div>
          <ShortcutField action="toggle" label="开始或结束" accel={settings.shortcut_toggle} />
        </div>
        <div className="settings-row">
          <div>
            <div>全屏和浮窗切换</div>
            <div className="hint">专注中才有用</div>
          </div>
          <ShortcutField action="display" label="全屏和浮窗切换" accel={settings.shortcut_display} />
        </div>
      </section>
    </div>
  );
}
