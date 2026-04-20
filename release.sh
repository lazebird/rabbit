#!/bin/bash
#
# Rabbit Release Script
#
# Usage: ./release.sh <version>
# Example: ./release.sh 1.2.0
#
# This script:
# 1. Builds release binaries for current platform
# 2. Calculates SHA256 and file size
# 3. Updates release/versions.json
# 4. Generates release notes template
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# ============================================================
# Configuration
# ============================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_DIR="${SCRIPT_DIR}/release"
VERSIONS_FILE="${RELEASE_DIR}/versions.json"

# ============================================================
# Helper Functions
# ============================================================

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

die() {
    log_error "$1"
    exit 1
}

# ============================================================
# Platform Detection
# ============================================================

detect_platform() {
    local os=$(uname -s)
    local arch=$(uname -m)

    case "${os}_${arch}" in
        Linux_x86_64)
            echo "linux-x64"
            ;;
        Linux_aarch64)
            echo "linux-arm64"
            ;;
        Darwin_x86_64)
            echo "macos-x64"
            ;;
        Darwin_arm64)
            echo "macos-arm64"
            ;;
        *)
            # Windows detection (via MSYS/CYGWIN)
            if [[ "${os}" == *"NT"* ]] || [[ "${OS}" == *"Windows"* ]]; then
                echo "windows-x64"
            else
                die "Unsupported platform: ${os}_${arch}"
            fi
            ;;
    esac
}

get_binary_name() {
    local platform=$1
    case "${platform}" in
        windows-x64)
            echo "rabbit.exe"
            ;;
        *)
            echo "rabbit"
            ;;
    esac
}

get_output_filename() {
    local platform=$1
    local version=$2
    case "${platform}" in
        windows-x64)
            echo "rabbit-${version}-windows-x64.exe"
            ;;
        linux-x64)
            echo "rabbit-${version}-linux-x64"
            ;;
        linux-arm64)
            echo "rabbit-${version}-linux-arm64"
            ;;
        macos-x64)
            echo "rabbit-${version}-macos-x64"
            ;;
        macos-arm64)
            echo "rabbit-${version}-macos-arm64"
            ;;
    esac
}

# ============================================================
# Build Functions
# ============================================================

build_release() {
    local version=$1
    log_info "Building release version ${version}..." >&2

    # Build release binary
    log_info "Running cargo build --release..." >&2
    cd "${SCRIPT_DIR}"
    cargo build --release || die "Build failed"

    local binary_name=$(get_binary_name "${PLATFORM}")
    local binary_path="${SCRIPT_DIR}/target/release/${binary_name}"

    if [ ! -f "${binary_path}" ]; then
        die "Binary not found at ${binary_path}"
    fi

    log_info "Build successful: ${binary_path}" >&2
    echo "${binary_path}"
}

# ============================================================
# Release Functions
# ============================================================

calculate_sha256() {
    local file=$1

    if command -v sha256sum &> /dev/null; then
        sha256sum "$file" | awk '{print $1}'
    elif command -v shasum &> /dev/null; then
        shasum -a 256 "$file" | awk '{print $1}'
    else
        die "No SHA256 tool found (need sha256sum or shasum)"
    fi
}

get_file_size() {
    local file=$1
    stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || die "Cannot get file size"
}

update_versions_json() {
    local version=$1
    local platform=$2
    local filename=$3
    local sha256=$4
    local size=$5

    local download_url="https://raw.githubusercontent.com/lazebird/rabbit/rewrite/release/${filename}"

    log_info "Updating ${VERSIONS_FILE}..."

    if [ -f "${VERSIONS_FILE}" ]; then
        # Update existing file
        log_info "Adding ${platform} entry to existing versions.json"

        # Use python for JSON manipulation (more reliable than jq)
        python3 -c "
import json
import sys

with open('${VERSIONS_FILE}', 'r') as f:
    data = json.load(f)

# Update version info
data['version'] = '${version}'
data['release_date'] = '$(date +%Y/%m/%d)'

# Update or add platform entry
if 'platforms' not in data:
    data['platforms'] = {}

data['platforms']['${platform}'] = {
    'sha256': '${sha256}',
    'size': ${size},
    'url': '${download_url}'
}

with open('${VERSIONS_FILE}', 'w') as f:
    json.dump(data, f, indent=2)
" || die "Failed to update versions.json"
    else
        # Create new file
        log_info "Creating new versions.json"

        cat > "${VERSIONS_FILE}" << EOF
{
  "version": "${version}",
  "release_date": "$(date +%Y/%m/%d)",
  "release_notes": "Release ${version}",
  "platforms": {
    "${platform}": {
      "sha256": "${sha256}",
      "size": ${size},
      "url": "${download_url}"
    }
  }
}
EOF
    fi

    log_info "versions.json updated successfully"
}

generate_release_notes() {
    local version=$1
    local platform=$2
    local notes_file="${RELEASE_DIR}/RELEASE_NOTES_${version}.md"

    cat > "${notes_file}" << EOF
# Rabbit ${version} Release Notes

## Release Date

$(date +%Y-%m-%d)

## Platform

- ${platform}

## Changes

- TODO: Add release notes here

## Download

| Platform | URL |
|----------|-----|
| ${platform} | https://github.com/lazebird/rabbit/raw/rewrite/release/rabbit-${version}-${platform}$( [[ "${platform}" == "windows-x64" ]] && echo ".exe" ) |

## SHA256 Checksums

\`\`\`
TODO: Run release.sh to generate checksums
\`\`\`
EOF

    log_info "Release notes generated: ${notes_file}"
}

# ============================================================
# Main
# ============================================================

main() {
    # Check arguments
    if [ $# -lt 1 ]; then
        echo "Usage: $0 <version> [--notes-only]"
        echo "Example: $0 1.2.0"
        exit 1
    fi

    VERSION="$1"
    NOTES_ONLY=false

    if [ "$2" == "--notes-only" ]; then
        NOTES_ONLY=true
    fi

    log_info "Rabbit Release Script"
    log_info "Version: ${VERSION}"

    # Detect platform
    PLATFORM=$(detect_platform)
    log_info "Platform: ${PLATFORM}"

    # Create release directory if needed
    mkdir -p "${RELEASE_DIR}"

    if [ "${NOTES_ONLY}" = false ]; then
        # Build
        BINARY_PATH=$(build_release "${VERSION}")

        # Get output filename
        OUTPUT_FILENAME=$(get_output_filename "${PLATFORM}" "${VERSION}")

        # Copy binary to release directory
        cp "${BINARY_PATH}" "${RELEASE_DIR}/${OUTPUT_FILENAME}"
        log_info "Binary copied to ${RELEASE_DIR}/${OUTPUT_FILENAME}"

        # Calculate SHA256 and size
        SHA256=$(calculate_sha256 "${RELEASE_DIR}/${OUTPUT_FILENAME}")
        SIZE=$(get_file_size "${RELEASE_DIR}/${OUTPUT_FILENAME}")

        log_info "SHA256: ${SHA256}"
        log_info "Size: ${SIZE} bytes ($(echo "scale=1; ${SIZE}/1024/1024" | bc) MB)"

        # Update versions.json
        update_versions_json "${VERSION}" "${PLATFORM}" "${OUTPUT_FILENAME}" "${SHA256}" "${SIZE}"
    fi

    # Generate release notes
    generate_release_notes "${VERSION}" "${PLATFORM}"

    log_info "Release process complete!"
    log_info "Files in ${RELEASE_DIR}:"
    ls -la "${RELEASE_DIR}"
}

main "$@"
