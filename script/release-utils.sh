# ============================================================
# Release Utilities: checksums, JSON metadata management
# Requires: common.sh (for SCRIPT_DIR, RELEASE_DIR, VERSIONS_FILE,
#           PYTHON, log_*, die)
# ============================================================

# ============================================================
# Checksum and File Size
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

# ============================================================
# Tag Helpers
# ============================================================

# Get the tag before current version (for changelog range)
_get_prev_tag() {
    local version=$1
    git tag -l --sort=creatordate | awk -v cur="v${version}" '
        $0 == cur { print prev; exit }
        { prev = $0 }
    '
}

# Generate concise release notes summary from git for versions.json
_git_release_notes_summary() {
    local version=$1
    local prev_tag
    prev_tag=$(_get_prev_tag "$version")
    if [ -n "$prev_tag" ]; then
        "${SCRIPT_DIR}/git-changelog.sh" --release-notes "$version" --from-tag "$prev_tag" 2>/dev/null
    else
        "${SCRIPT_DIR}/git-changelog.sh" --release-notes "$version" 2>/dev/null
    fi
}

# ============================================================
# JSON Metadata Management
# ============================================================

update_versions_json() {
    local version=$1
    local platform=$2
    local filename=$3
    local sha256=$4
    local size=$5

    local download_url="https://raw.githubusercontent.com/lazebird/rabbit/rewrite/release/${filename}"

    local release_notes
    release_notes=$(_git_release_notes_summary "$version") || {
        log_warn "Failed to generate release notes from git history"
        release_notes="Release ${version}"
    }

    log_info "Updating ${VERSIONS_FILE}..."

    if [ -f "${VERSIONS_FILE}" ]; then
        log_info "Adding ${platform} entry to existing versions.json"

        local _tmp_py _tmp_notes
        _tmp_py=$(mktemp)
        _tmp_notes=$(mktemp)

        cat > "${_tmp_notes}" <<< "${release_notes}"
        cat > "${_tmp_py}" << 'PYEOF'
import json, sys
vf, nf, ver, dt, plat, sha, sz, url = sys.argv[1:9]
with open(vf) as f:
    d = json.load(f)
with open(nf) as f:
    d['release_notes'] = f.read().strip()
d['version'] = ver
d['release_date'] = dt
d.setdefault('platforms', {})[plat] = {'sha256': sha, 'size': int(sz), 'url': url}
with open(vf, 'w') as f:
    json.dump(d, f, indent=2)
    f.write('\n')
PYEOF

        ${PYTHON} "${_tmp_py}" \
            "${VERSIONS_FILE}" "${_tmp_notes}" \
            "${version}" "$(date +%Y/%m/%d)" \
            "${platform}" "${sha256}" "${size}" "${download_url}" \
            || die "Failed to update versions.json"

        rm -f "${_tmp_py}" "${_tmp_notes}"
    else
        log_info "Creating new versions.json"

        local _tmp_py_new _tmp_notes_new
        _tmp_py_new=$(mktemp)
        _tmp_notes_new=$(mktemp)

        cat > "${_tmp_notes_new}" <<< "${release_notes}"
        cat > "${_tmp_py_new}" << 'PYEOF'
import json, sys
nf, ver, dt, plat, sha, sz, url, out = sys.argv[1:9]
with open(nf) as f:
    notes = f.read().strip()
d = {
    'version': ver, 'release_date': dt, 'release_notes': notes,
    'platforms': {plat: {'sha256': sha, 'size': int(sz), 'url': url}}
}
with open(out, 'w') as f:
    json.dump(d, f, indent=2)
    f.write('\n')
PYEOF

        ${PYTHON} "${_tmp_py_new}" \
            "${_tmp_notes_new}" "${version}" "$(date +%Y/%m/%d)" \
            "${platform}" "${sha256}" "${size}" "${download_url}" "${VERSIONS_FILE}" \
            || die "Failed to create versions.json"

        rm -f "${_tmp_py_new}" "${_tmp_notes_new}"
    fi

    log_info "versions.json updated successfully"
}

update_release_notes_only() {
    local version=$1

    local release_notes
    release_notes=$(_git_release_notes_summary "$version") || {
        log_warn "Failed to generate release notes from git history"
        release_notes="Release ${version}"
    }

    log_info "Updating release_notes in ${VERSIONS_FILE}..."

    if [ -f "${VERSIONS_FILE}" ]; then
        local _tmp_py _tmp_notes
        _tmp_py=$(mktemp)
        _tmp_notes=$(mktemp)

        cat > "${_tmp_notes}" <<< "${release_notes}"
        cat > "${_tmp_py}" << 'PYEOF'
import json, sys
vf, nf = sys.argv[1:3]
with open(vf) as f:
    d = json.load(f)
with open(nf) as f:
    d['release_notes'] = f.read().strip()
with open(vf, 'w') as f:
    json.dump(d, f, indent=2)
    f.write('\n')
PYEOF

        ${PYTHON} "${_tmp_py}" "${VERSIONS_FILE}" "${_tmp_notes}" \
            || die "Failed to update versions.json"

        rm -f "${_tmp_py}" "${_tmp_notes}"
        log_info "release_notes updated in versions.json"
    else
        log_warn "versions.json not found, skipping release_notes update"
    fi
}
