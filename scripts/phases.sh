#!/bin/bash

HARNESS_ONLY_GUARDRAILS='## Harness 项目硬约束
- 本仓库只允许开发 opencode-harness 自身能力
- 本仓库禁止实现、补完、替代或承载 opencode-rs 的产品功能
- 如果发现 opencode-rs 与 opencode 不一致，只能记录为 mismatch、contract gap、regression candidate、report item
- 不得把“修复 opencode-rs”或“在本仓库实现 opencode-rs 功能”当作任务目标
- 所有实现必须落在 harness 自身范围内，例如 harness/tasks、harness/fixtures、harness/contracts、harness/runners、harness/comparators、harness/verifiers、harness/reports、governance、workspace 管理
- 测试命令必须在当前仓库根目录执行，除非任务明示需要调用外部二进制做黑盒验证
- 禁止假设存在 opencode-rust/ 子目录；本仓库根目录就是实现目录'

run_phase_gap_analysis() {
    local prd_path="$1"
    local output_dir="$2"
    local constitution="$3"

    local gap_file="$output_dir/gap-analysis.md"

    if check_file_quiet "$gap_file"; then
        echo "⏭️  跳过Gap Analysis（已存在）"
        return 0
    fi

    PROMPT_GAP_ANALYSIS="分析当前实现与PRD的差距，并将完整的差距分析报告写入文件：$gap_file

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有分析工作
- 只使用 Read、Write、Edit、Grep、LSP 等直接工具

$HARNESS_ONLY_GUARDRAILS

## PRD
$(cat $prd_path)

## 任务
1. 读取当前实现目录结构（当前仓库根目录、harness/crates/、docs/、harness/contracts/、harness/tasks/、harness/runners/ 等）
2. 基于上述PRD识别 harness 自身缺失的核心能力
3. 对比实现与PRD的差距
4. 明确哪些只是未来要检测的 opencode-rs mismatch，不能当作本仓库要实现的产品功能

## 分析维度
1. 功能完整性：PRD中描述的 harness 功能是否都已实现？
2. 接口完整性：harness 自身 API/CLI/配置约定是否完整？
3. 数据模型：任务、夹具、契约、报告等实体是否都已建模？
4. 配置管理：PRD中要求的配置项是否都已实现？
5. 测试覆盖：是否有必要的 harness 自测？
6. 项目边界：是否存在把 opencode-rs 产品功能误吸收到 harness 的风险？

## 通用差距识别
- 缺失的 harness 模块
- 不完整的 harness 实现
- 未连接的 harness 模块
- 硬编码/魔法数字
- 错误处理缺失
- 类型定义缺失
- 边界不清导致的职责漂移

## 输出要求
将完整的差距分析报告写入到：$gap_file

报告必须包含：
1. 差距列表（表格格式：差距项 | 严重程度 | 模块 | 修复建议）
2. P0/P1/P2问题分类（必须包含P0阻断性问题）
3. 技术债务清单
4. 实现进度总结
5. 项目边界检查（说明哪些能力属于 harness，哪些只应作为被测对象差异记录）"

    generate_if_missing "$gap_file" "$PROMPT_GAP_ANALYSIS" 5
}

run_phase_constitution() {
    local constitution_path="$1"
    local gap_analysis="$2"
    local output_dir="$3"

    local const_update_file="$output_dir/constitution_updates.md"

    if check_file_quiet "$const_update_file"; then
        echo "⏭️  跳过Constitution检查（已存在）"
        return 0
    fi

    PROMPT_CONSTITUTION="检查Constitution是否需要更新，并将更新建议写入文件：$const_update_file

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有分析工作
- 只使用 Read、Write、Edit、Grep、LSP 等直接工具

$HARNESS_ONLY_GUARDRAILS

## Constitution
$(cat $constitution_path 2>/dev/null || echo "Constitution不存在")

## 差距分析
$(cat $gap_analysis)

## 任务
1. 检查现有Constitution是否覆盖新的P0问题
2. 如需更新，提出Constitution修订建议
3. 确保新的设计决策符合 Harness 的职责边界

## 输出要求
将Constitution更新建议写入到：$const_update_file"

    generate_if_missing "$const_update_file" "$PROMPT_CONSTITUTION" 5
}

run_phase_spec() {
    local prd_path="$1"
    local gap_analysis="$2"
    local output_dir="$3"
    local iteration="$4"

    local spec_file="$output_dir/spec_v${iteration}.md"

    if check_file_quiet "$spec_file"; then
        echo "⏭️  跳过Spec更新（已存在）"
        return 0
    fi

    PROMPT_SPEC="基于PRD和差距分析，更新规格文档，并写入文件：$spec_file

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有分析工作
- 只使用 Read、Write、Edit、Grep、LSP 等直接工具

$HARNESS_ONLY_GUARDRAILS

## PRD
$(cat $prd_path)

## 差距分析
$(cat $gap_analysis)

## 任务
1. 基于差距分析，更新 harness 自身的规格文档
2. 确保新增内容是 harness 的模块、接口、规则或工件，而不是 opencode-rs 功能实现
3. 添加功能需求编号(FR-XXX)

## 输出要求
将更新后的规格文档写入到：$spec_file"

    generate_if_missing "$spec_file" "$PROMPT_SPEC" 5
}

run_phase_plan() {
    local spec_file="$1"
    local gap_analysis="$2"
    local output_dir="$3"
    local iteration="$4"

    local plan_file="$output_dir/plan_v${iteration}.md"
    local tasks_file="$output_dir/tasks_v${iteration}.md"
    local tasks_json="$output_dir/tasks_v${iteration}.json"

    if check_file_quiet "$plan_file" && check_file_quiet "$tasks_file"; then
        echo "⏭️  跳过Plan/Tasks更新（已存在）"
        return 0
    fi

    PROMPT_PLAN="基于Spec更新实现计划和任务清单，并将它们写入文件。

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有分析工作
- 只使用 Read、Write、Edit、Grep、LSP 等直接工具

$HARNESS_ONLY_GUARDRAILS

## Spec
$(cat $spec_file)

## 差距分析
$(cat $gap_analysis)

## 任务
1. 更新 harness 实现计划
2. 更新 harness 任务清单
3. 确保 P0 任务优先
4. 所有任务必须是建设 harness 自身能力，不允许出现“开发 opencode-rs 功能”的任务

## 输出要求
将更新后的计划写入到：$plan_file
将更新后的任务清单写入到：$tasks_file"

    generate_if_missing "$plan_file" "$PROMPT_PLAN" 5
    generate_if_missing "$tasks_file" "$PROMPT_PLAN" 5

    if ! check_file_quiet "$tasks_json"; then
        generate_tasks_json "$tasks_file" "$tasks_json"
    fi
}

run_phase_implementation() {
    local tasks_json="$1"
    local spec_file="$2"
    local output_dir="$3"
    local max_rounds="$4"

    TASK_FILE="$output_dir/tasks_v${NEXT_ITERATION}.md"
    TASKS_JSON="$tasks_json"

    for round in $(seq 1 $max_rounds); do
        echo ""
        echo "=============================================="
        echo "外循环轮次 $round/$max_rounds"
        echo "=============================================="

        if [ ! -f "$TASKS_JSON" ]; then
            if [ -f "$TASK_FILE" ]; then
                generate_tasks_json "$TASK_FILE" "$TASKS_JSON"
            else
                echo "⚠️  任务文件不存在: $TASK_FILE"
                break
            fi
        fi

        remaining_p0_p1=$(check_remaining_p0_p1 "$TASKS_JSON")
        echo "剩余未完成的P0/P1任务: $remaining_p0_p1"

        todo_count=$(count_todo_tasks "$TASKS_JSON")
        done_count=$(count_done_tasks "$TASKS_JSON")
        total_count=$((todo_count + done_count))
        echo "任务进度: $done_count/$total_count 完成"

        if [ "$remaining_p0_p1" -eq 0 ] && [ "$todo_count" -eq 0 ]; then
            echo "所有P0/P1任务已完成!"
            break
        fi

        if [ "$todo_count" -eq 0 ]; then
            echo "没有待办任务了"
            break
        fi

        echo ""
        echo "开始逐个实现待办任务..."

        while true; do
            next_task=$(get_next_todo_task "$TASKS_JSON")
            if [ -z "$next_task" ]; then
                echo "所有待办任务已处理完毕"
                break
            fi

            implement_task "$next_task" "$TASKS_JSON" "$spec_file"

            remaining_p0_p1=$(check_remaining_p0_p1 "$TASKS_JSON")
            if [ "$remaining_p0_p1" -eq 0 ]; then
                echo ""
                echo "🎉 所有P0/P1阻断性问题已解决!"
                break
            fi
        done

        if [ $round -eq $max_rounds ]; then
            echo ""
            echo "达到最大外循环轮次"
            remaining_p0_p1=$(check_remaining_p0_p1 "$TASKS_JSON")
            if [ "$remaining_p0_p1" -gt 0 ]; then
                echo "仍有 $remaining_p0_p1 个P0/P1任务未完成"
            fi
            echo "继续到验证阶段..."
            break
        fi
    done
}

run_phase_verification() {
    local gap_analysis="$1"
    local tasks_md="$2"
    local tasks_json="$3"
    local output_dir="$4"

    local verif_file="$output_dir/verification-report.md"

    if check_file_quiet "$verif_file"; then
        echo "⏭️  跳过验证报告（已存在）"
        return 0
    fi

    PROMPT_VERIFICATION="生成迭代验证报告，并将报告写入文件：$verif_file

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有验证工作
- 只使用 Read、Write、Edit、Grep、LSP、Bash 等直接工具

$HARNESS_ONLY_GUARDRAILS

## 差距分析
$(cat $gap_analysis)

## 任务清单
$(cat $tasks_md)

## 任务JSON
$(cat $tasks_json 2>/dev/null || echo "{}")

## 实现状态
检查当前仓库中的 harness 代码、文档与 git 提交历史

## 输出要求
将完整的迭代验证报告写入到：$verif_file

报告必须包含：
1. P0问题状态（表格：问题 | 状态 | 备注）
2. Harness 职责边界检查
3. PRD完整度评估
4. 遗留问题清单
5. 下一步建议"

    generate_if_missing "$verif_file" "$PROMPT_VERIFICATION" 5
}

implement_task() {
    local task_id="$1"
    local task_json="$2"
    local spec_file="$3"

    echo ""
    echo "----------------------------------------------"
    echo "🎯 实现任务: $task_id"
    echo "----------------------------------------------"

    local task_details=$(get_task_details "$task_json" "$task_id")
    echo "任务详情:"
    echo "$task_details" | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'标题: {d.get(\"title\",\"\")}'); print(f'优先级: {d.get(\"priority\",\"\")}'); print(f'测试标准: {d.get(\"test_criteria\",\"\")}')" 2>/dev/null || echo "$task_details"

    update_task_status "$task_json" "$task_id" "in_progress"

    echo ""
    echo "开始实现..."

    local prompt="实现任务：$task_id

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 禁止将工作委托给其他 agent
- 必须直接在当前 session 中完成所有实现工作
- 只使用 Read、Write、Edit、Grep、LSP、Bash 等直接工具

$HARNESS_ONLY_GUARDRAILS

## 任务信息
$(echo "$task_details" | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'ID: {d.get(\"id\",\"\")}'); print(f'标题: {d.get(\"title\",\"\")}'); print(f'描述: {d.get(\"description\",\"\")}'); print(f'优先级: {d.get(\"priority\",\"\")}'); print(f'测试标准: {chr(10).join(d.get(\"test_criteria\",[]))}'); print(f'测试命令: {chr(10).join(d.get(\"test_commands\",[]))}'); print(f'实现注意事项: {d.get(\"impl_notes\",\"\")}'); print(f'依赖: {d.get(\"dependencies\",[])}')" 2>/dev/null || echo "$task_details")

## Spec
$(cat $spec_file)

## 实现目录
当前仓库根目录及其 harness/crates, docs, harness/tasks, harness/contracts, harness/runners, harness/providers, harness/golden, harness/regression, harness/reports 子目录。禁止假设 ./iterations/src/ 是主实现目录。

## 任务
1. 分析任务需求和测试标准
2. 只实现 harness 自身代码、文档、配置或测试
3. 运行测试命令验证
4. 确保本仓库内相关 build / test 通过
5. 完成后更新任务状态

## 验证
- 必须通过任务定义的当前仓库测试命令
- 如无更细粒度命令，至少验证 cargo build 和 cargo test（若本轮已有 Rust workspace）

## 完成后的操作
1. 更新任务JSON文件中的状态为 done
2. 如果有对应的Markdown任务文件，也需要更新状态为 ✅ Done
3. 提交代码变更"

    run_opencode_with_session_export "$prompt" "$SESSION_EXPORT_DIR/task_${task_id}.json" "$MODEL"

    echo ""
    echo "验证实现..."

    local test_passed=true
    local test_output=""
    local task_details_obj=$(echo "$task_details" | python3 -c "import sys,json; print(json.dumps(json.load(sys.stdin)))" 2>/dev/null)

    if [ -n "$task_details_obj" ]; then
        local test_commands=$(echo "$task_details_obj" | python3 -c "import sys,json; cmds=json.load(sys.stdin).get('test_commands', ['cargo build']); print('\n'.join(cmds) if cmds else 'cargo build')" 2>/dev/null || echo "cargo build")
        echo "运行测试命令..."
        while IFS= read -r cmd; do
            echo "执行: $cmd"
            if ! (cd "$WORKSPACE_DIR" && eval "$cmd" 2>&1); then
                echo "⚠️  测试有问题，请检查"
                test_passed=false
                break
            fi
        done <<< "$test_commands"
        if [ "$test_passed" = true ]; then
            echo "测试通过"
        fi
    fi

    if [ "$test_passed" = false ]; then
        echo ""
        echo "❌ 测试失败，重新生成修复方案..."

        local fix_prompt="任务 $task_id 测试失败，需要修复。

## 测试失败输出
\`\`\`
$test_output
\`\`\`

## 任务信息
$(echo "$task_details" | python3 -c "import sys,json; d=json.load(sys.stdin); print(f'ID: {d.get(\"id\",\"\")}'); print(f'标题: {d.get(\"title\",\"\")}'); print(f'描述: {d.get(\"description\",\"\")}'); print(f'测试命令: {chr(10).join(d.get(\"test_commands\",[]))}')" 2>/dev/null)

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 必须直接在当前 session 中修复所有问题
- 只允许修复 harness 自身代码、配置、测试或文档
- 不允许把问题转化为开发 opencode-rs 功能
- 分析测试失败原因，修复代码，确保所有测试通过

## 任务
1. 分析测试失败原因
2. 修复 harness 自身问题
3. 重新运行测试验证修复
4. 确保当前仓库内相关 build / test 通过

## 完成后的操作
1. 更新任务JSON文件中的状态为 done
2. 如果有对应的Markdown任务文件，也需要更新状态为 ✅ Done
3. 提交代码变更"

        run_opencode_with_session_export "$fix_prompt" "$SESSION_EXPORT_DIR/task_${task_id}_fix.json" "$MODEL"

        echo ""
        echo "验证修复..."

        test_output=$(cd "$WORKSPACE_DIR" && eval "$test_commands" 2>&1) && test_passed=true || test_passed=false

        if [ "$test_passed" = false ]; then
            echo "⚠️  再次测试失败，标记为需手动检查，继续处理下一任务"
            echo "失败输出:"
            echo "$test_output"
            update_task_status "$task_json" "$task_id" "manual_check"
            local task_md_file="${task_json%.json}.md"
            if [ -f "$task_md_file" ]; then
                sed -i '' "s/^### $task_id:.*/### $task_id: ⚠️ Manual Check/" "$task_md_file" 2>/dev/null || true
            fi
            continue
        fi
    fi

    if [ -n "$(git status --porcelain)" ]; then
        echo ""
        echo "提交代码..."
        git add -A
        git commit -m "impl($task_id): $(echo "$task_details" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('title', d.get('id', 'task'))[:50])" 2>/dev/null || echo "task implementation")"
        echo "提交完成"
    fi

    update_task_status "$task_json" "$task_id" "done"

    local task_file="${task_json%.json}.md"
    if [ -f "$task_file" ]; then
        sed -i '' "s/^### $task_id:.*/### $task_id: ✅ Done/" "$task_file" 2>/dev/null || true
    fi

    echo ""
    echo "任务 $task_id 完成"
}
