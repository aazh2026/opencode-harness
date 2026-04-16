#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/scripts/task-json.sh"
source "$SCRIPT_DIR/scripts/opencode-wrapper.sh"
source "$SCRIPT_DIR/scripts/phases.sh"

parse_args() {
    RESUME_ITERATION=""
    MODEL=""
    MAX_IMPLEMENTATION_ROUNDS=10
    PRD_INPUT=""
    LOG_FILE=""
    VERBOSE="false"

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --resume)
                RESUME_ITERATION="$2"
                shift 2
                ;;
            --model|-m)
                MODEL="$2"
                shift 2
                ;;
            --rounds|-r)
                MAX_IMPLEMENTATION_ROUNDS="$2"
                shift 2
                ;;
            --prd|-p)
                PRD_INPUT="$2"
                shift 2
                ;;
            --log)
                LOG_FILE="$2"
                shift 2
                ;;
            --verbose|-v)
                VERBOSE="true"
                shift
                ;;
            *)
                shift
                ;;
        esac
    done

    MODEL="${MODEL:-minimax-cn/MiniMax-M2.7}"
}

WORKSPACE_DIR="$(cd "$(dirname "$0")" && pwd)"

parse_args "$@"

# Logging setup
SESSION_LOG_DIR="$WORKSPACE_DIR/sessions"

mkdir -p "$SESSION_LOG_DIR"

# Logging function with timestamp
log() {
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local message="[$timestamp] $1"
    
    if [ "$VERBOSE" = "true" ]; then
        echo "$message"
    fi
    
    if [ -n "$LOG_FILE" ]; then
        echo "$message" >> "$LOG_FILE"
    fi
}

# Console output control - suppress if not verbose
log_echo() {
    if [ "$VERBOSE" = "true" ]; then
        echo "$1"
    else
        echo "$1" >> /dev/null
    fi
}

log "=============================================="
log "SpecKit 迭代开发 v3.0 (重构版)"
log "=============================================="
log "工作目录: $WORKSPACE_DIR"
log "日志文件: $LOG_FILE"
if [ -n "$PRD_INPUT" ]; then
    if [ -d "$PRD_INPUT" ]; then
        mapfile -t prd_files < <(find "$PRD_INPUT" -maxdepth 1 -name "*.md" | sort)
        if [ ${#prd_files[@]} -eq 0 ]; then
            echo "❌ 文件夹中未找到.md文件: $PRD_INPUT"
            exit 1
        fi
        OUTPUTS_DIR="${OUTPUTS_DIR:-$WORKSPACE_DIR/iterations}"
        PRD_PATH="$OUTPUTS_DIR/_prd_combined.md"
        cat "${prd_files[@]}" > "$PRD_PATH"
        echo "📂 使用PRD文件夹: $PRD_INPUT (合并${#prd_files[@]}个文件)"
    elif [ -f "$PRD_INPUT" ]; then
        PRD_PATH="$PRD_INPUT"
        echo "📄 使用PRD文件: $PRD_PATH"
    else
        echo "❌ PRD路径不存在: $PRD_INPUT"
        exit 1
    fi
else
    PRD_PATH="$WORKSPACE_DIR/PRD.md"
fi

if [ -n "$RESUME_ITERATION" ]; then
    NEXT_ITERATION="$RESUME_ITERATION"
    OUTPUTS_DIR="$WORKSPACE_DIR/iterations/iteration-${NEXT_ITERATION}"
    if [ ! -d "$OUTPUTS_DIR" ]; then
        echo "❌ 指定迭代不存在: $OUTPUTS_DIR"
        exit 1
    fi
    echo "📦 恢复迭代 #${NEXT_ITERATION}"
else
    LAST_ITERATION=$(ls -d "$WORKSPACE_DIR/iterations/iteration-"* 2>/dev/null | sed 's/.*iteration-//' | sort -n | tail -1)
    NEXT_ITERATION=${LAST_ITERATION:-0}
    NEXT_ITERATION=$((NEXT_ITERATION + 1))
    OUTPUTS_DIR="$WORKSPACE_DIR/iterations/iteration-${NEXT_ITERATION}"
    mkdir -p "$OUTPUTS_DIR"
fi

SESSION_EXPORT_DIR="$SESSION_LOG_DIR/iteration-${NEXT_ITERATION}"
mkdir -p "$SESSION_EXPORT_DIR"

if [ -z "$LOG_FILE" ]; then
    LOG_FILE="$SESSION_LOG_DIR/iteration-${NEXT_ITERATION}_$(date +%Y%m%d_%H%M%S).log"
fi

log "迭代目录: $OUTPUTS_DIR"
log "模型: $MODEL"
log "最大外循环轮次: $MAX_IMPLEMENTATION_ROUNDS"
log ""

log "[1/6] 执行PRD差距分析..."
save_checkpoint "$NEXT_ITERATION" "phase1"
run_phase_gap_analysis "$PRD_PATH" "$OUTPUTS_DIR"

# Constitution phase skipped by user request
# log ""
# log "[2/6] Constitution 检查..."
# save_checkpoint "$NEXT_ITERATION" "phase2"
# run_phase_constitution "$CONSTITUTION_PATH" "$OUTPUTS_DIR/gap-analysis.md" "$OUTPUTS_DIR"

log ""
log "[3/6] 更新Spec..."
save_checkpoint "$NEXT_ITERATION" "phase3"
run_phase_spec "$PRD_PATH" "$OUTPUTS_DIR/gap-analysis.md" "$OUTPUTS_DIR" "$NEXT_ITERATION"

log ""
log "[4/6] 更新Plan和Tasks..."
save_checkpoint "$NEXT_ITERATION" "phase4"
run_phase_plan "$OUTPUTS_DIR/spec_v${NEXT_ITERATION}.md" "$OUTPUTS_DIR/gap-analysis.md" "$OUTPUTS_DIR" "$NEXT_ITERATION"

TASKS_JSON="$OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.json"

log ""
log "[5/6] Per-Task 实现循环..."
save_checkpoint "$NEXT_ITERATION" "phase5"
run_phase_implementation "$TASKS_JSON" "$OUTPUTS_DIR/spec_v${NEXT_ITERATION}.md" "$OUTPUTS_DIR" "$MAX_IMPLEMENTATION_ROUNDS"

log ""
log "[6/6] 验证报告..."
save_checkpoint "$NEXT_ITERATION" "phase6"
run_phase_verification "$OUTPUTS_DIR/gap-analysis.md" "$OUTPUTS_DIR/tasks_v${NEXT_ITERATION}.md" "$TASKS_JSON" "$OUTPUTS_DIR"

log ""
log "=============================================="
log "SpecKit 迭代完成!"
log "=============================================="
log "迭代目录: $OUTPUTS_DIR"
log "任务文件: $TASKS_JSON"
log "验证报告: $OUTPUTS_DIR/verification-report.md"

log "日志保存于: $LOG_FILE"