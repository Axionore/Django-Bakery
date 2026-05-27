#!/usr/bin/env sh
# Coolify-style installer for django-bakery.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/Axionore/Django-Barkery/main/installer/install.sh | sh
#   curl -fsSL ... | sh -s -- --version v0.1.0
#   curl -fsSL ... | sh -s -- --prefix ~/.local/bin

set -eu

REPO="Axionore/Django-Barkery"
VERSION="latest"
PREFIX="${INSTALL_PREFIX:-/usr/local/bin}"

while [ "$#" -gt 0 ]; do
    case "$1" in
        --version) VERSION="$2"; shift 2 ;;
        --prefix)  PREFIX="$2";  shift 2 ;;
        -h|--help)
            cat <<EOF
django-bakery installer
  --version <tag>   tag to install (default: latest)
  --prefix <dir>    install dir (default: /usr/local/bin)
EOF
            exit 0
            ;;
        *) printf 'Unknown flag: %s\n' "$1" >&2; exit 64 ;;
    esac
done

uname_s=$(uname -s | tr '[:upper:]' '[:lower:]')
uname_m=$(uname -m)

case "$uname_s" in
    linux)
        os="linux-musl"
        case "$uname_m" in
            x86_64|amd64) arch="x86_64" ;;
            aarch64|arm64) arch="aarch64" ;;
            *) printf 'Unsupported arch: %s\n' "$uname_m" >&2; exit 1 ;;
        esac
        ;;
    darwin)
        os="apple-darwin"
        case "$uname_m" in
            x86_64) arch="x86_64" ;;
            arm64)  arch="aarch64" ;;
            *) printf 'Unsupported arch: %s\n' "$uname_m" >&2; exit 1 ;;
        esac
        ;;
    *) printf 'Unsupported OS: %s\n' "$uname_s" >&2; exit 1 ;;
esac

target="${arch}-unknown-${os}"
[ "$uname_s" = "darwin" ] && target="${arch}-${os}"

if [ "$VERSION" = "latest" ]; then
    api_url="https://api.github.com/repos/${REPO}/releases/latest"
    VERSION=$(
        curl -fsSL "$api_url" \
        | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' \
        | head -n1
    )
fi
if [ -z "$VERSION" ]; then
    printf 'Could not determine latest version.\n' >&2
    exit 1
fi

archive="django-bakery-${VERSION}-${target}.tar.gz"
url="https://github.com/${REPO}/releases/download/${VERSION}/${archive}"
checksums_url="https://github.com/${REPO}/releases/download/${VERSION}/SHA256SUMS"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

printf '↻  Downloading %s\n' "$url"
curl -fsSL -o "$tmp/$archive"     "$url"
curl -fsSL -o "$tmp/SHA256SUMS"   "$checksums_url" || printf '⚠  No SHA256SUMS file — skipping verification.\n'

if [ -f "$tmp/SHA256SUMS" ]; then
    expected=$(grep "  ${archive}$" "$tmp/SHA256SUMS" | awk '{print $1}')
    if [ -n "$expected" ]; then
        actual=$(sha256sum "$tmp/$archive" 2>/dev/null | awk '{print $1}' || shasum -a 256 "$tmp/$archive" | awk '{print $1}')
        if [ "$expected" != "$actual" ]; then
            printf '✘  SHA-256 mismatch:\n  expected %s\n  got      %s\n' "$expected" "$actual" >&2
            exit 1
        fi
        printf '✔  SHA-256 verified.\n'
    fi
fi

tar -xzf "$tmp/$archive" -C "$tmp"
binary="$tmp/django-bakery-${VERSION}-${target}"
chmod +x "$binary"

if [ ! -w "$PREFIX" ]; then
    if command -v sudo >/dev/null 2>&1; then
        sudo install -m 755 "$binary" "$PREFIX/django-bakery"
    else
        printf '✘  %s is not writable and sudo is unavailable. Re-run with --prefix ~/.local/bin\n' "$PREFIX" >&2
        exit 1
    fi
else
    install -m 755 "$binary" "$PREFIX/django-bakery"
fi

printf '\n🥧  django-bakery installed to %s/django-bakery\n' "$PREFIX"
"$PREFIX/django-bakery" --version
printf 'Run `django-bakery new` to bake a Django project.\n'
