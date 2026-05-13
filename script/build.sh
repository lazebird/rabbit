# ============================================================
# Build: cargo build and version bumping
# Requires: common.sh (for SCRIPT_DIR, PLATFORM, log_info, die)
# ============================================================

update_cargo_version() {
    local version=$1
    log_info "Updating Cargo.toml version to ${version}..." >&2
    sed -i "s/^version = \".*\"/version = \"${version}\"/" "${SCRIPT_DIR}/Cargo.toml"
}

build_release() {
    local version=$1
    log_info "Building release version ${version}..." >&2

    update_cargo_version "${version}"

    log_info "Running cargo build --release..." >&2
    (cd "${SCRIPT_DIR}" && cargo build --release) || die "Build failed"

    local binary_name
    binary_name=$(get_binary_name "${PLATFORM}")
    local binary_path="${SCRIPT_DIR}/target/release/${binary_name}"

    if [ ! -f "${binary_path}" ]; then
        die "Binary not found at ${binary_path}"
    fi

    log_info "Build successful: ${binary_path}" >&2
    echo "${binary_path}"
}
