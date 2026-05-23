# btviz

[BehaviorTree.CPP 4.x](https://github.com/BehaviorTree/BehaviorTree.CPP) 行为树的现代化轻量级可视化工具。

是 [Groot2](https://www.behaviortree.dev/groot/) 的免费替代品，没有免费版 20 节点上限。

## 功能

- **实时只读可视化**：任意 BT.CPP 4.x 行为树都能直接连，无节点数量限制
- **在线模式**：连到运行中的 `BT::Groot2Publisher`（默认 `tcp://127.0.0.1:1667`），通过 ZMQ 拉树+轮询状态
  - 状态轮询最高 200 Hz；只把变更过的节点推到画布
  - idle / running / success / failure / skipped 五态切换带 220ms cubic-out 动画
- **离线模式**：直接打开磁盘上的 BT XML 文件做静态检查
- **Canvas 操作**：拖拽平移、滚轮缩放（锚定光标）、`0` 复位、`+` / `-` 步进缩放
- **Inspector 侧栏**：选中节点查看 ports、跳转父节点、子节点计数
- **跨平台桌面应用**：基于 Tauri 2（当前打包 Linux x86_64；macOS / Windows 后续）

## 安装

一键安装脚本（自动识别 Ubuntu / Debian / Fedora / Arch）：

```bash
git clone https://github.com/Boombroke/btviz.git && cd btviz
./scripts/install.sh           # 装系统依赖 + 编译 + 安装到 /usr/local/bin
./scripts/install.sh bundle    # 改为产出 .deb / .rpm / .AppImage 三件套
./scripts/install.sh help      # 查看所有子命令
```

`npm run tauri:build` 产出的预构建包：

| 格式 | 路径 | 大小 |
|---|---|---|
| `.deb` | `target/release/bundle/deb/btviz_0.1.0_amd64.deb` | ~2 MB |
| `.rpm` | `target/release/bundle/rpm/btviz-0.1.0-1.x86_64.rpm` | ~2 MB |
| `.AppImage` | `target/release/bundle/appimage/btviz_0.1.0_amd64.AppImage` | ~75 MB |

手工装系统依赖（Debian / Ubuntu 24.04）：

```bash
sudo apt install libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev \
                 libayatana-appindicator3-dev librsvg2-dev libzmq3-dev
```

完整部署指南（离线安装、AppImage 排错、CI 模板、卸载）：[`docs/DEPLOYMENT.md`](docs/DEPLOYMENT.md)。

## 从源码构建

```bash
# 装前端依赖
npm install

# 类型检查
npm run typecheck

# 开发模式（热重载）
npm run tauri:dev

# 生产打包（Linux 下产出 deb / rpm / AppImage）
npm run tauri:build
```

需要 Rust 1.75+ 和 Node.js 20+。这两个 `scripts/install.sh` 会自动装。

## 使用方法

1. 在你的 BT.CPP 4.x 应用里挂上 `BT::Groot2Publisher`：

   ```cpp
   BT::Groot2Publisher publisher(tree, 1667);
   ```

2. 启动 btviz（`./target/release/btviz` 或者装好的桌面入口）。
3. 工具栏点 **Connect**（默认 `tcp://127.0.0.1:1667`）。先把整棵树拉一次，之后状态变化通过 ZMQ 流推送。
4. 或者点 **Open File...** 打开静态 `.xml` 做离线分析。
5. 点任意节点查看 ports / 父节点 / 子节点。拖拽 Canvas 平移，滚轮缩放，按 `0` 复位视图。

## 架构

```
src/                   Solid + TypeScript 前端
  components/          TopBar / Canvas (pan/zoom) / BtNode / BtEdge / Sidebar
  stores/              Solid 响应式 store + Tauri 事件监听
  styles/              Tailwind v4 主题 token

src-tauri/             Tauri 2 后端 (Rust)。命令: load_xml / connect_server / disconnect_server
crates/btparse/        BT.CPP 4.x XML 解析器 + Reingold-Tilford 布局
crates/groot2_client/  ZMQ 客户端 (REQ/REP, FullTree + Status)
tools/check-fulltree/  诊断 CLI: 直连 1667 拉 FULLTREE 喂给 btparse
```

## 协议

Apache-2.0
