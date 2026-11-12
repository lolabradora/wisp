#!/bin/sh
# Installs the prebuilt wisp binary for this machine from the GitHub releases.
#
#   curl -fsSL https://raw.githubusercontent.com/lolabradora/wisp/main/install.sh | sh
#
#   WISP_VERSION=v0.2.0   install that release instead of the latest
#   WISP_INSTALL_DIR=DIR  install into DIR instead of ~/.local/bin
#   WISP_BASE_URL=URL     download the archive from URL/ instead of GitHub (mirrors, tests)
set -eu

REPO=lolabradora/wisp

say() {
  printf '%s\n' "$*"
}

die() {
  printf 'wisp install: %s\n' "$*" >&2
  exit 1
}

have() {
  command -v "$1" >/dev/null 2>&1
}

# Prints the release target triple for this machine.
detect_target() {
  os=$(uname -s)
  arch=$(uname -m)
  case $os in
    Darwin)
      # A shell under Rosetta reports x86_64 on Apple Silicon; the native build is faster.
      if [ "$(sysctl -n sysctl.proc_translated 2>/dev/null || true)" = 1 ]; then
        arch=arm64
      fi
      vendor_os=apple-darwin
      ;;
    Linux)
      # The release binaries link against glibc.
      if ldd --version 2>&1 | grep -qi musl; then
        die "no prebuilt binary for musl Linux; use: cargo install --git https://github.com/$REPO"
      fi
      vendor_os=unknown-linux-gnu
      ;;
    *)
      die "unsupported operating system: $os (wisp runs on macOS and Linux)"
      ;;
  esac
  case $arch in
    arm64 | aarch64) cpu=aarch64 ;;
    x86_64 | amd64) cpu=x86_64 ;;
    *) die "unsupported CPU architecture: $arch (need arm64/aarch64 or x86_64)" ;;
  esac
  say "$cpu-$vendor_os"
}

# download URL FILE
download() {
  if have curl; then
    curl -fsSL --retry 3 -o "$2" "$1"
  elif have wget; then
    wget -q -O "$2" "$1"
  else
    die "curl or wget is required"
  fi
}

sha256_of() {
  if have sha256sum; then
    sha256sum "$1" | cut -d ' ' -f 1
  elif have shasum; then
    shasum -a 256 "$1" | cut -d ' ' -f 1
  else
    die "sha256sum or shasum is required to verify the download"
  fi
}

main() {
  have tar || die "tar is required"
  target=$(detect_target)
  asset=wisp-$target.tar.gz

  version=${WISP_VERSION:-}
  case $version in
    '' | v*) ;;
    *) version=v$version ;;
  esac
  if [ -n "${WISP_BASE_URL:-}" ]; then
    base=${WISP_BASE_URL%/}
  elif [ -n "$version" ]; then
    base=https://github.com/$REPO/releases/download/$version
  else
    base=https://github.com/$REPO/releases/latest/download
  fi

  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  trap 'exit 1' INT TERM

  say "Downloading $asset (${version:-latest release})"
  download "$base/$asset" "$tmp/$asset" ||
    die "could not download $base/$asset (is there a release for $target?)"
  download "$base/$asset.sha256" "$tmp/$asset.sha256" ||
    die "could not download $base/$asset.sha256"

  expected=$(cut -d ' ' -f 1 <"$tmp/$asset.sha256")
  actual=$(sha256_of "$tmp/$asset")
  [ -n "$expected" ] || die "the checksum file is empty"
  [ "$expected" = "$actual" ] ||
    die "checksum mismatch for $asset: expected $expected, got $actual"

  mkdir "$tmp/unpacked"
  tar -xzf "$tmp/$asset" -C "$tmp/unpacked"
  [ -f "$tmp/unpacked/wisp" ] || die "$asset does not contain the wisp binary"

  dir=${WISP_INSTALL_DIR:-$HOME/.local/bin}
  mkdir -p "$dir"
  # Copied next to the destination and renamed, so a running wisp is replaced in one step.
  cp "$tmp/unpacked/wisp" "$dir/.wisp.new"
  chmod 755 "$dir/.wisp.new"
  mv -f "$dir/.wisp.new" "$dir/wisp"
  say "Installed $("$dir/wisp" --version 2>/dev/null || say wisp) to $dir/wisp"

  case ":$PATH:" in
    *":$dir:"*) ;;
    *)
      say ""
      say "Warning: $dir is not on your PATH. Add this to ~/.zshrc:"
      say "  export PATH=\"$dir:\$PATH\""
      ;;
  esac

  say ""
  say "Next steps:"
  if ! have llama-server; then
    if [ "$(uname -s)" = Darwin ]; then
      say "  brew install llama.cpp"
    else
      say "  install llama.cpp so that llama-server is on your PATH:"
      say "    brew install llama.cpp, or https://github.com/ggml-org/llama.cpp/releases"
    fi
  fi
  say "  wisp setup                                  # model download + background service"
  say "  echo 'eval \"\$(wisp init zsh)\"' >> ~/.zshrc   # then open a new shell"
}

# Called on the last line, so a download cut short runs nothing.
main "$@"
