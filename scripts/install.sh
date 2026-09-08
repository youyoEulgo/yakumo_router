#!/bin/sh
# Install yakumo: use a binary next to this script when present (unpacked
# archive), otherwise download the latest release for this platform.
# It also puts yakumo on PATH and creates a default config.
set -eu

REPO="youyoEulgo/yakumo_router"
BIN_NAME="yakumo"
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" 2>/dev/null && pwd || pwd)
LOCAL_SOURCE="${SCRIPT_DIR}/${BIN_NAME}"

detect_target() {
  os=$(uname -s)
  arch=$(uname -m)

  case "${os}" in
    Linux)
      case "${arch}" in
        x86_64 | amd64) printf 'x86_64-unknown-linux-musl\n' ;;
        aarch64 | arm64) printf 'aarch64-unknown-linux-musl\n' ;;
        *)
          echo "error: no prebuilt binary for Linux/${arch}" >&2
          exit 1
          ;;
      esac
      ;;
    Darwin)
      case "${arch}" in
        arm64 | aarch64) printf 'aarch64-apple-darwin\n' ;;
        *)
          echo "error: no prebuilt binary for macOS/${arch}; build from source" >&2
          exit 1
          ;;
      esac
      ;;
    *)
      echo "error: unsupported operating system: ${os}" >&2
      exit 1
      ;;
  esac
}

fetch_binary() {
  target=$(detect_target)
  version="${YAKUMO_VERSION:-}"

  if [ -z "${version}" ]; then
    version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" |
      grep -m1 '"tag_name"' |
      sed -E 's/.*"tag_name" *: *"([^"]+)".*/\1/')
  fi

  if [ -z "${version}" ]; then
    echo "error: could not determine the latest release" >&2
    exit 1
  fi

  asset="${BIN_NAME}-${version}-${target}.tar.gz"
  url="https://github.com/${REPO}/releases/download/${version}/${asset}"
  tmp_dir=$(mktemp -d)
  trap 'rm -rf "${tmp_dir}"' EXIT

  echo "Downloading ${asset} ..." >&2
  if ! curl -fsSL "${url}" -o "${tmp_dir}/${asset}"; then
    echo "error: failed to download ${url}" >&2
    exit 1
  fi

  tar -xzf "${tmp_dir}/${asset}" -C "${tmp_dir}"
  found=$(find "${tmp_dir}" -type f -name "${BIN_NAME}" | head -n 1)
  if [ -z "${found}" ]; then
    echo "error: ${BIN_NAME} not found inside ${asset}" >&2
    exit 1
  fi

  printf '%s\n' "${found}"
}

if [ -f "${LOCAL_SOURCE}" ]; then
  SOURCE="${LOCAL_SOURCE}"
else
  SOURCE=$(fetch_binary)
fi

INSTALL_DIR="${YAKUMO_INSTALL_DIR:-${XDG_BIN_HOME:-${HOME}/.local/bin}}"
mkdir -p "${INSTALL_DIR}"

if command -v install >/dev/null 2>&1; then
  install -m 0755 "${SOURCE}" "${INSTALL_DIR}/${BIN_NAME}"
else
  cp "${SOURCE}" "${INSTALL_DIR}/${BIN_NAME}"
  chmod 0755 "${INSTALL_DIR}/${BIN_NAME}"
fi

# macOS blocks unsigned downloaded binaries; clear the quarantine flag.
if [ "$(uname -s)" = "Darwin" ] && command -v xattr >/dev/null 2>&1; then
  xattr -dr com.apple.quarantine "${INSTALL_DIR}/${BIN_NAME}" 2>/dev/null || true
fi

PATH_LINE="export PATH=\"${INSTALL_DIR}:\$PATH\""

add_line() {
  target="$1"
  line="$2"
  mkdir -p "$(dirname -- "${target}")"
  touch "${target}"
  if ! grep -Fqs "${line}" "${target}"; then
    printf '\n# Added by the yakumo installer\n%s\n' "${line}" >> "${target}"
  fi
}

skip_path_edit=0
case "${YAKUMO_NO_MODIFY_PATH:-0}" in
  1 | true | yes) skip_path_edit=1 ;;
esac

case ":${PATH}:" in
  *":${INSTALL_DIR}:"*)
    already_on_path=1
    ;;
  *)
    already_on_path=0
    if [ "${skip_path_edit}" = "0" ]; then
      case "$(basename -- "${SHELL:-sh}")" in
        zsh)
          add_line "${HOME}/.zshrc" "${PATH_LINE}"
          ;;
        bash)
          add_line "${HOME}/.bashrc" "${PATH_LINE}"
          ;;
        fish)
          FISH_CONFIG="${HOME}/.config/fish/config.fish"
          mkdir -p "$(dirname -- "${FISH_CONFIG}")"
          touch "${FISH_CONFIG}"
          if ! grep -Fqs "${INSTALL_DIR}" "${FISH_CONFIG}"; then
            printf '\n# Added by the yakumo installer\nfish_add_path "%s"\n' "${INSTALL_DIR}" >> "${FISH_CONFIG}"
          fi
          ;;
        *)
          add_line "${HOME}/.profile" "${PATH_LINE}"
          ;;
      esac
    fi
    ;;
esac

# Create a default config the first time; init is a no-op if one already exists.
"${INSTALL_DIR}/${BIN_NAME}" init >/dev/null 2>&1 || true

echo "yakumo installed to ${INSTALL_DIR}/${BIN_NAME}"
if [ "${already_on_path}" = "1" ]; then
  echo "Run: yakumo"
elif [ "${skip_path_edit}" = "1" ]; then
  echo "Add this to your shell profile to use yakumo:"
  echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
else
  echo "Added ${INSTALL_DIR} to your shell profile."
  echo "Restart your terminal, or run this in the current shell:"
  echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
fi
