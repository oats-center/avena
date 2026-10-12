---
title: "Install the avena CLI"
---

Every Avena image already includes the `avena` CLI. You only need to install it yourself to
control devices **from a workstation**.

## Prebuilt binaries

Each Avena release publishes static binaries for Linux and macOS:

```bash
# Linux x86_64
curl -LO https://github.com/oats-center/avena/releases/latest/download/avena-x86_64-unknown-linux-musl.tar.gz
tar -xzf avena-x86_64-unknown-linux-musl.tar.gz
sudo install -m 0755 avena /usr/local/bin/avena
```

| Platform        | Asset                                         |
|-----------------|-----------------------------------------------|
| Linux x86_64    | `avena-x86_64-unknown-linux-musl.tar.gz`      |
| Linux aarch64   | `avena-aarch64-unknown-linux-musl.tar.gz`     |
| macOS (Apple)   | `avena-aarch64-apple-darwin.tar.gz`           |

## From source

The CLI is the `avena-cli` crate in this repository:

```bash
cargo install --locked --git https://github.com/oats-center/avena avena-cli
```

## Verify

```bash
avena --version
```

```text
avena 1.4.0 (trellis protocol 3)
```

## Shell completion

```bash
avena completion bash > ~/.local/share/bash-completion/completions/avena
avena completion zsh  > "${fpath[1]}/_avena"
avena completion fish > ~/.config/fish/completions/avena.fish
```
