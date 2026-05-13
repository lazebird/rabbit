# ============================================================
# Release Notes: generate RELEASE_NOTES_<version>.md
# Requires: common.sh (for SCRIPT_DIR, RELEASE_DIR, VERSIONS_FILE,
#           PYTHON, log_*, get_output_filename, calculate_sha256)
# Requires: release-utils.sh (for _get_prev_tag)
# ============================================================

generate_release_notes() {
    local version=$1
    local platform=$2
    local notes_file="${RELEASE_DIR}/RELEASE_NOTES_${version}.md"

    log_info "Generating release notes from git history..."

    # Find the tag BEFORE current version (handles re-runs when tag already exists)
    local prev_tag
    prev_tag=$(_get_prev_tag "$version")

    local changelog
    if [ -n "$prev_tag" ]; then
        changelog=$("${SCRIPT_DIR}/git-changelog.sh" "$version" --from-tag "$prev_tag" 2>/dev/null) || {
            log_warn "Failed to generate changelog from git history"
            changelog="No changelog available"
        }
    else
        changelog=$("${SCRIPT_DIR}/git-changelog.sh" "$version" 2>/dev/null) || {
            log_warn "Failed to generate changelog from git history"
            changelog="No changelog available"
        }
    fi

    # Generate platform/download/SHA sections from versions.json (ALL platforms)
    local platform_section
    local download_section
    local sha_section

    if [ -f "${VERSIONS_FILE}" ]; then
        local _tmp_py
        _tmp_py=$(mktemp)

        cat > "${_tmp_py}" << 'PYEOF'
import json, sys
with open(sys.argv[1]) as f:
    d = json.load(f)
plats = d.get('platforms', {})
sorted_plats = sorted(plats)
print('--DIVIDER--')
for p in sorted_plats:
    print('- ' + p)
print('--DIVIDER--')
print('| Platform | URL |')
print('|----------|-----|')
for p in sorted_plats:
    print(f'| {p} | {plats[p]["url"]} |')
print('--DIVIDER--')
print('```')
for p in sorted_plats:
    suffix = '.exe' if p.startswith('windows') else ''
    print(f'{plats[p]["sha256"]}  rabbit-{d["version"]}-{p}{suffix}')
print('```')
PYEOF

        local raw
        raw=$(${PYTHON} "${_tmp_py}" "${VERSIONS_FILE}" | tr -d '\r')
        rm -f "${_tmp_py}"

        # Split by divider lines into three sections
        local divider_count=0
        while IFS= read -r line; do
            if [ "$line" = "--DIVIDER--" ]; then
                divider_count=$((divider_count + 1))
                continue
            fi
            case "$divider_count" in
                1) platform_section="${platform_section}${line}"$'\n' ;;
                2) download_section="${download_section}${line}"$'\n' ;;
                3) sha_section="${sha_section}${line}"$'\n' ;;
            esac
        done <<< "$raw"
        # Trim trailing newlines
        platform_section="${platform_section%%$'\n'}"
        download_section="${download_section%%$'\n'}"
        sha_section="${sha_section%%$'\n'}"
    else
        local output_filename
        output_filename=$(get_output_filename "${platform}" "${version}")
        local sha256_val
        sha256_val=$(calculate_sha256 "${RELEASE_DIR}/${output_filename}" 2>/dev/null) || sha256_val=""
        local download_url="https://raw.githubusercontent.com/lazebird/rabbit/rewrite/release/${output_filename}"

        platform_section="- ${platform}"
        download_section="| ${platform} | ${download_url} |"
        sha_section="\`\`\`\n${sha256_val}  rabbit-${version}-${platform}\n\`\`\`"
    fi

    # Write release notes
    cat > "${notes_file}" << EOF
# Rabbit ${version} Release Notes

## Release Date

$(date +%Y-%m-%d)

## Platforms

${platform_section}

## Changes

${changelog}

## Download

${download_section}

## SHA256 Checksums

${sha_section}
EOF

    log_info "Release notes generated: ${notes_file}"
}
