#!/usr/bin/env bash
# btviz one-shot installer.
#
# Usage:
#   ./scripts/install.sh [deps|build|bundle|dev|uninstall]
#
# Default subcommand: build (full deps + compile + install to INSTALL_PREFIX/bin).
#
# Env knobs:
#   INSTALL_PREFIX   install prefix for `build` (default: /usr/local)
#   RELEASE_PROFILE  cargo profile: release | debug   (default: release)
#   SKIP_RUST=1      skip rustup install (already have a toolchain)
#   SKIP_NODE=1      skip Node.js install (already have one)
#   ASSUME_YES=1     never prompt (for CI / scripted runs)

set -euo pipefail

# ---- Paths ------------------------------------------------------------------
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." &> /dev/null && pwd)"

INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
RELEASE_PROFILE="${RELEASE_PROFILE:-release}"
ASSUME_YES="${ASSUME_YES:-0}"

# ---- Pretty printing --------------------------------------------------------
if [[ -t 1 ]]; then
    C_RESET=$'\033[0m'; C_BOLD=$'\033[1m'
    C_RED=$'\033[31m'; C_GREEN=$'\033[32m'; C_YELLOW=$'\033[33m'; C_CYAN=$'\033[36m'
else
    C_RESET=""; C_BOLD=""; C_RED=""; C_GREEN=""; C_YELLOW=""; C_CYAN=""
fi
log()  { printf '%s[btviz]%s %s\n' "${C_CYAN}${C_BOLD}" "${C_RESET}" "$*"; }
ok()   { printf '%s[ok]%s %s\n'    "${C_GREEN}${C_BOLD}" "${C_RESET}" "$*"; }
warn() { printf '%s[warn]%s %s\n'  "${C_YELLOW}${C_BOLD}" "${C_RESET}" "$*" >&2; }
die()  { printf '%s[fail]%s %s\n'  "${C_RED}${C_BOLD}"    "${C_RESET}" "$*" >&2; exit 1; }

# ---- Sudo helper ------------------------------------------------------------
SUDO=""
if [[ $EUID -ne 0 ]]; then
    if command -v sudo >/dev/null 2>&1; then
        SUDO="sudo"
    else
        die "需要 root 权限（或先装 sudo）"
    fi
fi

confirm() {
    [[ "${ASSUME_YES}" == "1" ]] && return 0
    local prompt="${1:-继续？} [y/N] "
    read -r -p "$prompt" reply
    [[ "$reply" =~ ^[Yy]$ ]]
}

# ---- OS detection -----------------------------------------------------------
OS_ID=""
OS_VERSION_ID=""
PKG_MGR=""

detect_os() {
    if [[ ! -r /etc/os-release ]]; then
        die "无法识别系统：/etc/os-release 缺失"
    fi
    # shellcheck source=/dev/null
    . /etc/os-release
    OS_ID="${ID:-unknown}"
    OS_VERSION_ID="${VERSION_ID:-unknown}"

    case "$OS_ID" in
        ubuntu|debian|linuxmint|pop) PKG_MGR="apt" ;;
        fedora|rhel|centos|rocky|alma) PKG_MGR="dnf" ;;
        arch|manjaro|endeavouros) PKG_MGR="pacman" ;;
        *)
            warn "未识别的发行版 ($OS_ID)，将尝试通用流程"
            if   command -v apt    >/dev/null 2>&1; then PKG_MGR="apt"
            elif command -v dnf    >/dev/null 2>&1; then PKG_MGR="dnf"
            elif command -v pacman >/dev/null 2>&1; then PKG_MGR="pacman"
            else die "未识别的包管理器"
            fi
            ;;
    esac

    # WebKitGTK 4.1 is mandatory for Tauri 2; Ubuntu 22.04 ships only 4.0.
    if [[ "$OS_ID" == "ubuntu" && "$OS_VERSION_ID" == "22.04" ]]; then
        warn "Ubuntu 22.04 默认源只有 webkit2gtk-4.0；Tauri 2 需要 4.1"
        warn "建议升级到 24.04，或直接使用 AppImage 预构建包"
        confirm "仍然继续？" || die "用户中止"
    fi
}

# ---- Package install --------------------------------------------------------
install_runtime_deps() {
    log "安装运行时依赖 ($PKG_MGR)"
    case "$PKG_MGR" in
        apt)
            $SUDO apt-get update
            $SUDO apt-get install -y \
                libwebkit2gtk-4.1-0 \
                libgtk-3-0 \
                librsvg2-2 \
                libayatana-appindicator3-1 \
                libssl3 \
                libzmq5 \
                ca-certificates
            ;;
        dnf)
            $SUDO dnf install -y \
                webkit2gtk4.1 \
                gtk3 \
                librsvg2 \
                libappindicator-gtk3 \
                openssl-libs \
                zeromq \
                ca-certificates
            ;;
        pacman)
            $SUDO pacman -Sy --needed --noconfirm \
                webkit2gtk-4.1 \
                gtk3 \
                librsvg \
                libappindicator-gtk3 \
                openssl \
                zeromq \
                ca-certificates
            ;;
    esac
    ok "运行时依赖装好"
}

install_build_deps() {
    log "安装构建依赖 ($PKG_MGR)"
    case "$PKG_MGR" in
        apt)
            $SUDO apt-get update
            $SUDO apt-get install -y \
                build-essential \
                pkg-config \
                curl \
                git \
                libwebkit2gtk-4.1-dev \
                libgtk-3-dev \
                libssl-dev \
                libayatana-appindicator3-dev \
                librsvg2-dev \
                libzmq3-dev
            ;;
        dnf)
            $SUDO dnf install -y \
                gcc gcc-c++ make \
                pkgconf-pkg-config \
                curl git \
                webkit2gtk4.1-devel \
                gtk3-devel \
                openssl-devel \
                libappindicator-gtk3-devel \
                librsvg2-devel \
                zeromq-devel
            ;;
        pacman)
            $SUDO pacman -Sy --needed --noconfirm \
                base-devel \
                pkgconf \
                curl git \
                webkit2gtk-4.1 \
                gtk3 \
                openssl \
                libappindicator-gtk3 \
                librsvg \
                zeromq
            ;;
    esac
    ok "构建依赖装好"
}

# ---- Toolchains -------------------------------------------------------------
ensure_rust() {
    if [[ "${SKIP_RUST:-0}" == "1" ]]; then
        ok "SKIP_RUST=1，跳过 rustup"
        return
    fi
    if command -v cargo >/dev/null 2>&1 && command -v rustc >/dev/null 2>&1; then
        ok "Rust 已安装：$(rustc --version)"
        return
    fi
    log "安装 Rust 工具链 (rustup, stable, default profile)"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
        sh -s -- -y --default-toolchain stable --profile default
    # shellcheck source=/dev/null
    . "$HOME/.cargo/env"
    ok "Rust 装好：$(rustc --version)"
}

ensure_node() {
    if [[ "${SKIP_NODE:-0}" == "1" ]]; then
        ok "SKIP_NODE=1，跳过 Node 安装"
        return
    fi
    if command -v node >/dev/null 2>&1; then
        local node_major
        node_major="$(node -v | sed 's/^v\([0-9]*\).*/\1/')"
        if (( node_major >= 18 )); then
            ok "Node 已安装：$(node -v)"
            return
        fi
        warn "已有 Node 版本过旧 ($(node -v))，升级到 20"
    fi
    log "安装 Node.js 20"
    case "$PKG_MGR" in
        apt)
            curl -fsSL https://deb.nodesource.com/setup_20.x | $SUDO -E bash -
            $SUDO apt-get install -y nodejs
            ;;
        dnf)
            curl -fsSL https://rpm.nodesource.com/setup_20.x | $SUDO -E bash -
            $SUDO dnf install -y nodejs
            ;;
        pacman)
            $SUDO pacman -Sy --needed --noconfirm nodejs npm
            ;;
    esac
    ok "Node 装好：$(node -v)"
}

# ---- Build / install --------------------------------------------------------
prepare_path() {
    [[ -f "$HOME/.cargo/env" ]] && . "$HOME/.cargo/env"
    export PATH="$HOME/.cargo/bin:$PATH"
}

frontend_install() {
    log "npm install (前端依赖)"
    (cd "$REPO_ROOT" && npm install --no-audit --no-fund)
    ok "前端依赖装好"
}

build_binary() {
    prepare_path
    log "构建 btviz 二进制 (profile=$RELEASE_PROFILE)"
    (cd "$REPO_ROOT" && npm run build)
    local cargo_args=()
    [[ "$RELEASE_PROFILE" == "release" ]] && cargo_args+=(--release)
    (cd "$REPO_ROOT/src-tauri" && cargo build "${cargo_args[@]}")
    ok "构建完成"
}

build_bundles() {
    prepare_path
    log "运行 npm run tauri:build (产出 deb / rpm / AppImage)"
    (cd "$REPO_ROOT" && npm run tauri:build)
    ok "Bundles 落在 $REPO_ROOT/target/release/bundle/"
    find "$REPO_ROOT/target/release/bundle" -maxdepth 3 \
        \( -name '*.deb' -o -name '*.rpm' -o -name '*.AppImage' \) \
        -printf '  %p (%s bytes)\n' 2>/dev/null || true
}

install_binary_to_prefix() {
    local src dst desktop_dir desktop_file icon_src icon_dst
    if [[ "$RELEASE_PROFILE" == "release" ]]; then
        src="$REPO_ROOT/target/release/btviz"
    else
        src="$REPO_ROOT/target/debug/btviz"
    fi
    [[ -x "$src" ]] || die "找不到二进制：$src"
    dst="$INSTALL_PREFIX/bin/btviz"

    log "安装二进制 -> $dst"
    $SUDO install -d "$(dirname "$dst")"
    $SUDO install -m 0755 "$src" "$dst"

    # User-level desktop entry so the app shows up in launchers.
    desktop_dir="$HOME/.local/share/applications"
    desktop_file="$desktop_dir/btviz.desktop"
    mkdir -p "$desktop_dir"
    icon_src="$REPO_ROOT/src-tauri/icons/128x128.png"
    icon_dst="$HOME/.local/share/icons/btviz.png"
    if [[ -f "$icon_src" ]]; then
        mkdir -p "$(dirname "$icon_dst")"
        cp "$icon_src" "$icon_dst"
    fi
    cat > "$desktop_file" <<EOF
[Desktop Entry]
Type=Application
Name=btviz
GenericName=BehaviorTree Visualizer
Comment=Modern visualizer for BehaviorTree.CPP 4.x trees
Exec=$dst
Icon=${icon_dst:-btviz}
Terminal=false
Categories=Development;Utility;
StartupNotify=true
EOF
    ok "桌面入口：$desktop_file"
    ok "完成。运行：${C_BOLD}btviz${C_RESET}"
}

uninstall() {
    log "卸载"
    $SUDO rm -f "$INSTALL_PREFIX/bin/btviz"
    rm -f "$HOME/.local/share/applications/btviz.desktop"
    rm -f "$HOME/.local/share/icons/btviz.png"
    ok "已删除二进制 + 桌面入口"
    warn "如用 .deb / .rpm 安装请改用包管理器卸载（apt remove btviz / dnf remove btviz）"
}

# ---- Subcommands ------------------------------------------------------------
cmd_deps() {
    detect_os
    install_runtime_deps
    install_build_deps
    ensure_rust
    ensure_node
}

cmd_build() {
    detect_os
    install_runtime_deps
    install_build_deps
    ensure_rust
    ensure_node
    frontend_install
    build_binary
    install_binary_to_prefix
}

cmd_bundle() {
    detect_os
    install_runtime_deps
    install_build_deps
    ensure_rust
    ensure_node
    frontend_install
    build_bundles
}

cmd_dev() {
    detect_os
    install_runtime_deps
    install_build_deps
    ensure_rust
    ensure_node
    frontend_install
    prepare_path
    log "启动开发模式 (npm run tauri:dev)，Ctrl-C 退出"
    (cd "$REPO_ROOT" && exec npm run tauri:dev)
}

cmd_help() {
    cat <<EOF
${C_BOLD}btviz installer${C_RESET} — Tauri 2 桌面应用一键部署

Usage:
  $0 [subcommand]

Subcommands:
  deps        仅装系统依赖 + Rust + Node
  build       deps + 编 release 二进制 + 安装到 \$INSTALL_PREFIX/bin (默认)
  bundle      deps + 产出 .deb / .rpm / .AppImage
  dev         deps + 启动 npm run tauri:dev
  uninstall   删除二进制 + 桌面入口
  help        显示本帮助

Env:
  INSTALL_PREFIX=$INSTALL_PREFIX       (build 默认 /usr/local)
  RELEASE_PROFILE=$RELEASE_PROFILE     (release | debug)
  SKIP_RUST=${SKIP_RUST:-0}            (1 跳过 rustup)
  SKIP_NODE=${SKIP_NODE:-0}            (1 跳过 nodesource)
  ASSUME_YES=${ASSUME_YES:-0}          (1 无人值守，跳过确认)

详细文档：docs/DEPLOYMENT.md
EOF
}

main() {
    local sub="${1:-build}"
    case "$sub" in
        deps)        cmd_deps ;;
        build)       cmd_build ;;
        bundle)      cmd_bundle ;;
        dev)         cmd_dev ;;
        uninstall)   uninstall ;;
        help|-h|--help) cmd_help ;;
        *)
            warn "未知子命令：$sub"
            cmd_help
            exit 1
            ;;
    esac
}

main "$@"
