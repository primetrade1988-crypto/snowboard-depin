#!/usr/bin/env bash
set -euo pipefail
export PATH="$HOME/.cargo/bin:$HOME/.local/share/solana/install/active_release/bin:$PATH"

echo "=== 1. rustup ==="
if ! command -v rustc >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi
rustc --version
cargo --version

echo "=== 2. solana ==="
if ! command -v solana >/dev/null 2>&1; then
  curl -sSfL https://release.anza.xyz/v1.18.26/install | sh
fi
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
solana --version

echo "=== 3. node ==="
if ! command -v node >/dev/null 2>&1; then
  curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
  sudo apt-get install -y nodejs
fi
node --version
npm --version

echo "=== 4. anchor-cli ==="
if ! command -v anchor >/dev/null 2>&1; then
  mkdir -p "$HOME/.local/bin"
  # try prebuilt first
  if curl -fsSL -o /tmp/anchor.tar.gz "https://github.com/coral-xyz/anchor/releases/download/v0.30.1/anchor-linux-x86_64.tar.gz"; then
    tar -xzf /tmp/anchor.tar.gz -C /tmp || true
  fi
  if [ -f /tmp/anchor ]; then
    mv /tmp/anchor "$HOME/.local/bin/anchor"
    chmod +x "$HOME/.local/bin/anchor"
  else
    echo "prebuilt missing, cargo-installing avm..."
    cargo install --git https://github.com/coral-xyz/anchor --tag v0.30.1 anchor-cli --locked --force
  fi
fi
export PATH="$HOME/.local/bin:$PATH"
anchor --version || true

echo "=== TOOLCHAIN_READY ==="
which rustc solana node
command -v anchor && anchor --version || echo "anchor missing"
