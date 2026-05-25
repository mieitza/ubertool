#!/usr/bin/env sh
# ubertool installer
#
# Quick install (latest):
#   curl -fsSL https://raw.githubusercontent.com/mieitza/ubertool/main/install.sh | sh
#
# Pin a version:
#   curl -fsSL .../install.sh | sh -s -- --version v0.1.0
#
# Install binary + Claude agent skill:
#   curl -fsSL .../install.sh | sh -s -- --with-claude-skill
#
# Override install dir:
#   curl -fsSL .../install.sh | sh -s -- --bin-dir /usr/local/bin
#
# Install shell completions (auto-detects shell from $SHELL):
#   curl -fsSL .../install.sh | sh -s -- --with-completions
#
# Install completions to a custom directory:
#   curl -fsSL .../install.sh | sh -s -- --with-completions --completions-dir /some/dir
#
# Install binary + agent skill + shell completions (everything):
#   curl -fsSL .../install.sh | sh -s -- --with-claude-skill --with-completions
#
# Env var overrides:
#   UBERTOOL_BIN_DIR, UBERTOOL_VERSION, UBERTOOL_SKILL_DIR, UBERTOOL_COMPLETIONS_DIR

set -eu

REPO="mieitza/ubertool"
BIN_DIR="${UBERTOOL_BIN_DIR:-$HOME/.local/bin}"
VERSION="${UBERTOOL_VERSION:-latest}"
WITH_CLAUDE_SKILL=0
SKILL_DIR="${UBERTOOL_SKILL_DIR:-$HOME/.claude/skills/ubertool}"
WITH_COMPLETIONS=0
COMPLETIONS_DIR="${UBERTOOL_COMPLETIONS_DIR:-}"

# ---- parse args ----
while [ $# -gt 0 ]; do
    case "$1" in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --bin-dir)
            BIN_DIR="$2"
            shift 2
            ;;
        --with-claude-skill)
            WITH_CLAUDE_SKILL=1
            shift
            ;;
        --skill-dir)
            SKILL_DIR="$2"
            shift 2
            ;;
        --with-completions)
            WITH_COMPLETIONS=1
            shift
            ;;
        --completions-dir)
            COMPLETIONS_DIR="$2"
            shift 2
            ;;
        -h|--help)
            sed -n 's/^# //p;s/^#$//p' "$0" | head -26
            exit 0
            ;;
        *)
            echo "ubertool installer: unknown flag: $1" >&2
            echo "run with --help for usage" >&2
            exit 2
            ;;
    esac
done

# ---- detect platform ----
os_lower=$(uname -s | tr '[:upper:]' '[:lower:]')
arch_raw=$(uname -m)

case "$os_lower" in
    linux)
        os="linux"
        ;;
    darwin)
        os="darwin"
        ;;
    msys*|mingw*|cygwin*)
        os="windows"
        ;;
    *)
        echo "ubertool installer: unsupported OS: $os_lower" >&2
        exit 1
        ;;
esac

case "$arch_raw" in
    x86_64|amd64)
        arch="x86_64"
        ;;
    arm64|aarch64)
        arch="aarch64"
        ;;
    *)
        echo "ubertool installer: unsupported arch: $arch_raw" >&2
        exit 1
        ;;
esac

case "$os" in
    linux)
        target="${arch}-unknown-linux-musl"
        ext="tar.gz"
        bin_name="ubertool"
        ;;
    darwin)
        target="${arch}-apple-darwin"
        ext="tar.gz"
        bin_name="ubertool"
        ;;
    windows)
        if [ "$arch" = "aarch64" ]; then
            echo "ubertool installer: aarch64-windows is not currently published" >&2
            exit 1
        fi
        target="${arch}-pc-windows-msvc"
        ext="zip"
        bin_name="ubertool.exe"
        ;;
esac

asset="ubertool-${target}.${ext}"

if [ "$VERSION" = "latest" ]; then
    url="https://github.com/${REPO}/releases/latest/download/${asset}"
else
    url="https://github.com/${REPO}/releases/download/${VERSION}/${asset}"
fi

# ---- download + extract ----
echo "Installing ubertool ($target, $VERSION) to $BIN_DIR"

tmpdir=$(mktemp -d 2>/dev/null || mktemp -d -t ubertool-install)
trap 'rm -rf "$tmpdir"' EXIT

archive="$tmpdir/$asset"

if command -v curl >/dev/null 2>&1; then
    curl --fail --location --silent --show-error --output "$archive" "$url"
elif command -v wget >/dev/null 2>&1; then
    wget --quiet --output-document "$archive" "$url"
else
    echo "ubertool installer: need curl or wget on PATH" >&2
    exit 1
fi

cd "$tmpdir"
case "$ext" in
    tar.gz)
        tar -xzf "$asset"
        ;;
    zip)
        if command -v unzip >/dev/null 2>&1; then
            unzip -q "$asset"
        else
            echo "ubertool installer: need unzip on PATH to extract Windows archive" >&2
            exit 1
        fi
        ;;
esac

mkdir -p "$BIN_DIR"
mv "$bin_name" "$BIN_DIR/$bin_name"
chmod +x "$BIN_DIR/$bin_name"

cd - >/dev/null

# ---- optional Claude skill install ----
if [ "$WITH_CLAUDE_SKILL" = "1" ]; then
    skill_url="https://raw.githubusercontent.com/${REPO}/main/docs/claude-skill/SKILL.md"
    echo "Installing Claude agent skill to $SKILL_DIR"
    mkdir -p "$SKILL_DIR"
    if command -v curl >/dev/null 2>&1; then
        curl --fail --location --silent --show-error --output "$SKILL_DIR/SKILL.md" "$skill_url"
    else
        wget --quiet --output-document "$SKILL_DIR/SKILL.md" "$skill_url"
    fi
fi

# ---- optional shell completion install ----
COMPLETIONS_PATH=""
if [ "$WITH_COMPLETIONS" = "1" ]; then
    shell_name=$(basename "${SHELL:-}")

    # Determine target path (custom dir overrides shell convention)
    if [ -n "$COMPLETIONS_DIR" ]; then
        case "$shell_name" in
            zsh)        comp_file="_ubertool" ;;
            fish)       comp_file="ubertool.fish" ;;
            powershell) comp_file="ubertool.ps1" ;;
            elvish)     comp_file="ubertool.elv" ;;
            *)          comp_file="ubertool" ;;
        esac
        COMPLETIONS_PATH="$COMPLETIONS_DIR/$comp_file"
    else
        case "$shell_name" in
            bash)
                COMPLETIONS_PATH="$HOME/.local/share/bash-completion/completions/ubertool"
                ;;
            zsh)
                COMPLETIONS_PATH="$HOME/.zsh/completions/_ubertool"
                ;;
            fish)
                COMPLETIONS_PATH="$HOME/.config/fish/completions/ubertool.fish"
                ;;
            powershell|elvish)
                echo ""
                echo "  completions: auto-install not supported for $shell_name."
                echo "    Run the following and source the output in your profile:"
                echo "      ubertool completions $shell_name"
                ;;
            "")
                echo ""
                echo "  completions: \$SHELL is unset; skipping completion install."
                echo "    Run 'ubertool completions <shell>' manually."
                ;;
            *)
                echo ""
                echo "  completions: unrecognised shell '$shell_name'; skipping."
                echo "    Run 'ubertool completions <shell>' manually."
                ;;
        esac
    fi

    if [ -n "$COMPLETIONS_PATH" ]; then
        comp_dir=$(dirname "$COMPLETIONS_PATH")
        mkdir -p "$comp_dir"
        "$BIN_DIR/$bin_name" completions "$shell_name" > "$COMPLETIONS_PATH"
        echo "Installing shell completions ($shell_name) to $COMPLETIONS_PATH"
        if [ "$shell_name" = "zsh" ] && [ -z "$COMPLETIONS_DIR" ]; then
            echo "  Note: add the following to your ~/.zshrc BEFORE 'compinit':"
            echo "    fpath=(~/.zsh/completions \$fpath)"
        fi
    fi
fi

# ---- post-install ----
echo ""
echo "  installed: $BIN_DIR/$bin_name"
if [ "$WITH_CLAUDE_SKILL" = "1" ]; then
    echo "  skill:     $SKILL_DIR/SKILL.md"
fi
if [ -n "$COMPLETIONS_PATH" ]; then
    echo "  completions: $COMPLETIONS_PATH"
fi
echo ""

case ":$PATH:" in
    *":$BIN_DIR:"*)
        ;;
    *)
        echo "  $BIN_DIR is not on your PATH. Add this to your shell rc:"
        echo ""
        echo "    export PATH=\"$BIN_DIR:\$PATH\""
        echo ""
        ;;
esac

"$BIN_DIR/$bin_name" --version 2>/dev/null || true
