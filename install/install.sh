#!/usr/bin/env sh

# exit on any error
set -euo pipefail

NAME="git-kit"
BIN=${BIN:-"/usr/local/bin"}

# colors
RED='\033[0;31m'
ORANGE='\033[0;33m'
NONE='\033[0m'

# logs
err() {
    echo "$1" >&2
}

error() {
    err "🙈 ${RED}error:${NONE} $1"
}

# utils
is_installed() {
    command -v "$1" 1>/dev/null 2>&1
}

derive_target_config() {
    plat=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "${plat}" in
        darwin) echo "$HOME/Library/Application Support/dev.xsv24.$NAME" ;;
        *) error "Currently unsupported OS platform '${plat}'" && exit 1 ;;
    esac
}

derive_zip_ext() {
    plat=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "${plat}" in
        *windows*) echo ".zip" ;;
        *) echo ".tar.gz"
    esac
}

derive_binary_name() {
    plat=$(uname -s | tr '[:upper:]' '[:lower:]')
    case "${plat}" in
        darwin) plat="apple-darwin" ;;
        # Add as needed
        *) error "Currently unsupported OS platform '${plat}'" && exit 1 ;;
    esac
    
    arch=$(uname -m | tr '[:upper:]' '[:lower:]')
    case "${arch}" in
        amd64 | x86_64) arch="x86_64" ;;
        armv*) arch="arm" ;;
        arm64) arch="aarch64" ;;
        *) error "Currently unsupported architecture '${arch}'" && exit 1 ;;
    esac

    echo "$NAME--$arch-$plat"
}

unzip() {
    printf "⏳ Unzipping binary..."

    path="$1"
    to="$2"
    ext=$(derive_zip_ext)

    case "$ext" in
    *.tar.gz)
        # TODO: Look at these flags?
        flags=$(test -n "${VERBOSE-}" && echo "-xzvof" || echo "-xzof")
        tar "$flags" "$path" -C "$to"
        ;;
    *.zip)
        flags=$(test -z "${VERBOSE-}" && echo "-qqo" || echo "-o")
        "$flags" unzip "$path" -d "$to"
        ;;
    *)
        error "Unsupported compressed file type ${path}"
        exit 1
        ;;
    esac

    echo " ✅"
}

get_http_client() {
    if is_installed curl; then
        echo "curl --fail --silent --location --output"
    elif is_installed wget; then
        echo "wget --quiet --output-document="
    elif is_installed fetch; then
        echo "fetch --quiet --output="
    else
        error "Could not find http client please install one of the following:"
        err "→ curl, wget or fetch"
        exit 1
    fi
}

download() {
    file="$1"
    binary_name="$2"
    ext=$(derive_zip_ext)

    printf "⏳ Downlaoding binary %s..." "$binary_name"

    release="https://github.com/xsv24/$NAME/releases/latest/download/${binary_name}${ext}"

    request="$(get_http_client) $file $release"
    # execute request 
    $request && echo " ✅" && return 0

    echo ""
    error "Failed to download latest $NAME release for binary '${binary_name}'"
    exit 1
}

default_template_config() {
    binary_name="$1"
    uncompressed="$2"
    location="$uncompressed/$binary_name"
    
    printf "⏳ Configuring..."
    mv "$location/$NAME" "$BIN"

    target_config=$(derive_target_config)
    mkdir -p "$target_config"

    mv "$location/conventional.yml" "$target_config" 
    mv "$location/default.yml" "$target_config"

    echo " ✅"
}

main() {
    echo "⏳ Installing $NAME..."
    binary_name=$(derive_binary_name)
    compressed=$(mktemp)
    uncompressed=$(mktemp -d)

    download "$compressed" "$binary_name"
    unzip "$compressed" "$uncompressed"
    rm -r "$compressed"

    default_template_config "$binary_name" "$uncompressed"
    rm -r "$uncompressed"

    echo "🚀 ${ORANGE}$NAME${NONE} is now installed!"
    echo ""
    echo "✨ Get started with ↓"
    echo "> $NAME --help"
}

main
