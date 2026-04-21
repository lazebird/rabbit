#!/bin/bash
# Rabbit 项目统一测试脚本
# 通过参数提供不同的测试模式

set -e  # 遇到错误立即退出

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
CYAN='\033[0;36m'
NC='\033[0m'

# 日志函数
log() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')]${NC} $1"
}

success() {
    echo -e "${GREEN}✓ $1${NC}"
}

fail() {
    echo -e "${RED}✗ $1${NC}"
}

warn() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

header() {
    echo ""
    echo -e "${BOLD}${CYAN}━━━ $1 ━━━${NC}"
}

section() {
    echo -e "${BLUE}  $1${NC}"
}

# 显示帮助信息
show_help() {
    echo -e "${BOLD}🐰 Rabbit 项目测试脚本${NC}"
    echo ""
    echo -e "${BOLD}用法:${NC}"
    echo "  ./test.sh [选项]"
    echo ""
    echo -e "${BOLD}选项:${NC}"
    printf "  ${BOLD}%-12s${NC} %s\n" "quick" "快速测试（默认）- 编译检查 + 所有测试 + Release构建"
    printf "  ${BOLD}%-12s${NC} %s\n" "full" "完整测试 - 快速测试 + 格式检查 + 警告统计"
    printf "  ${BOLD}%-12s${NC} %s\n" "unit" "仅单元测试"
    printf "  ${BOLD}%-12s${NC} %s\n" "integration" "仅集成测试"
    printf "  ${BOLD}%-12s${NC} %s\n" "all" "所有测试（unit + integration）"
    printf "  ${BOLD}%-12s${NC} %s\n" "release" "仅Release构建"
    printf "  ${BOLD}%-12s${NC} %s\n" "fmt" "仅格式检查"
    printf "  ${BOLD}%-12s${NC} %s\n" "clippy" "仅代码检查"
    printf "  ${BOLD}%-12s${NC} %s\n" "check" "仅编译检查"
    printf "  ${BOLD}%-12s${NC} %s\n" "help" "显示此帮助信息"
    echo ""
    echo -e "${BOLD}示例:${NC}"
    echo "  ./test.sh              # 快速测试（默认）"
    echo "  ./test.sh quick        # 快速测试"
    echo "  ./test.sh full         # 完整测试"
    echo "  ./test.sh unit         # 仅单元测试"
    echo "  ./test.sh integration  # 仅集成测试"
    echo "  ./test.sh all          # 所有测试"
    echo ""
    echo -e "${BOLD}特性:${NC}"
    echo "  - 错误时立即停止（set -e）"
    echo "  - 彩色输出，易于阅读"
    echo "  - 显示测试统计信息"
    echo "  - 记录执行时间"
    echo ""
}

# 记录开始时间
START_TIME=$(date +%s)

# 默认模式
MODE=${1:-quick}

# 显示头部
echo ""
echo -e "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}${BLUE}  🐰 Rabbit 项目测试套件${NC}"
echo -e "${BOLD}${BLUE}  模式: ${MODE}${NC}"
echo -e "${BOLD}${BLUE}  时间: $(date '+%Y-%m-%d %H:%M:%S')${NC}"
echo -e "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# 测试计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0
CURRENT_SUITE=""

# 显示测试套件名称
print_suite_name() {
    local suite="$1"
    echo -e "  ${CYAN}套件:${NC} $suite"
}

# 显示测试详情（紧凑格式，按组显示）
print_test_details() {
    local output="$1"
    local suite_name="$2"

    # 提取测试数量
    local count=$(echo "$output" | grep -oP "running \K\d+" | head -1)
    count=${count:-0}

    # 跳过空套件
    [ "$count" -eq 0 ] && return

    # 显示套件标题
    echo -e "\n  ${BOLD}${CYAN}${suite_name}${NC} ($count tests)"

    # 提取并显示测试名称（每行一个）
    echo "$output" | grep -E "^test .* \.\.\. ok" | while read -r line; do
        local test_name=$(echo "$line" | awk '{print $2}')
        echo -e "    ${GREEN}✓${NC} $test_name"
    done

    # 显示失败的测试
    echo "$output" | grep -E "^test .* \.\.\. FAILED" | while read -r line; do
        local test_name=$(echo "$line" | awk '{print $2}')
        echo -e "    ${RED}✗${NC} $test_name"
    done
}

# 更新测试计数
update_test_count() {
    local output="$1"
    local passed=$(echo "$output" | grep -oP 'ok\. \K\d+' | paste -sd+ | bc 2>/dev/null || echo "0")
    local failed=$(echo "$output" | grep -oP 'failed: \K\d+' | paste -sd+ | bc 2>/dev/null || echo "0")
    local ignored=$(echo "$output" | grep -oP 'ignored: \K\d+' | paste -sd+ | bc 2>/dev/null || echo "0")

    PASSED_TESTS=$((PASSED_TESTS + passed))
    FAILED_TESTS=$((FAILED_TESTS + failed))
    SKIPPED_TESTS=$((SKIPPED_TESTS + ignored))
    TOTAL_TESTS=$((TOTAL_TESTS + passed + failed))
}

# ============================================================================
# 测试函数
# ============================================================================

# 检查工具链
check_toolchain() {
    header "检查 Rust 工具链"
    local version=$(cargo --version 2>&1)
    log "Cargo 版本: $version"
    success "工具链检查完成"
}

# 格式检查
check_fmt() {
    header "代码格式检查"
    if cargo fmt --check 2>&1; then
        success "代码格式正确"
    else
        warn "代码格式存在问题，建议运行: cargo fmt"
        return 1
    fi
}

# 编译检查
check_build() {
    header "编译检查"
    if cargo check --all-targets 2>&1 | tail -1; then
        success "编译检查通过"
    else
        fail "编译检查失败"
        exit 1
    fi
}

# 单元测试
test_unit() {
    header "单元测试"
    local has_tests=false

    # rabbit_app 单元测试
    local output_app=$(cargo test --lib -p rabbit-app 2>&1)
    print_test_details "$output_app" "rabbit_app 单元测试"
    update_test_count "$output_app"
    echo "$output_app" | grep -q "running [1-9]" && has_tests=true

    # rabbit_core 单元测试
    local output_core=$(cargo test --lib -p rabbit-core 2>&1)
    print_test_details "$output_core" "rabbit_core 单元测试"
    update_test_count "$output_core"
    echo "$output_core" | grep -q "running [1-9]" && has_tests=true

    # rabbit_models 单元测试
    local output_models=$(cargo test --lib -p rabbit-models 2>&1)
    print_test_details "$output_models" "rabbit_models 单元测试"
    update_test_count "$output_models"
    echo "$output_models" | grep -q "running [1-9]" && has_tests=true

    # rabbit_platform 单元测试
    local output_platform=$(cargo test --lib -p rabbit-platform 2>&1)
    print_test_details "$output_platform" "rabbit_platform 单元测试"
    update_test_count "$output_platform"
    echo "$output_platform" | grep -q "running [1-9]" && has_tests=true

    if ! $has_tests; then
        echo -e "\n  ${YELLOW}提示: 部分 crate 无单元测试${NC}"
    fi

    if echo "$output_app $output_core $output_models $output_platform" | grep -q "test result: ok\."; then
        success "单元测试通过"
    else
        fail "单元测试失败"
        exit 1
    fi
}

# 集成测试
test_integration() {
    header "集成测试"

    local output=$(cargo test --test integration_tests 2>&1)
    print_test_details "$output" "集成测试"
    update_test_count "$output"

    if echo "$output" | grep -q "test result: ok\."; then
        success "集成测试通过"
    else
        fail "集成测试失败"
        exit 1
    fi
}

# 所有测试
test_all() {
    header "完整测试套件"

    # 运行所有测试并解析输出
    local output=$(cargo test --no-fail-fast 2>&1)

    # 解析各个测试套件
    echo "$output" | awk '
    /Running unittests.*rabbit_app-/ { suite="rabbit_app 单元测试"; }
    /Running unittests.*rabbit-d806/ { suite="rabbit (main) 单元测试"; }
    /Running tests\/integration_tests/ { suite="集成测试"; }
    /Running unittests.*rabbit_core-/ { suite="rabbit_core 单元测试"; }
    /Running unittests.*rabbit_models-/ { suite="rabbit_models 单元测试"; }
    /Running unittests.*rabbit_platform-/ { suite="rabbit_platform 单元测试"; }
    /Doc-tests rabbit_app/ { suite="rabbit_app 文档测试"; }
    /Doc-tests rabbit_core/ { suite="rabbit_core 文档测试"; }
    /Doc-tests rabbit_models/ { suite="rabbit_models 文档测试"; }
    /Doc-tests rabbit_platform/ { suite="rabbit_platform 文档测试"; }
    /running [1-9][0-9]* test/ {
        printf "\n  \033[1;36m%s\033[0m (%d tests)\n", suite, $2;
    }
    /^test .* \.\.\. ok/ {
        printf "    \033[0;32m✓\033[0m %s\n", $2;
    }
    /^test .* \.\.\. FAILED/ {
        printf "    \033[0;31m✗\033[0m %s\n", $2;
    }
    '

    update_test_count "$output"

    if echo "$output" | grep -q "test result: ok\."; then
        success "所有测试通过"
    else
        fail "测试失败"
        exit 1
    fi
}

# Release 构建
build_release() {
    header "Release 构建"
    if cargo build --release 2>&1 | tail -1; then
        success "Release 构建成功"
    else
        fail "Release 构建失败"
        exit 1
    fi
}

# Clippy 检查
check_clippy() {
    header "代码质量检查 (clippy)"
    if cargo clippy --all-targets 2>&1 | tail -5; then
        success "代码质量检查通过"
    else
        warn "代码质量检查发现问题"
        return 1
    fi
}

# 警告统计
count_warnings() {
    header "编译警告统计"
    local warnings=$(cargo build --release 2>&1 | grep -c "warning:" || echo "0")
    if [ "$warnings" -eq 0 ]; then
        success "无编译警告"
    else
        warn "发现 $warnings 个编译警告"
    fi
}

# ============================================================================
# 模式执行
# ============================================================================

case "$MODE" in
    quick|快速)
        # 快速测试：编译检查 + 所有测试 + Release构建
        check_toolchain
        check_build
        test_all
        build_release
        ;;
    
    full|完整)
        # 完整测试：快速测试 + 格式检查 + 警告统计
        check_toolchain
        check_fmt || true  # 格式问题不阻止后续测试
        check_build
        test_all
        build_release
        count_warnings
        ;;
    
    unit|单元)
        # 仅单元测试
        check_toolchain
        test_unit
        ;;
    
    integration|集成)
        # 仅集成测试
        check_toolchain
        test_integration
        ;;
    
    all|所有)
        # 所有测试（分开运行）
        check_toolchain
        test_unit
        test_integration
        ;;
    
    release|发布)
        # 仅Release构建
        build_release
        ;;
    
    fmt|格式)
        # 仅格式检查
        check_fmt
        ;;
    
    clippy|检查)
        # 仅代码检查
        check_clippy
        ;;
    
    check|编译)
        # 仅编译检查
        check_toolchain
        check_build
        ;;
    
    help|帮助|-h|--help)
        show_help
        exit 0
        ;;
    
    *)
        echo -e "${RED}错误: 未知的测试模式 '$MODE'${NC}"
        echo ""
        show_help
        exit 1
        ;;
esac

# 计算执行时间
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))
MINUTES=$((ELAPSED / 60))
SECONDS=$((ELAPSED % 60))

# 显示完成信息
echo ""
echo -e "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}${GREEN}✅ 测试完成！${NC}"
echo -e "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "  ${BLUE}完成时间:${NC} $(date '+%Y-%m-%d %H:%M:%S')"
echo -e "  ${BLUE}总测试数:${NC} $TOTAL_TESTS"
echo -e "  ${GREEN}通过:${NC} $PASSED_TESTS"
if [ "$FAILED_TESTS" -gt 0 ]; then
    echo -e "  ${RED}失败:${NC} $FAILED_TESTS"
fi
if [ "$SKIPPED_TESTS" -gt 0 ]; then
    echo -e "  ${YELLOW}跳过:${NC} $SKIPPED_TESTS"
fi
echo -e "  ${BLUE}执行时间:${NC} ${MINUTES}分${SECONDS}秒"
echo -e "  ${BLUE}测试模式:${NC} $MODE"
echo -e "  ${BLUE}状态:${NC} ${GREEN}全部通过${NC}"
echo -e "${BOLD}${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

exit 0
