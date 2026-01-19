#!/usr/bin/env bash
set -euo pipefail

BIN=jottty
REPO='j0tlabs/jottty'
INSTALL_DIR="/usr/local/bin"
DOWNLOAD_DIR=''
VERSION=''

print_help() {
    echo "Installs latest (or specific) version of jottty. Installation directory defaults to /usr/local/bin."
    echo -e
    echo "Usage:"
    echo "install.sh [--dir <dir>] [--download-dir <download-dir>] [--version <version>]"
    echo -e
    echo "Defaults:"
    echo " * Installation directory: ${INSTALL_DIR}"
    echo " * Download directory: temporary"
    echo " * Version: <Latest release on GitHub>"
    exit 1
}

while [[ $# -gt 0 ]]; do
    key="$1"
    case "$key" in
        -h|--help)
            print_help
            ;;
        --dir)
            INSTALL_DIR="$2"
            shift
            shift
            ;;
        --download-dir)
            DOWNLOAD_DIR="$2"
            shift
            shift
            ;;
        --version)
            VERSION="$2"
            shift
            shift
            ;;
        *)  # unknown option
            print_help
            ;;
    esac
done

if [[ -z "$DOWNLOAD_DIR" ]]; then
    DOWNLOAD_DIR="$(mktemp -d)"
    trap 'rm -rf "$DOWNLOAD_DIR"' EXIT
fi

if [[ -z "$VERSION" ]]; then
    VERSION=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" \
        | sed -n 's/.*"tag_name": "\(.*\)".*/\1/p')
fi

if [[ -z "$VERSION" ]]; then
    echo "Unable to determine latest version. Use --version to set it explicitly."
    exit 1
fi

case "$(uname -s)" in
    Linux*)  PLATFORM=unknown-linux-gnu;;
    Darwin*) PLATFORM=apple-darwin;;
    *) echo "Unsupported OS: $(uname -s)"; exit 1;;
esac

case "$(uname -m)" in
    x86_64*) ARCH=x86_64;;
    aarch64*) ARCH=aarch64;;
    arm64*) ARCH=aarch64;;
    *) echo "Unsupported architecture: $(uname -m)"; exit 1;;
esac

FILEEXT=zip
FILENAME=jottty-${VERSION}-${ARCH}-${PLATFORM}
FILE=${FILENAME}.${FILEEXT}
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILE}"

if [[ "$FILEEXT" == "zip" ]]; then
    if ! command -v unzip >/dev/null 2>&1; then
        echo "Missing unzip. Please install it and re-run."
        exit 1
    fi
    UTIL="unzip -qqo"
else
    if ! command -v tar >/dev/null 2>&1; then
        echo "Missing tar. Please install it and re-run."
        exit 1
    fi
    UTIL="tar -zxf"
fi

mkdir -p "$DOWNLOAD_DIR" && (
    cd "$DOWNLOAD_DIR"
    echo "Downloading $DOWNLOAD_URL to $DOWNLOAD_DIR"
    curl -fL -o "$FILE" "$DOWNLOAD_URL"
    $UTIL "$FILE"
    rm -f "$FILE"
)

if [[ ! -f "$DOWNLOAD_DIR/$FILENAME/$BIN" ]]; then
    echo "Expected binary not found at $DOWNLOAD_DIR/$FILENAME/$BIN"
    exit 1
fi

if [[ "$DOWNLOAD_DIR" != "$INSTALL_DIR" ]]; then
    mkdir -p "$INSTALL_DIR"
    if [[ -f "$INSTALL_DIR/$BIN" ]]; then
        echo "Moving $INSTALL_DIR/$BIN to $INSTALL_DIR/$BIN.old"
        mv -f "$INSTALL_DIR/$BIN" "$INSTALL_DIR/$BIN.old"
    fi
    mv -f "$DOWNLOAD_DIR/$FILENAME/$BIN" "$INSTALL_DIR/$BIN"
    chmod +x "$INSTALL_DIR/$BIN"
fi

echo "Successfully installed $BIN in $INSTALL_DIR"
