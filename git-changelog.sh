#!/bin/bash
#
# Git Changelog Generator
#
# Generates changelog entries from git history between two versions/tags.
# This script is independent of business logic and can be reused across projects.
#
# Usage:
#   ./git-changelog.sh <new_version> [from_version]
#
# Examples:
#   ./git-changelog.sh 0.2.0              # Generate changelog from last tag to v0.2.0
#   ./git-changelog.sh 0.2.0 0.1.0       # Generate changelog between specific versions
#   ./git-changelog.sh 0.2.0 --from-tag v0.1.0
#

set -e

# ============================================================
# Configuration
# ============================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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
# Git Operations
# ============================================================

# Get the most recent tag before a given commit/branch
get_last_tag() {
    git describe --tags --abbrev=0 HEAD 2>/dev/null || echo ""
}

# Get all tags sorted by date
get_all_tags() {
    git tag -l --sort=-creatordate
}

# Get commit hash for a tag
get_tag_commit() {
    local tag=$1
    git rev-list -n 1 "$tag" 2>/dev/null || echo ""
}

# Get commits between two refs
get_commits_between() {
    local from=$1
    local to=$2
    
    if [ -z "$from" ]; then
        # Get all commits up to 'to'
        git log "$to" --oneline --no-merges
    else
        git log "${from}..${to}" --oneline --no-merges
    fi
}

# Get commits in conventional commit format with details
get_detailed_commits() {
    local from=$1
    local to=$2
    
    if [ -z "$from" ]; then
        git log "$to" --no-merges --pretty=format:"%h %s"
    else
        git log "${from}..${to}" --no-merges --pretty=format:"%h %s"
    fi
}

# ============================================================
# Changelog Generation
# ============================================================

# Categorize commits by type (feat, fix, docs, refactor, etc.)
categorize_commits() {
    local commits="$1"
    
    local features=""
    local fixes=""
    local docs=""
    local refactors=""
    local perf=""
    local chore=""
    local other=""
    
    while IFS= read -r line; do
        [ -z "$line" ] && continue
        
        local hash=$(echo "$line" | cut -d' ' -f1)
        local msg=$(echo "$line" | cut -d' ' -f2-)
        
        # Extract type from conventional commit format
        if [[ "$msg" =~ ^feat(\(.+\))?:\ (.+)$ ]]; then
            features="${features}- ${BASH_REMATCH[2]} (\`${hash}\`)\n"
        elif [[ "$msg" =~ ^fix(\(.+\))?:\ (.+)$ ]]; then
            fixes="${fixes}- ${BASH_REMATCH[2]} (\`${hash}\`)\n"
        elif [[ "$msg" =~ ^docs(\(.+\))?:\ (.+)$ ]]; then
            docs="${docs}- ${BASH_REMATCH[2]} (\`${hash}\`)\n"
        elif [[ "$msg" =~ ^refactor(\(.+\))?:\ (.+)$ ]]; then
            refactors="${refactors}- ${BASH_REMATCH[2]} (\`${hash}\`)\n"
        elif [[ "$msg" =~ ^perf(\(.+\))?:\ (.+)$ ]]; then
            perf="${perf}- ${BASH_REMATCH[2]} (\`${hash}\`)\n"
        elif [[ "$msg" =~ ^chore(\(.+\))?:\ (.+)$ ]]; then
            chore="${chore}- ${BASH_REMATCH[2]} (\`${hash}\`)\n"
        elif [[ "$msg" =~ ^style(\(.+\))?:\ (.+)$ ]]; then
            other="${other}- ${BASH_REMATCH[2]} (\`${hash}\`)\n"
        elif [[ "$msg" =~ ^test(\(.+\))?:\ (.+)$ ]]; then
            other="${other}- ${BASH_REMATCH[2]} (\`${hash}\`)\n"
        elif [[ "$msg" =~ ^ci(\(.+\))?:\ (.+)$ ]]; then
            other="${other}- ${BASH_REMATCH[2]} (\`${hash}\`)\n"
        else
            other="${other}- ${msg} (\`${hash}\`)\n"
        fi
    done <<< "$commits"
    
    # Output categorized sections
    echo "FEATURES_START"
    echo -e "$features"
    echo "FEATURES_END"
    echo "FIXES_START"
    echo -e "$fixes"
    echo "FIXES_END"
    echo "DOCS_START"
    echo -e "$docs"
    echo "DOCS_END"
    echo "REFACTOR_START"
    echo -e "$refactors"
    echo "REFACTOR_END"
    echo "PERF_START"
    echo -e "$perf"
    echo "PERF_END"
    echo "CHORE_START"
    echo -e "$chore"
    echo "CHORE_END"
    echo "OTHER_START"
    echo -e "$other"
    echo "OTHER_END"
}

# Generate concise release notes for versions.json
# This extracts only meaningful commits (feat, fix, perf, refactor) without full markdown formatting
generate_release_notes_txt() {
    local version=$1
    local from_ref=$2
    local to_ref=$3

    # Get commits
    local commits
    if [ -z "$from_ref" ]; then
        commits=$(get_detailed_commits "" "$to_ref")
    else
        commits=$(get_detailed_commits "$from_ref" "$to_ref")
    fi

    # Extract only meaningful commits (feat, fix, perf, refactor)
    local meaningful=()
    while IFS= read -r line; do
        [ -z "$line" ] && continue

        local hash=$(echo "$line" | cut -d' ' -f1)
        local msg=$(echo "$line" | cut -d' ' -f2-)

        # Only include feat, fix, perf, refactor - skip docs, chore, test, ci, style
        if [[ "$msg" =~ ^feat(\(.+\))?:\ (.+)$ ]]; then
            meaningful+=("${BASH_REMATCH[2]}")
        elif [[ "$msg" =~ ^fix(\(.+\))?:\ (.+)$ ]]; then
            meaningful+=("${BASH_REMATCH[2]}")
        elif [[ "$msg" =~ ^perf(\(.+\))?:\ (.+)$ ]]; then
            meaningful+=("${BASH_REMATCH[2]}")
        elif [[ "$msg" =~ ^refactor(\(.+\))?:\ (.+)$ ]]; then
            meaningful+=("${BASH_REMATCH[2]}")
        fi
    done <<< "$commits"

    # Output as a single string with bullet points, or empty if no meaningful commits
    if [ ${#meaningful[@]} -gt 0 ]; then
        local result=""
        for item in "${meaningful[@]}"; do
            if [ -z "$result" ]; then
                result="- ${item}"
            else
                result="${result}\n- ${item}"
            fi
        done
        echo -e "$result"
    else
        echo "Minor updates and bug fixes."
    fi
}

# Generate markdown changelog
generate_changelog_md() {
    local version=$1
    local from_ref=$2
    local to_ref=$3
    local date=$(date +%Y/%m/%d)

    # Get commits
    local commits
    if [ -z "$from_ref" ]; then
        commits=$(get_detailed_commits "" "$to_ref")
        from_label="beginning"
    else
        commits=$(get_detailed_commits "$from_ref" "$to_ref")
        from_label="$from_ref"
    fi
    
    # Categorize
    local categorized
    categorized=$(categorize_commits "$commits")
    
    # Extract sections
    local features=$(echo "$categorized" | sed -n '/FEATURES_START/,/FEATURES_END/p' | grep -v 'FEATURES_')
    local fixes=$(echo "$categorized" | sed -n '/FIXES_START/,/FIXES_END/p' | grep -v 'FIXES_')
    local docs=$(echo "$categorized" | sed -n '/DOCS_START/,/DOCS_END/p' | grep -v 'DOCS_')
    local refactors=$(echo "$categorized" | sed -n '/REFACTOR_START/,/REFACTOR_END/p' | grep -v 'REFACTOR_')
    local perf=$(echo "$categorized" | sed -n '/PERF_START/,/PERF_END/p' | grep -v 'PERF_')
    local chore=$(echo "$categorized" | sed -n '/CHORE_START/,/CHORE_END/p' | grep -v 'CHORE_')
    local other=$(echo "$categorized" | sed -n '/OTHER_START/,/OTHER_END/p' | grep -v 'OTHER_')
    
    # Generate markdown
    echo "## Version ${version} (${date})"
    echo ""
    
    local has_content=false
    
    if [ -n "$features" ] && [ "$features" != " " ]; then
        echo "### Features"
        echo ""
        echo -e "$features" | grep -v '^$'
        echo ""
        has_content=true
    fi
    
    if [ -n "$fixes" ] && [ "$fixes" != " " ]; then
        echo "### Bug Fixes"
        echo ""
        echo -e "$fixes" | grep -v '^$'
        echo ""
        has_content=true
    fi
    
    if [ -n "$docs" ] && [ "$docs" != " " ]; then
        echo "### Documentation"
        echo ""
        echo -e "$docs" | grep -v '^$'
        echo ""
        has_content=true
    fi
    
    if [ -n "$refactors" ] && [ "$refactors" != " " ]; then
        echo "### Refactoring"
        echo ""
        echo -e "$refactors" | grep -v '^$'
        echo ""
        has_content=true
    fi
    
    if [ -n "$perf" ] && [ "$perf" != " " ]; then
        echo "### Performance"
        echo ""
        echo -e "$perf" | grep -v '^$'
        echo ""
        has_content=true
    fi
    
    if [ -n "$other" ] && [ "$other" != " " ]; then
        echo "### Other Changes"
        echo ""
        echo -e "$other" | grep -v '^$'
        echo ""
        has_content=true
    fi
    
    if [ "$has_content" = false ]; then
        echo "No significant changes recorded."
        echo ""
    fi
    
    # Add comparison link
    if [ -n "$from_ref" ]; then
        echo "[Compare changes](${REPO_URL}/compare/${from_ref}...v${version})"
    fi
    echo ""
}

# ============================================================
# Tag Management
# ============================================================

# Create a new git tag
create_tag() {
    local version=$1
    local message=$2
    
    local tag_name="v${version}"
    
    # Check if tag already exists
    if git tag -l | grep -q "^${tag_name}$"; then
        log_warn "Tag ${tag_name} already exists"
        return 1
    fi
    
    git tag -a "$tag_name" -m "$message"
    log_info "Created tag: ${tag_name}"
    return 0
}

# ============================================================
# Main
# ============================================================

# Default repository URL (override as needed)
REPO_URL="https://github.com/lazebird/rabbit"

main() {
    if [ $# -lt 1 ]; then
        echo "Usage: $0 <version> [from_version]"
        echo "       $0 <version> --from-tag <tag>"
        echo "       $0 --list-tags"
        echo "       $0 --release-notes <version> [from_version]  # Generate concise release notes"
        echo ""
        echo "Examples:"
        echo "  $0 0.2.0                    # From last tag to v0.2.0"
        echo "  $0 0.2.0 0.1.0             # From v0.1.0 to v0.2.0"
        echo "  $0 0.2.0 --from-tag v0.1.0 # From specific tag"
        echo "  $0 --list-tags             # List all tags"
        echo "  $0 --release-notes 0.2.0   # Generate concise release notes for versions.json"
        exit 1
    fi
    
    # List tags mode
    if [ "$1" == "--list-tags" ]; then
        log_info "Available tags:"
        get_all_tags
        exit 0
    fi
    
    # Release notes mode - generate concise notes for versions.json
    if [ "$1" == "--release-notes" ]; then
        shift
        if [ $# -lt 1 ]; then
            log_error "Version required for --release-notes" >&2
            exit 1
        fi
        
        local version=$1
        local from_ref=""
        local to_ref="HEAD"
        
        if [ "$2" == "--from-tag" ]; then
            from_ref="$3"
        elif [ -n "$2" ]; then
            # Check if it's a tag (starts with v) or a commit hash
            if [[ "$2" =~ ^v[0-9] ]]; then
                from_ref="$2"
            elif [[ "$2" =~ ^[0-9] ]]; then
                from_ref="v$2"
            else
                # Assume it's a commit hash
                from_ref="$2"
            fi
        else
            from_ref=$(get_last_tag)
            if [ -n "$from_ref" ]; then
                log_info "Auto-detected last tag: ${from_ref}" >&2
            else
                log_warn "No previous tag found, generating from beginning" >&2
            fi
        fi
        
        generate_release_notes_txt "$version" "$from_ref" "$to_ref"
        exit 0
    fi
    
    local version=$1
    local from_version=""
    local from_ref=""
    local to_ref="HEAD"
    
    # Parse from_version argument
    if [ "$2" == "--from-tag" ]; then
        from_ref="$3"
    elif [ -n "$2" ]; then
        from_version="$2"
        from_ref="v${from_version}"
    else
        # Auto-detect last tag
        from_ref=$(get_last_tag)
        if [ -n "$from_ref" ]; then
            log_info "Auto-detected last tag: ${from_ref}" >&2
        else
            log_warn "No previous tag found, generating from beginning" >&2
        fi
    fi
    
    # Generate changelog
    generate_changelog_md "$version" "$from_ref" "$to_ref"
}

main "$@"
