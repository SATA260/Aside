# 完整计一段时间（M1）

只能由 agent 在浏览器或真机里点的检查。每次改动开始页、全屏页、结束页后照着走一遍。

## 准备

```bash
rm -rf .devdata
pnpm dev:bridge          # 后端，127.0.0.1:1430
pnpm dev                 # 前端，localhost:1420，/__dev 代理到后端
```

浏览器打开 `http://localhost:1420/`。调后端时钟：`fetch('/__dev/dev/advance', {method:'POST', body: JSON.stringify({ms})})`。

## 步骤与预期

1. 开始页
   - 标题是“今天也坐下来了”，科目默认选中“其他”，方式“倒计时”，时长 45 选中。
   - 点 25，“自定义”里的数字跟着变成 0 小时 25 分。
   - 选“数学”，任务填“二重积分 · 第三章习题”，点“开始专注”。
2. 全屏页（地址变成 `?view=overlay`）
   - 左上“● 数学 二重积分 · 第三章习题”，右上是当前时刻，中间大号“24:5x”，下面“刚开始”。
   - 天际线左端是开始时刻，右端是开始时刻 + 25 分钟。
   - 3 秒不动鼠标，“暂停 / 结束”淡出，鼠标指针也隐藏；动一下鼠标又出现。
   - 按空格暂停：大号时间变淡，下面“暂停中 · 刚开始”，“继续”下面有小光点。
   - 暂停时把时钟往前拨 25 分钟：不会结束，天际线右端时刻往后推了 25 分钟。
   - 再按空格继续，把时钟往前拨 25 分钟：几秒内变成结束页，听到一声铃音。
3. 结束页
   - “这一段学了 25 分钟，已经记下了。”，下面“● 数学 · 二重积分 · 第三章习题”。
   - 填备注“第 3 题没想通，明天问”回车，按钮变成“已记下”。
   - 点“回到电脑”回到开始页：标题“今天学了 25 分钟”，科目回到“其他”。
4. 太短的一段
   - 开始后马上点“结束”，按钮先变成“再点一次结束”，再点一次进入结束页。
   - 显示“这一段太短，没有记下。”，没有备注框。
   - 点“回到电脑”回到开始页，科目回到“其他”，今天的总时长不变。
5. 数据
   - `sqlite3 .devdata/data.db "select subject_id, focused_ms, task, note from sessions"` 只有一条：数学、1500000、任务和备注都在。
   - 这一条有两段 segments（中间暂停过）。

## 真机（`pnpm tauri dev`）

会弹出真实的全屏窗口，先跟用户说一声再跑。

```bash
ASIDE_DEV_BRIDGE=1 pnpm tauri dev --features devserver      # 真机后端同时听 127.0.0.1:1431
curl -s -X POST localhost:1431/call -d '{"cmd":"timer_start","args":{"subject_id":5,"mode":"countdown","planned_ms":60000,"task":"","display":"fullscreen"}}'
swift scripts/windows.swift                                  # 看窗口的位置、大小和层级
```

- 开始后 `windows.swift` 只列出一个窗口，位置 0,0，大小等于主屏的逻辑分辨率（盖住菜单栏和程序坞），主窗口不在列表里。
- 点“开始专注”后主窗口消失，全屏页盖住整个主屏，包括菜单栏和程序坞，没有新开一个桌面空间。
- 结束页点“回到电脑”，全屏页关闭，主窗口回到开始页。
- 计时中强制退出 app 再打开，开始页提示“上次有一段专注已记下：……”，点“知道了”提示消失。
- 专注中 `pmset -g assertions | grep aside` 能看到 PreventUserIdleSystemSleep 和 PreventUserIdleDisplaySleep，结束后消失。
- 有第二块屏幕时：主屏是计时内容，副屏是同样的底色，没有文字。

## 浮窗（M2）

浏览器里：

1. 开始页“显示”选“浮窗”开始，地址变成 `?view=mini`，页面中间是胶囊：“● 数学 24:5x”，底边一条细进度线。
2. 悬停胶囊：科目名换成三个按钮（暂停、展开到全屏、结束），时间变小。点“展开到全屏”变成全屏页，全屏页底部有“缩成浮窗”。
3. 在浮窗里点“结束”：时间位置先变成“再点一次结束”，3 秒内再点进入全屏的结束页。
4. 同时开着两个标签页时，两个都会跟着后端切换界面，点一下只应该切换一次（看 `/events` 里 dev:open 的次数）。

真机里（只出现右上角的小胶囊，不会弹全屏；但结束或展开会打开全屏页）：

```bash
curl -s -X POST localhost:1431/call -d '{"cmd":"timer_start","args":{"subject_id":3,"mode":"countdown","planned_ms":2700000,"task":"","display":"mini"}}'
swift scripts/windows.swift
```

- 第一次出现在主屏右上角：x = 屏宽 − 280 − 24，y = 56，大小 280×88，layer 比普通窗口高。
- 用户正在全屏的 app 里（比如全屏的 Safari）时，胶囊也浮在上面；`swift scripts/windows.swift` 能列出它。
- 胶囊四周透明，看不到方形的窗口底色。
- 没点过 Aside、别的 app 在前台时，鼠标移到胶囊上按钮照样出现，第一次点按钮就生效。
- 拖动胶囊换个位置，展开到全屏再缩回浮窗：出现在刚才拖到的位置；拔掉那块屏幕后回到主屏右上角。

## 托盘和菜单栏（M2）

macOS 26 的菜单栏图标归“控制中心”所有，看不到是谁的，只能比较宽度：

```bash
cat > /tmp/status.swift <<'EOF'
import CoreGraphics
let list = CGWindowListCopyWindowInfo([.optionAll], kCGNullWindowID) as? [[String: Any]] ?? []
for w in list where (w[kCGWindowLayer as String] as? Int ?? 0) == 25 {
  let b = w[kCGWindowBounds as String] as? [String: Double] ?? [:]
  print(Int(b["X"] ?? 0), Int(b["Width"] ?? 0))
}
EOF
swift /tmp/status.swift | sort -n
```

- 没在专注时，Aside 的图标宽约 34（只有一个圈）；专注中变宽到约 77，多出来的是“44:12”这样的剩余时间。退出后这一项消失。
- 再运行一次 `src-tauri/target/debug/aside`：第二个进程立刻退出，`pgrep -f target/debug/aside` 仍只有一个，主窗口出现在前面。
- 点菜单栏图标：菜单是“开始专注 / 显示主窗口 / 退出 Aside”；专注中第一项变成“结束这一段”。
- 用“开始专注”开始时，沿用上一段的科目、方式、时长和显示方式，任务为空。
- 点主窗口的关闭按钮：窗口消失，app 没退出，菜单栏图标还在；点程序坞图标主窗口回来。
- 专注中点“退出 Aside”：下次打开没有“上次有一段专注已记下”的提示，记录里已经有这一段。
- Windows：托盘图标在专注时变成一圈进度，暂停时变灰，悬停提示“数学 · 还剩 32:48”；左键点图标打开主窗口，右键出菜单。

## 全局快捷键（M2）

浏览器里（`pnpm dev:bridge` + `pnpm dev`）：

- 侧栏底部写着“随时开始 ⌥⌘S / 全屏 / 浮窗 ⌥⌘M”，开始按钮旁写着“⌥⌘S 也可以随时开始”；Windows 上是 `Ctrl+Alt+S`。
- 设置 → 快捷键，点“开始或结束”的框：变成“按下新的组合键”，下面小字“Esc 取消”。按住 ⌥⌘ 时框里显示“⌥⌘…”。
- 只按 S 或 Shift+S：小字变成“要同时按住 ⌘、⌥ 或 ⌃”，旧的不变。
- 按 ⇧⌘K：框和侧栏都变成 ⇧⌘K。再录 ⌥⌘M：提示“和“全屏和浮窗切换”的快捷键重复了”，仍是 ⇧⌘K。Esc 或点别处：退出录制，不变。
- 录制时 bridge 先收到 `dev:shortcuts-paused {paused:true}`，录完或取消后收到 `false`。

真机（`ASIDE_DEV_BRIDGE=1 pnpm tauri dev --features devserver …`）：

- `curl -X POST localhost:1431/call -d '{"cmd":"shortcut_set","args":{"action":"toggle","accel":"Alt+Super+K"}}'` 返回新设置；终端里没有“没有注册上”。最后记得改回 `CommandOrControl+Alt+S`。
- 下面几条要真的按键，需要人来做（agent 没有辅助功能权限，模拟不了全局按键）：
  - 在别的软件里按 ⌥⌘S：按上一次的选择开始；再按一次结束，出现结束页。
  - 专注中按 ⌥⌘M：全屏和浮窗来回切；没在专注时按，什么也不发生。
  - 在设置里录新快捷键时按 ⌥⌘S：不会开始专注（录制期间全局快捷键暂停了）。
- macOS 注册快捷键时不独占，别的软件已经用了的组合一般也能注册上，两边都会响应，所以“被别的软件占用了”主要出现在 Windows：在 Windows 上把快捷键设成别的软件已占用的组合（比如某输入法的 `Ctrl+Alt+S`），应提示“这个组合被别的软件占用了，还在用原来的”。

## 开机自启（M2）

- 浏览器里：设置 → 启动 → 开机自启，默认关着；提示写“开机后只待在菜单栏里”（Windows 是“托盘”）。打开后刷新页面仍是开着。
- 真机打开：`curl -X POST localhost:1431/call -d '{"cmd":"autostart_set","args":{"on":true}}'` 返回 `true`；`plutil -p ~/Library/LaunchAgents/Aside.plist` 里 `ProgramArguments` 是 aside 的路径加 `--autostart`，`RunAtLoad` 为真。
- 模拟开机：退出 Aside，用后台进程运行 `ASIDE_DEV_BRIDGE=1 src-tauri/target/debug/aside --autostart`（不要放进一条会结束的 shell 命令里跑，命令结束时进程会被一起杀掉）。
  - `swift /tmp/allwin.swift` 里主窗口 `onscreen nil`，没有弹出来；`swift /tmp/status.swift` 多了一个宽约 34 的菜单栏项。
  - `lsappinfo list | grep "pid = <aside 的 pid>"` 显示 `type="UIElement"`：程序坞里没有图标。
- 再运行一次 `src-tauri/target/debug/aside`：它立刻退出，原来那个的主窗口出现，`type` 变成 `"Foreground"`，程序坞出现图标。
- 关掉：`autostart_set {"on":false}` 返回 `false`，`~/Library/LaunchAgents/Aside.plist` 被删掉。检查完一定要关，不然用户开机时会启动 debug 版。
- Windows：打开后注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` 有一项 `Aside`，值是 exe 路径加 `--autostart`；重启后只有托盘图标，没有窗口。
