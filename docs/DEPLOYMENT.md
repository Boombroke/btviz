# btviz 部署指南

本文覆盖三种部署形态：

1. **终端用户安装**：直接装 `.deb` / `.rpm` / `.AppImage` 预构建包。
2. **从源码构建**：在新机器上拉仓库 → 编出可执行文件或安装包。
3. **开发模式**：热重载调试 (`npm run tauri:dev`)。

> 一键脚本 `scripts/install.sh` 涵盖 1 与 2，可作为生产部署的快速通道。

---

## 0. 系统依赖

btviz 是 Tauri 2 应用，前端走 WebView，因此目标机器需要 **WebKitGTK 4.1**、**GTK 3**、**rsvg2** 等图形栈。源码构建额外需要 **Rust 1.75+**、**Node.js 20+**、**zmq** 开发头文件。

| 发行版 | 运行依赖 | 构建依赖（运行依赖之上） |
|---|---|---|
| Ubuntu 24.04 / Debian 12+ | `libwebkit2gtk-4.1-0 libgtk-3-0 librsvg2-2 libayatana-appindicator3-1 libssl3 libzmq5` | `build-essential pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libzmq3-dev curl ca-certificates` |
| Ubuntu 22.04 | 同上但 webkit 包名为 `libwebkit2gtk-4.0-37`；Tauri 2 仍走 4.1，建议升级或自行装 4.1 后向源 | 同上 |
| Fedora 39+ / RHEL 9+ | `webkit2gtk4.1 gtk3 librsvg2 libappindicator-gtk3 zeromq` | `gcc gcc-c++ make pkgconf-pkg-config webkit2gtk4.1-devel gtk3-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel zeromq-devel` |
| Arch / Manjaro | `webkit2gtk-4.1 gtk3 librsvg libappindicator-gtk3 zeromq` | `base-devel webkit2gtk-4.1 gtk3 librsvg libappindicator-gtk3 openssl pkgconf zeromq` |

> Ubuntu 22.04 注意：upstream Tauri 2 锁定 `webkit2gtk-4.1`，22.04 默认仓库只到 4.0。可加 PPA 或直接升级到 24.04。`scripts/install.sh` 在检测到 22.04 时会拒绝继续并打印提示。

Rust / Node 由脚本通过 rustup / nodesource 自动装；手动安装时：

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

# Node.js 20 (NodeSource)
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
```

---

## 1. 终端用户：装预构建包

`npm run tauri:build` 在 `target/release/bundle/` 下产出三种格式（详见 [README](../README.md#install)）。在用户机器上：

### `.deb` (Ubuntu / Debian)

```bash
sudo apt install -y ./btviz_0.1.0_amd64.deb
btviz   # 启动
```

依赖会被 apt 自动解析。卸载：`sudo apt remove btviz`。

### `.rpm` (Fedora / RHEL)

```bash
sudo dnf install -y ./btviz-0.1.0-1.x86_64.rpm
btviz
```

### `.AppImage` (任意发行版)

`AppImage` 自带 webkit / gtk 运行时（约 75 MB），免装系统依赖：

```bash
chmod +x btviz_0.1.0_amd64.AppImage
./btviz_0.1.0_amd64.AppImage
```

放到 `~/Applications/` 并配合 [AppImageLauncher](https://github.com/TheAssassin/AppImageLauncher) 即可加入系统菜单。

---

## 2. 从源码构建

### 2.1 一键脚本（推荐）

```bash
git clone https://github.com/Boombroke/btviz.git
cd btviz
./scripts/install.sh build      # 自动装系统依赖 + Rust + Node + 编译 + 安装到 /usr/local/bin
```

`install.sh` 子命令：

| 子命令 | 行为 |
|---|---|
| `deps` | 仅安装系统依赖（apt / dnf / pacman 自动检测） |
| `build` | 装依赖 + 编出 `btviz` 二进制 + 安装到 `/usr/local/bin/btviz`（默认） |
| `bundle` | 装依赖 + `npm run tauri:build` 产出 `.deb` / `.rpm` / `.AppImage` 三件套到 `target/release/bundle/` |
| `dev` | 装依赖 + `npm install` + 启动 `npm run tauri:dev` 开发模式 |
| `uninstall` | 删除 `/usr/local/bin/btviz` 与 `~/.local/share/applications/btviz.desktop` |

环境变量：

- `INSTALL_PREFIX=/opt/btviz` 改安装路径
- `SKIP_RUST=1` 已有 Rust 工具链时跳过 rustup
- `SKIP_NODE=1` 已有 Node.js 时跳过 nodesource
- `RELEASE_PROFILE=debug` 编 debug 版（默认 release）

### 2.2 手工步骤

```bash
# 1. 装系统依赖（以 Ubuntu 24.04 为例）
sudo apt update
sudo apt install -y build-essential pkg-config curl ca-certificates \
    libwebkit2gtk-4.1-dev libgtk-3-dev libssl-dev \
    libayatana-appindicator3-dev librsvg2-dev libzmq3-dev

# 2. 装 Rust（脚本式，无 sudo）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
. "$HOME/.cargo/env"

# 3. 装 Node.js 20
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# 4. 拉源码 + 装前端依赖 + 类型检查
git clone https://github.com/Boombroke/btviz.git
cd btviz
npm install
npm run typecheck

# 5. 编译
npm run tauri:build      # 产出 deb / rpm / AppImage
# 或者只要二进制：
cargo build --release --manifest-path src-tauri/Cargo.toml
sudo install -m 0755 target/release/btviz /usr/local/bin/btviz
```

---

## 3. 开发模式

```bash
./scripts/install.sh dev
# 等价于：
npm install && npm run tauri:dev
```

开发模式下：
- 前端 Vite dev server 跑在 `http://localhost:1420`，热重载所有 TSX / CSS。
- 后端 Rust 改动后 Tauri 自动重启窗口。
- 调试日志：`RUST_LOG=debug npm run tauri:dev`。

---

## 4. 验证安装

```bash
# 二进制能跑：
btviz --help            # 当前没有 CLI 参数，会直接启动 GUI
which btviz

# 跑一个 BT.CPP 4.x 服务端验证连接
# （在 Sentry26 项目里）
ros2 launch sentry_behavior sentry_behavior_launch.py target_tree:=a
# btviz → Connect → tcp://127.0.0.1:1667
```

成功后 Canvas 会渲染整棵树，状态高亮随 BT tick 节奏更新（默认 50 Hz 轮询）。

---

## 5. 排错

### 5.1 `error while loading shared libraries: libwebkit2gtk-4.1.so`

运行依赖缺失。装系统依赖：

```bash
./scripts/install.sh deps        # 自动检测发行版
# 或手工：
sudo apt install -y libwebkit2gtk-4.1-0 librsvg2-2 libgtk-3-0 libayatana-appindicator3-1
```

Ubuntu 22.04 默认源只有 4.0；要么升级系统，要么用 AppImage（自带 4.1）。

### 5.2 `cannot find -lzmq`

构建时 zmq 头文件缺失：

```bash
sudo apt install -y libzmq3-dev          # Debian/Ubuntu
sudo dnf install -y zeromq-devel         # Fedora
sudo pacman -S zeromq                    # Arch
```

### 5.3 Connect 后报 `request_full_tree: timeout`

- 服务端没启动：确认目标进程里有 `BT::Groot2Publisher publisher(tree, 1667);`
- 端口被占：`ss -ltn | grep 1667`，或在服务端换端口后在 btviz 里输入对应地址
- 防火墙：本地连接无影响；跨主机需放通 1667 (TCP)

### 5.4 拖动 Canvas 没反应 / 节点点不中

- 检查 BtNode 是否带 `data-bt-node` 属性（pan 时按这个 attr 跳过节点）
- 浏览器缩放 ≠ Canvas 缩放：DPI 缩放应在系统设置里调

### 5.5 AppImage 在某些桌面环境闪退

```bash
APPIMAGE_EXTRACT_AND_RUN=1 ./btviz_0.1.0_amd64.AppImage
```

绕开 fuse 挂载，直接解压运行。

### 5.6 macOS / Windows

当前 release 只覆盖 Linux x86_64。两端的构建步骤等价（装 Rust + Node + 系统 webview 依赖 → `npm run tauri:build`），但尚未在 CI 中验证。欢迎 PR。

---

## 6. CI / 持续集成（建议蓝图）

`.github/workflows/build.yml` 大致：

```yaml
name: build
on: [push, pull_request]
jobs:
  linux:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with: { toolchain: stable }
      - uses: actions/setup-node@v4
        with: { node-version: '20' }
      - run: ./scripts/install.sh deps
      - run: npm install
      - run: npm run typecheck
      - run: cargo test --workspace --manifest-path Cargo.toml
      - run: npm run tauri:build
      - uses: actions/upload-artifact@v4
        with:
          name: btviz-linux
          path: |
            target/release/bundle/deb/*.deb
            target/release/bundle/rpm/*.rpm
            target/release/bundle/appimage/*.AppImage
```

### 发布流程

打 tag 触发 release：

```bash
git tag v0.1.0
git push origin v0.1.0
# CI 自动 attach bundles 到 GitHub release
```

---

## 7. 卸载

```bash
./scripts/install.sh uninstall
```

或手工：

```bash
sudo rm /usr/local/bin/btviz
rm ~/.local/share/applications/btviz.desktop 2>/dev/null
sudo apt remove btviz                  # 仅 .deb 安装的情况
```
