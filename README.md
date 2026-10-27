# Countdown 桌面翻牌倒计时

> Windows 11 桌面小工具：一个 520×340 的置顶悬浮窗，用三张翻牌卡片倒计时，最小化就藏进托盘，时间到了自己弹回来。

## 📋 目录

- [项目简介](#项目简介)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [环境要求](#环境要求)
- [快速开始](#快速开始)
- [配置说明](#配置说明)
- [部署指南](#部署指南)
- [内部命令与事件](#内部命令与事件)
- [常见问题](#常见问题)
- [许可证](#许可证)

## 项目简介

写这个工具是为了解决「开个会/煮个面/番茄钟，不想开浏览器标签也不想装一堆全家桶」的场景：双击就能用，不占任务栏，到点自己跳出来，没有声音也没有系统通知打扰。

界面对照的是翻牌时钟：深灰圆角卡片里放白色窄体数字，每秒沿中缝翻一次。

![倒计时页](docs/screenshots/countdown.png)

### 核心功能

- [x] 时 / 分 / 秒 三项输入，小时上限 99
- [x] 30 / 45 / 60 分钟快捷按钮，点一下直接填好
- [x] 三张卡片翻牌动画，只翻发生变化的那一张
- [x] 中途暂停、继续、取消；到点后按钮变「重置」
- [x] 点 `—` 或 `✕` 隐藏到系统托盘，点托盘图标唤回并继续计时
- [x] 窗口隐藏状态下到点，自动 `show` 回来并置顶，`00:00:00` 呼吸闪动
- [x] 单实例锁：重复双击 exe 只唤回已有窗口，不会开出两个计时器
- [ ] 开机自启、代码签名、自动更新、时长记忆（都没做，见[常见问题](#常见问题)）

## 技术栈

| 分类 | 技术 | 版本 |
| ---- | ---- | ---- |
| 外壳 | Tauri | 2.12.5 |
| 构建 CLI | @tauri-apps/cli | 2.12.4 |
| 单实例 | tauri-plugin-single-instance | 2.5.4 |
| 语言 | Rust（msvc 工具链） | 1.99.0 |
| 前端 | 原生 HTML / CSS / JS（无框架、无打包器） | - |
| 数字字体 | Oswald SemiBold（OFL 1.1） | 600 字重 |
| 运行时 | WebView2 | 133.0.3065.51 |

前端没有引入任何 npm 运行时依赖，只通过 `withGlobalTauri` 注入的 `window.__TAURI__` 调命令和监听事件，所以没有 `vite`、没有 `dist`、没有 HMR，改完 UI 重启 `npm run dev` 即可。

Oswald 字体文件已随项目分发（`ui/fonts/oswald-600.woff2`）。OFL 1.1 要求再分发时保留许可证，`ui/fonts/OFL-oswald.txt` 不要删。

## 项目结构

```text
countdown/
├── ui/                     # 前端，直接作为 frontendDist 打包
│   ├── index.html          # 两个视图：设置页 / 倒计时页
│   ├── style.css           # 全部视觉与翻牌动画（含数字标定值）
│   ├── flip.js             # 翻牌组件：3 张卡片，值变化才触发翻转
│   ├── main.js             # 视图切换、输入校验、命令与事件绑定
│   └── fonts/              # Oswald woff2 与 OFL 许可证
├── src-tauri/
│   ├── src/timer.rs        # 计时状态机（Idle/Running/Paused/Finished）+ 单元测试
│   ├── src/main.rs         # 窗口、托盘、6 个命令、100ms 轮询线程
│   ├── Cargo.toml          # tauri / serde / single-instance 依赖
│   ├── tauri.conf.json     # 窗口参数、打包配置
│   ├── capabilities/       # Tauri 2 权限声明
│   └── icons/              # 由 assets/icon.png 生成的全套图标
├── assets/icon.png         # 图标源图（1024×1024）
├── README.md               # 项目说明（本文件）
├── LICENSE                 # MIT 项目许可证
├── build.bat               # 打包脚本（Windows 双击可用）
├── docs/
│   ├── screenshots/        # 运行时截图
│   └── superpowers/        # 设计规格与实施计划
└── package.json            # 只有 @tauri-apps/cli 一个 devDependency
```

## 环境要求

Windows 10/11 64 位（在 Windows 11 25H2 上验证）。除此之外需要：

| 依赖 | 要求 | 说明 |
| ---- | ---- | ---- |
| Node.js | 18+ | 本机 22.22.2，只用来跑 `@tauri-apps/cli` |
| Rust | stable，msvc 工具链 | 本机 1.99.0 |
| MSVC + Windows SDK | 用于链接 | 本机 Visual Studio 2026 (18) Community，MSVC 14.50.35717，SDK 10.0.26100.0 |
| WebView2 运行时 | 已安装 | Win11 自带，本机 133.0.3065.51 |
| 网络 | 首次构建需要 | 要拉 crates.io 依赖；打安装包还要从 GitHub 下载 NSIS |

`cargo build` 靠 `vswhere.exe` 自动找 MSVC。如果链接阶段报找不到 `link.exe`，检查 `C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe` 是否存在——没有它就说明 C++ 桌面开发工作负载没装。

## 快速开始

### 开发运行

```bash
npm install
npm run dev
```

预期：弹出一个 520×340 的无边框窗口，屏幕居中、置顶、任务栏里看不到它。首次会编译 Rust 依赖，视机器 1-3 分钟；之后改 Rust 代码约 5-10 秒，改 `ui/` 下的文件需要重启 `npm run dev`（资源是编译进二进制的）。

### 跑单元测试

```bash
cd src-tauri
cargo test
```

预期输出：

```text
running 4 tests
test timer::tests::cancel_returns_to_idle ... ok
test timer::tests::expired_clamps_to_zero ... ok
test timer::tests::pause_freezes_and_resume_shifts_deadline ... ok
test timer::tests::running_counts_down ... ok
test result: ok. 4 passed; 0 failed
```

### 打包

```bash
build.bat          # 出 NSIS 安装包；下载失败会自动退回只出 exe
build.bat exe      # 只出便携版 exe
```

在终端里也可以直接跑 `npm run build` / `npm run build:exe`（`build.bat` 结尾有 `pause`，双击时用来留住窗口）。

产物：

```text
src-tauri/target/release/countdown.exe
src-tauri/target/release/bundle/nsis/Countdown_0.1.0_x64-setup.exe
```

便携版约 9.5 MB，双击就跑，不需要安装。

## 配置说明

| 配置项 | 位置 | 说明 | 默认值 |
| ---- | ---- | ---- | ---- |
| `app.windows[0].width` / `height` | `src-tauri/tauri.conf.json` | 窗口逻辑尺寸（不是物理像素，150% 缩放下实际是 780×510） | `520` / `340` |
| `app.windows[0].alwaysOnTop` | 同上 | 置顶，翻牌窗需要一直看得见 | `true` |
| `app.windows[0].skipTaskbar` | 同上 | 不进任务栏，隐藏后只能从托盘唤回 | `true` |
| `identifier` | 同上 | 单实例锁、托盘归属都靠它，改了就变成另一个应用 | `com.countdown.desktop` |
| `.card` 的 `width` / `height` | `ui/style.css` | 卡片尺寸，比例对齐参考截图的 369×367 | `136` / `136` |
| `.digits` 的 `font-size` / `scaleX` / `translateY` | 同上 | 字号、横向收窄、垂直居中校正，三个值配套标定，改一个其它两个要重算 | `121px` / `0.79` / `-0.053em` |

`.digits` 那三个值不是随手调的：字号 121px + `scaleX(0.79)` 让字高占卡高 72%、两字总宽占卡宽 67%，`translateY(-0.053em)` 用来抵消 Oswald 升降部不对称导致的下沉（不加的话字身中心比卡片中心低约 4 CSS px）。换字体或改字号必须重新量一遍，否则会重新跑偏。

## 部署指南

### 便携版

把 `src-tauri/target/release/countdown.exe` 拷到任意目录双击即可。首次运行 Windows SmartScreen 可能拦一次（没有代码签名），点「更多信息 → 仍要运行」。

### 安装包

`build.bat` 产出的 `Countdown_0.1.0_x64-setup.exe` 是 NSIS 安装包，双击按向导装完即可，卸载在「应用和功能」里找 Countdown。

打安装包这一步要联网从 `github.com/tauri-apps/binary-releases` 下载 `nsis-3.13.zip`，还要拉 `nsis_tauri_utils.dll`。下载过一次会缓存在 `%LOCALAPPDATA%\tauri\NSIS`，之后离线也能打。内网或代理环境下常见失败就是这一步超时，此时用 `build.bat exe` 只出便携版。

## 内部命令与事件

前端通过 `window.__TAURI__.core.invoke` 调命令、`window.__TAURI__.event.listen` 听事件，没有 HTTP 接口。

| 命令 | 参数 | 返回值 | 说明 |
| ---- | ---- | ---- | ---- |
| `start_countdown` | `{ seconds: number }` | `null` | 秒数为 0 时返回错误 |
| `pause_countdown` | 无 | `null` | 非 `Running` 状态返回错误 |
| `resume_countdown` | 无 | `null` | 非 `Paused` 状态返回错误 |
| `cancel_countdown` | 无 | `null` | 回到 `Idle`；结束时也用它当「重置」 |
| `get_state` | 无 | `{ phase, remaining_ms }` | `phase` 取 `idle` / `running` / `paused` / `finished` |
| `hide_window` | 无 | `null` | 隐藏到托盘 |

| 事件 | 载荷 | 触发时机 |
| ---- | ---- | ---- |
| `countdown://tick` | `{ remaining_ms: number }` | 运行中整秒变化时；到点时会补发一次 `0` |
| `countdown://finished` | 无 | 剩余归零，窗口已 `show` 并置顶 |
| `countdown://sync` | 无 | 从托盘或第二次启动唤回窗口，前端据此重新拉 `get_state` |

计时放在 Rust 而不是前端，是因为 WebView2 对隐藏窗口会节流定时器，靠 JS `setInterval` 触发「到点弹窗」会延迟；现在由 Rust 侧 100ms 轮询 deadline，隐藏状态下也能准点把窗口叫回来。

## 常见问题

**Q: `npm run build` 报 `failed to bundle project: 'timeout: global'`？**

A: NSIS 下载超时，不是代码问题。等网络恢复重试，或者直接 `build.bat exe` 出便携版。下载成功一次后会被缓存，后面离线也能打安装包。

**Q: 往 `build.bat` 里加中文提示后，脚本报 `'.bat' is not recognized`？**

A: 批处理里同时用 `chcp 65001` 和 UTF-8 中文，会让 cmd 解析时字节偏移错乱，把下一行的 `echo` 吃掉当命令执行。这个脚本刻意全用 ASCII 输出，别往里加中文。

**Q: 构建报 `Access is denied` 或「文件被占用」？**

A: 正在运行的 `countdown.exe` 占着 `target/release` 下的同名文件，先关掉它再打包。`build.bat` 会检测并提示，但不会替你杀进程。

**Q: 双击第二次没反应，也不弹新窗口？**

A: 单实例锁生效了，它把已有窗口唤回并保持原倒计时状态。要开第二个窗口得先退出托盘里的那个（托盘右键 → 退出）。

**Q: 到点没声音、也没有 Windows 通知？**

A: 这是明确选择的：只把窗口弹回置顶 + 数字闪动。需要声音或系统通知的话得改 Rust 侧的 `poll_loop`，在 `EVENT_FINISHED` 前后加。

**Q: 数字上下不对齐 / 换字体后就歪了？**

A: `.digits` 的 `translateY` 是针对 Oswald 600 @ 121px 标定的值。换字体、改字号、改卡片高度都会让字身重新偏离中缝，需要用截图逐像素量一遍（对齐卡片中心与数字包围盒中心），不能凭眼睛调。

**Q: 有开机自启、代码签名、自动更新吗？**

A: 都没有。当前是自用工具的实现范围：不装服务、不写注册表、不联网。要给别人分发的话，代码签名和自动更新（Tauri updater）是上生产前必须补的两块，否则 SmartScreen 会拦、版本也没法平滑升级。

**Q: 之前遇到过一次进程直接退出（`exit code: 0xffffffff`），日志里什么都没有？**

A: 只出现过一次，之后完整跑了暂停/继续/取消/隐藏/到点/重置和多轮打包都没再复现，没定位到原因。如果频繁出现，建议用 `npm run dev` 跑一遍看 stderr。

## 许可证

本项目代码用 [MIT](./LICENSE)。

`ui/fonts/oswald-600.woff2` 是 Oswald 字体，单独适用 SIL Open Font License 1.1，许可证在 `ui/fonts/OFL-oswald.txt`。它不归本项目的 MIT 许可管，再分发（包括打包成 exe 发出去）时需要保留那份许可证文本。
