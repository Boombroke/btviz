# btviz

A modern, lightweight visualizer for [BehaviorTree.CPP 4.x](https://github.com/BehaviorTree/BehaviorTree.CPP) trees.

Designed as a free alternative to [Groot2](https://www.behaviortree.dev/groot/), without the 20-node limit
of the free tier.

## Goals

- Read-only realtime visualization of any BT.CPP 4.x tree
- Connects to a running `BT::Groot2Publisher` over ZMQ (port 1667)
- Modern UI (Tauri + Solid + SVG + animated transitions)
- No node limit
- Cross-platform (Linux first, then macOS / Windows)

## Status

Bootstrapping. Targets a 3-day MVP:

- Day 1: BT XML parsing + Reingold-Tilford layout, Tauri/Solid skeleton
- Day 2: Groot2 ZMQ protocol client, status polling at the BT tick rate, animated state highlights
- Day 3: pan/zoom, node detail panel, packaging

## License

Apache-2.0
