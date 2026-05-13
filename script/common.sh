# ============================================================
# Common: shared configuration and utility functions
# Source via: source "$(dirname "$0")/script/common.sh"
# ============================================================

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

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
# Python Detection
# ============================================================

# Detect a working Python interpreter (skip MS Store stubs on Windows)
detect_python() {
    # 1. Standard commands (execution test filters MS Store stubs)
    for cmd in python3 python; do
        if command -v "$cmd" &>/dev/null && "$cmd" --version &>/dev/null 2>&1; then
            echo "$cmd"
            return 0
        fi
    done

    # 2. Python Launcher (Windows)
    if command -v py &>/dev/null && py -3 --version &>/dev/null 2>&1; then
        echo "py -3"
        return 0
    fi

    # 3. Scan PATH for any working Python (handles MS Store stubs shadowing real installs)
    local saved_ifs="$IFS"
    IFS=':'
    for dir in $PATH; do
        [ -z "$dir" ] && continue
        for exe in python3.exe python.exe python3 python; do
            local candidate="${dir}/${exe}"
            if [ -f "$candidate" ] && [ -x "$candidate" ]; then
                if "$candidate" --version &>/dev/null 2>&1; then
                    echo "$candidate"
                    IFS="$saved_ifs"
                    return 0
                fi
            fi
        done
    done
    IFS="$saved_ifs"

    # 4. Check common conda environment directories for a working Python
    for base in "${HOME}/.codegeex/mamba/envs" "${HOME}/miniconda3/envs" "${HOME}/anaconda3/envs"; do
        for env_dir in "$base"/*/; do
            [ -d "$env_dir" ] || continue
            for candidate in "${env_dir}python.exe" "${env_dir}bin/python3"; do
                [ -f "$candidate" ] && [ -x "$candidate" ] && "$candidate" --version &>/dev/null 2>&1 && {
                    echo "$candidate"
                    return 0
                }
            done
        done
    done

    return 1
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
        windows-x64) echo "rabbit.exe" ;;
        *)           echo "rabbit" ;;
    esac
}

get_output_filename() {
    local platform=$1
    local version=$2
    case "${platform}" in
        windows-x64)  echo "rabbit-${version}-windows-x64.exe" ;;
        linux-x64)    echo "rabbit-${version}-linux-x64" ;;
        linux-arm64)  echo "rabbit-${version}-linux-arm64" ;;
        macos-x64)    echo "rabbit-${version}-macos-x64" ;;
        macos-arm64)  echo "rabbit-${version}-macos-arm64" ;;
    esac
}
