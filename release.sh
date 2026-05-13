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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_DIR="${SCRIPT_DIR}/release"
VERSIONS_FILE="${RELEASE_DIR}/versions.json"

# Source module scripts (order matters: dependencies first)
source "${SCRIPT_DIR}/script/common.sh"
source "${SCRIPT_DIR}/script/build.sh"
source "${SCRIPT_DIR}/script/release-utils.sh"
source "${SCRIPT_DIR}/script/generate-notes.sh"

# Bootstrap: detect Python interpreter
PYTHON=$(detect_python) || die "Python (python3 or python) not found or not usable"

# ============================================================
# Git Tag
# ============================================================

create_git_tag() {
    local version=$1
    local tag_name="v${version}"

    if git tag -l | grep -q "^${tag_name}$"; then
        log_warn "Tag ${tag_name} already exists, skipping"
        return 0
    fi

    local message="Release version ${version}"
    git tag -a "$tag_name" -m "$message" \
        && log_info "Created git tag: ${tag_name}" \
        || log_warn "Failed to create git tag"
}

# ============================================================
# Main
# ============================================================

main() {
    if [ $# -lt 1 ]; then
        echo "Usage: $0 <version> [--notes-only]"
        echo "Example: $0 1.2.0"
        exit 1
    fi

    VERSION="$1"
    NOTES_ONLY=false
    [ "$2" = "--notes-only" ] && NOTES_ONLY=true

    log_info "Rabbit Release Script"
    log_info "Version: ${VERSION}"

    PLATFORM=$(detect_platform)
    log_info "Platform: ${PLATFORM}"

    mkdir -p "${RELEASE_DIR}"

    if [ "${NOTES_ONLY}" = false ]; then
        BINARY_PATH=$(build_release "${VERSION}")
        OUTPUT_FILENAME=$(get_output_filename "${PLATFORM}" "${VERSION}")

        cp "${BINARY_PATH}" "${RELEASE_DIR}/${OUTPUT_FILENAME}"
        log_info "Binary copied to ${RELEASE_DIR}/${OUTPUT_FILENAME}"

        SHA256=$(calculate_sha256 "${RELEASE_DIR}/${OUTPUT_FILENAME}")
        SIZE=$(get_file_size "${RELEASE_DIR}/${OUTPUT_FILENAME}")
        log_info "SHA256: ${SHA256}"
        log_info "Size: ${SIZE} bytes ($(( (SIZE + 524288) / 1048576 )) MB)"

        update_versions_json "${VERSION}" "${PLATFORM}" "${OUTPUT_FILENAME}" "${SHA256}" "${SIZE}"
    else
        update_release_notes_only "${VERSION}"
    fi

    generate_release_notes "${VERSION}" "${PLATFORM}"
    create_git_tag "${VERSION}"

    log_info "Release process complete!"
    log_info "Files in ${RELEASE_DIR}:"
    ls -la "${RELEASE_DIR}"
    log_info ""
    log_info "Don't forget to push tags: git push origin rewrite --tags"
}

main "$@"
