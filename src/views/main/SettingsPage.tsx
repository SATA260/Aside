import { useEffect, useState } from "react";
import { importCharacter } from "../../features/character/import";
import { isMac } from "../../features/shortcuts/keys";
import { ShortcutField } from "../../features/shortcuts/ShortcutField";
import { api } from "../../shared/api";
import { configureSfx, play } from "../../shared/sfx";
import { useApp } from "../../shared/state";
import type { CharacterInfo, Settings } from "../../shared/types";
import { Segmented } from "../../shared/ui/controls";
import { Slider, Switch } from "../../shared/ui/Switch";
import { attempt } from "../../shared/ui/toast";

export function SettingsPage() {
  const settings = useApp((s) => s.settings);
  const [autostart, setAutostart] = useState<boolean | null>(null);
  const [characters, setCharacters] = useState<CharacterInfo[]>([]);
  const dataVersion = useApp((s) => s.dataVersion);
  useEffect(() => {
    void attempt(api.autostart.get()).then((on) => on !== undefined && setAutostart(on));
  }, []);
  useEffect(() => {
    void api.character.list().then(setCharacters).catch(() => setCharacters([]));
  }, [dataVersion]);
  if (!settings) return <div className="page" />;
  const update = (patch: Partial<Settings>) => attempt(api.settings.update(patch));

  return (
    <div className="page settings-page">
      <h1 className="page-title">设置</h1>
      <div className="settings-cols">
        <section className="settings-group">
          <h3>外观</h3>
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
          <div className="settings-row">
            <div>
              <div>导入时对齐响度</div>
              <div className="hint">新导入的声音会调到和内置环境音差不多</div>
            </div>
            <Switch
              label="导入时对齐响度"
              on={settings.loudness_match !== false}
              onChange={(loudness_match) => void update({ loudness_match })}
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

        <section className="settings-group">
          <h3>考试</h3>
          <div className="settings-row">
            <div>
              <div>考研倒计时</div>
              <div className="hint">只出现在开始页顶部，全屏页里看不到</div>
            </div>
            <Switch label="考研倒计时" on={!!settings.exam_on} onChange={(exam_on) => void update({ exam_on })} />
          </div>
          <div className="settings-row">
            <div>考试日期</div>
            <input
              className="input small"
              type="date"
              aria-label="考试日期"
              value={settings.exam_date ?? ""}
              onChange={(e) => void update({ exam_date: e.target.value })}
            />
          </div>
        </section>

        <section className="settings-group">
          <h3>勿扰</h3>
          {isMac ? (
            <>
              <p className="hint">
                系统不让应用直接打开专注模式。在“快捷指令”里建两个指令，一个打开、一个关闭，把名字填在下面。开始专注时运行打开的那个，结束时运行关闭的那个。
              </p>
              <div className="settings-row">
                <div>打开时运行</div>
                <input
                  className="input small"
                  aria-label="打开专注模式的指令"
                  value={settings.dnd_on ?? ""}
                  placeholder="打开专注模式"
                  onChange={(e) => void update({ dnd_on: e.target.value })}
                />
              </div>
              <div className="settings-row">
                <div>结束时运行</div>
                <input
                  className="input small"
                  aria-label="关闭专注模式的指令"
                  value={settings.dnd_off ?? ""}
                  placeholder="关闭专注模式"
                  onChange={(e) => void update({ dnd_off: e.target.value })}
                />
              </div>
            </>
          ) : (
            <p className="hint">开始专注时提示一次“可以按 Win+N 打开勿扰”，可以点“不再提示”。</p>
          )}
        </section>

        <section className="settings-group">
          <h3>浮窗</h3>
          <div className="settings-row">
            <div>
              <div>样式</div>
              <div className="hint">胶囊不显示形象。空白处的点击会穿到下面的窗口</div>
            </div>
            <div className="atmo-picks">
              {characters.map((item) => (
                <button
                  type="button"
                  key={item.id}
                  className={(settings.character || "gaga") === item.id ? "on" : ""}
                  onClick={() => void update({ character: item.id })}
                >
                  {item.name}
                </button>
              ))}
              <button
                type="button"
                className={settings.character === "capsule" ? "on" : ""}
                onClick={() => void update({ character: "capsule" })}
              >
                胶囊
              </button>
              <button type="button" onClick={() => void attempt(importCharacter())}>
                导入形象
              </button>
            </div>
          </div>
          <div className="settings-row">
            <div>
              <div>时间到叫一声</div>
              <div className="hint">默认关闭。打开后，到点时形象会先叫一声，再响铃声</div>
            </div>
            <Switch label="时间到叫一声" on={!!settings.quack} onChange={(quack) => void update({ quack })} />
          </div>
        </section>

        <section className="settings-group">
          <h3>启动</h3>
          <div className="settings-row">
            <div>
              <div>开机自启</div>
              <div className="hint">开机后只待在{isMac ? "菜单栏" : "托盘"}里，不弹出窗口</div>
            </div>
            {autostart !== null && (
              <Switch
                label="开机自启"
                on={autostart}
                onChange={(on) => void attempt(api.autostart.set(on)).then((now) => now !== undefined && setAutostart(now))}
              />
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
