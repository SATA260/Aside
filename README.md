# Aside

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/SATA260/Aside)](https://github.com/SATA260/Aside/releases)

陪考研学生自习的桌面计时器。开始之后，电脑退到一旁，只安静地显示时间，同时如实记下每一段学习。

A quiet desktop focus timer for postgraduate entrance exam students. Local-only. Windows and macOS.

- **只记录，不评判。** 数据只展示做到了什么。没有失败、没有断签、没有红色。
- **电脑是背景，不是主角。** 计时页在房间另一头也能看清，但不会催促人。
- **数据只留在这台电脑上。** 没有账号，没有云同步。

## 能做什么

- **倒计时 / 正计时 / 模考**：预设 25、45、60、90 分钟，也可以自定义；正计时适合做整套卷子；模考按考研时长走，结束后可以填分数。
- **全屏或浮窗**：默认铺满所有屏幕，只在主屏显示时间。也可以缩成角落里的小人，按住形象、气泡或空白处就能拖走。
- **氛围**：五种画面（天色、雨窗、台灯、星夜、晨雾）和四段背景音（雨声、咖啡馆、篝火、白噪音），也可以导入自己的图片和音频。专注中随时能换。
- **记录**：某一天的时间轴、趋势、热力图、补记和备注。周报可以存成一张图。
- **系统边角**：菜单栏显示剩余时间，全局快捷键开始或切换显示方式，计时期间阻止休眠。开机自启默认关掉。

提前结束不算失败：实际学了多久就记多久，界面上只说「已记下」。少于 1 分钟的不记录。

更细的产品说明见 [`docs/product-plan.md`](docs/product-plan.md)，代码结构和流程见 [`docs/implementation.md`](docs/implementation.md)。

## 安装

目前提供 **macOS Apple Silicon** 安装包，从 [Releases](https://github.com/SATA260/Aside/releases) 下载 `Aside_0.1.0_aarch64.dmg`。

第一次打开如果被系统拦住，按住 Control 点图标，选「打开」。安装包是本地打包、未经过 Apple 公证的。

需要 Intel Mac 或 Windows 时，请在本机从源码编译。

## 从源码运行

需要 [Node.js](https://nodejs.org) 20+、[pnpm](https://pnpm.io) 和通过 [rustup](https://rustup.rs) 安装的 Rust。

```bash
pnpm install
pnpm tauri dev
```

只构建前端：

```bash
pnpm build
```

打安装包（在当前系统上）：

```bash
pnpm tauri build --bundles dmg    # macOS
pnpm tauri build --bundles nsis   # Windows
```

## 测试

```bash
pnpm test
pnpm test:e2e
cargo test --manifest-path src-tauri/Cargo.toml --lib
```

## 数据放在哪

运行时产生的数据不在仓库里，而在系统给应用的数据目录：

| 系统 | 路径 |
|---|---|
| macOS | `~/Library/Application Support/com.aside.app/` |
| Windows | `%APPDATA%\com.aside.app\` |

里面是本地 SQLite、导入的声音和图片、形象包。设置里可以打开这个文件夹，也可以导出备份。

## 许可

Aside 以 [MIT License](LICENSE) 发布。

内置形象「嘎嘎」是项目原创的企鹅连体服小人，随源码按 MIT 授权。内置背景音和字体是第三方作品，授权不变：

- 雨声、篝火、咖啡馆录音为 [CC0](src/assets/sounds/CREDITS.md)
- 白噪音由程序生成
- [Manrope](https://github.com/sharanda/manrope) 与 [霞鹜文楷 屏幕版](https://github.com/lxgw/LxgwWenKai-Screen) 为 SIL Open Font License 1.1
