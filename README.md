# btviz

A modern, lightweight visualizer for [BehaviorTree.CPP 4.x](https://github.com/BehaviorTree/BehaviorTree.CPP) trees.

Designed as a free alternative to [Groot2](https://www.behaviortree.dev/groot/), without the 20-node limit of the free tier.

## Features

- **Read-only realtime visualization** of any BT.CPP 4.x tree — no node-count limit.
- **Live connection** to a running `BT::Groot2Publisher` over ZMQ (default `tcp://127.0.0.1:1667`).
  - Status diff polling at up to 200 Hz; only changed nodes are pushed to the canvas.
  - Animated transitions on state changes (idle / running / success / failure / skipped).
- **Static file mode** — open any BT XML file off disk for offline inspection.
- **Pan & zoom** canvas (drag to pan, scroll to zoom anchored at cursor, `0` to fit, `+`/`-` to step).
- **Inspector sidebar** with parent jump, child count, and full port table.
- **Cross-platform** desktop app via Tauri 2 (Linux today; macOS / Windows next).

## Install

Pre-built bundles for Linux are produced by `npm run tauri:build`:

| Format | Path | Size |
|---|---|---|
| `.deb` | `target/release/bundle/deb/btviz_0.1.0_amd64.deb` | ~2 MB |
| `.rpm` | `target/release/bundle/rpm/btviz-0.1.0-1.x86_64.rpm` | ~2 MB |
| `.AppImage` | `target/release/bundle/appimage/btviz_0.1.0_amd64.AppImage` | ~75 MB |

System dependencies (Debian / Ubuntu):

```bash
sudo apt install libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev \
                 libayatana-appindicator3-dev librsvg2-dev
```

## Build from source

```bash
# Frontend deps
npm install

# Type check
npm run typecheck

# Dev (hot reload)
npm run tauri:dev

# Production bundles (deb / rpm / AppImage on Linux)
npm run tauri:build
```

## Usage

1. Start your BT.CPP 4.x application with a `BT::Groot2Publisher` attached, e.g.:

   ```cpp
   BT::Groot2Publisher publisher(tree, 1667);
   ```

2. Launch btviz (`./target/release/btviz` or the installed bundle).
3. Click **Connect** in the toolbar — defaults to `tcp://127.0.0.1:1667`. The full tree is fetched once, then status updates stream in.
4. Or click **Open File...** to load a static `.xml` for offline visualization.
5. Click any node to inspect its ports / parent / children. Drag the canvas to pan, scroll to zoom, `0` to refit.

## Architecture

```
src/                 Solid + TypeScript frontend
  components/        TopBar, Canvas (pan/zoom), BtNode, BtEdge, Sidebar
  stores/            Reactive tree + connection state, Tauri event listeners
  styles/            Tailwind v4 theme tokens

src-tauri/           Tauri 2 backend (Rust). Commands: load_xml, connect_server, disconnect_server.
crates/btparse/      BT.CPP 4.x XML parser + Reingold-Tilford layout.
crates/groot2_client/ZMQ client speaking the Groot2 wire protocol (REQ/REP, FullTree + Status).
tools/check-fulltree/Diagnostic CLI: pulls FULLTREE off a live publisher and feeds it through btparse.
```

## License

Apache-2.0
