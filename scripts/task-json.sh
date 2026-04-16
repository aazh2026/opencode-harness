#!/bin/bash

source "$(dirname "${BASH_SOURCE[0]}")/constitution.sh" 2>/dev/null || true

DEFAULT_MODEL="minimax-cn/MiniMax-M2.7"
MODEL="${MODEL:-$DEFAULT_MODEL}"

get_next_todo_task() {
    local json_file="$1"
    if [ ! -f "$json_file" ]; then
        echo ""
        return
    fi

    local task_id=$(cat "$json_file" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    tasks = data if isinstance(data, list) else data.get('tasks', [])
    for t in tasks:
        if t.get('status') == 'todo':
            print(t.get('id', ''))
            break
except:
    pass
" 2>/dev/null || echo "")

    echo "$task_id"
}

check_remaining_p0_p1() {
    local json_file="$1"
    if [ ! -f "$json_file" ]; then
        echo "0"
        return
    fi

    local remaining=$(cat "$json_file" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    if isinstance(data, list):
        count = sum(1 for t in data if t.get('priority') in ['P0', 'P1'] and t.get('status') != 'done')
        print(count)
    elif isinstance(data, dict) and 'tasks' in data:
        count = sum(1 for t in data['tasks'] if t.get('priority') in ['P0', 'P1'] and t.get('status') != 'done')
        print(count)
    else:
        print('0')
except:
    print('0')
" 2>/dev/null || echo "0")

    echo "${remaining:-0}"
}

has_todo_tasks_json() {
    local json_file="$1"
    if [ ! -f "$json_file" ]; then
        echo "0"
        return
    fi

    local has_todo=$(cat "$json_file" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    tasks = data if isinstance(data, list) else data.get('tasks', [])
    count = sum(1 for t in tasks if t.get('status') == 'todo')
    print(count)
except:
    print('0')
" 2>/dev/null || echo "0")

    echo "${has_todo:-0}"
}

get_task_details() {
    local json_file="$1"
    local task_id="$2"

    cat "$json_file" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    tasks = data if isinstance(data, list) else data.get('tasks', [])
    for t in tasks:
        if t.get('id') == '$task_id':
            print(json.dumps(t, indent=2))
            break
except:
    pass
" 2>/dev/null || echo "{}"
}

update_task_status() {
    local json_file="$1"
    local task_id="$2"
    local new_status="$3"

    if [ ! -f "$json_file" ]; then
        echo "⚠️  JSON文件不存在: $json_file"
        return 1
    fi

    python3 -c "
import json
with open('$json_file', 'r') as f:
    data = json.load(f)

tasks = data if isinstance(data, list) else data.get('tasks', [])
for t in tasks:
    if t.get('id') == '$task_id':
        t['status'] = '$new_status'
        break

with open('$json_file', 'w') as f:
    json.dump(data, f, indent=2)
print('✅ 任务 $task_id 状态更新为 $new_status')
" 2>/dev/null || echo "⚠️  状态更新失败"
}

count_todo_tasks() {
    local json_file="$1"
    if [ ! -f "$json_file" ]; then
        echo "0"
        return
    fi

    local count=$(cat "$json_file" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    tasks = data if isinstance(data, list) else data.get('tasks', [])
    print(sum(1 for t in tasks if t.get('status') == 'todo'))
except:
    print('0')
" 2>/dev/null || echo "0")

    echo "${count:-0}"
}

count_done_tasks() {
    local json_file="$1"
    if [ ! -f "$json_file" ]; then
        echo "0"
        return
    fi

    local count=$(cat "$json_file" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    tasks = data if isinstance(data, list) else data.get('tasks', [])
    print(sum(1 for t in tasks if t.get('status') == 'done'))
except:
    print('0')
" 2>/dev/null || echo "0")

    echo "${count:-0}"
}

generate_tasks_json_fallback() {
    local task_file="$1"
    local json_file="$2"

    local first_task=1
    local current_priority=""
    local task_id=""
    local task_desc=""
    local task_lines=""

    process_task() {
        if [[ -z "$task_id" || -z "$current_priority" ]]; then
            return
        fi

        local has_todo=0
        local has_done=0

        while IFS= read -r line; do
            if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*\[ ]]; then
                if [[ "$line" =~ ^[[:space:]]*-[[:space:]]*\[[[:space:]]*\] ]]; then
                    has_todo=1
                else
                    has_done=1
                fi
            fi
        done <<< "$task_lines"

        local status="todo"
        if [[ $has_done -eq 1 && $has_todo -eq 0 ]]; then
            status="done"
        fi

        local desc_json=$(echo "$task_desc" | jq -Rs '.')

        if [[ $first_task -eq 0 ]]; then
            echo ","
        fi
        first_task=0

        echo "    {"
        echo "      \"id\": \"$task_id\","
        echo "      \"priority\": \"$current_priority\","
        echo "      \"title\": \"$task_desc\","
        echo "      \"description\": $desc_json,"
        echo "      \"status\": \"$status\","
        echo "      \"test_criteria\": [\"代码编译通过\", \"功能测试通过\"],"
        echo "      \"test_commands\": [\"cargo build\"],"
        echo "      \"impl_notes\": \"\","
        echo "      \"dependencies\": []"
        echo -n "    }"

        task_id=""
        task_desc=""
        task_lines=""
    }

    {
        echo "{"
        echo '  "tasks": ['
        first_task=1

        while IFS= read -r line; do
            if [[ "$line" =~ ^##[[:space:]]*P0 ]]; then
                process_task
                current_priority="P0"
            elif [[ "$line" =~ ^##[[:space:]]*P1 ]]; then
                process_task
                current_priority="P1"
            elif [[ "$line" =~ ^##[[:space:]]*P2 ]]; then
                process_task
                current_priority="P2"
            elif [[ "$line" =~ ^###[[:space:]]*([A-Z][A-Z0-9]*-[0-9][0-9]*):[[:space:]]*(.+) ]]; then
                local _mid="${BASH_REMATCH[1]}" _mdesc="${BASH_REMATCH[2]}"
                process_task
                task_id="$_mid"
                task_desc="$_mdesc"
                task_lines=""
            elif [[ -n "$current_priority" && -n "$task_id" ]]; then
                task_lines="${task_lines}${line}"$'\n'
            fi
        done < "$task_file"

        process_task

        echo ""
        echo "  ]"
        echo "}"
    } > "$json_file"
}

generate_tasks_json() {
    local task_file="$1"
    local json_file="$2"

    if [ -f "$json_file" ] && [ $(wc -c < "$json_file") -gt 10 ]; then
        echo "⏭️  跳过JSON生成（已存在）: $json_file"
        return 0
    fi

    if [ ! -f "$task_file" ]; then
        echo "⚠️  任务文件不存在，无法生成JSON: $task_file"
        return 1
    fi

    echo "📝 生成结构化任务JSON（使用LLM）: $json_file"

    local prompt="基于任务Markdown文件，生成一个结构化的JSON任务文件。

## 重要约束
- 禁止使用 subagent 或 task 工具 spawning 其他 agent
- 必须直接在当前 session 中完成

## 任务Markdown文件
$(cat $task_file)

## 输出要求
将JSON写入到文件：$json_file

JSON格式必须包含以下字段：
{
  \"tasks\": [
    {
      \"id\": \"任务ID（如FR-001）\",
      \"priority\": \"P0|P1|P2\",
      \"title\": \"任务简短标题\",
      \"description\": \"任务详细描述\",
      \"status\": \"todo|done|in_progress\",
      \"test_criteria\": [\"测试标准1\", \"测试标准2\"],
      \"test_commands\": [\"cargo test --package <pkg>\", \"npm run build\"],
      \"impl_notes\": \"实现注意事项\",
      \"dependencies\": [\"依赖的任务ID\"]
    }
  ]
}

## 要求
1. 每个任务必须有清晰可测试的test_criteria，且必须包含自动化测试用例设计
2. test_commands必须是可自动化执行的验证命令，包括：
   - cargo test 指定具体测试文件或函数（如 cargo test -p opencode-core --test session_storage）
   - cargo clippy 进行代码质量检查
   - cargo build --release 确保发布构建成功
3. test_criteria must contain automated test scenarios, format:
   - AddUnitTest: verify XXX functionality works
   - AddIntegrationTest: verify XXX and YYY integrate correctly
   - AddRegressionTest: ensure XXX changes do not break existing functionality
   - AddErrorHandlingTest: test XXX edge cases and error handling
   - RunTestSuite: execute all related tests to ensure no regressions
4. Bug detection capability:
   - each test_criteria should contain catchable assertions/expected results
   - test failures should pinpoint specific files and functions
5. dependencies should reference other task IDs (if any)
6. parse Markdown status markers (- [ ] = todo, - [x] = done)
7. output must be valid JSON in English, write directly to file with no other content"

    run_opencode_with_session_export "$prompt" "$SESSION_EXPORT_DIR/tasks_json_${task_file##*/}.json" "$MODEL"

    if [ -f "$json_file" ] && [ $(wc -c < "$json_file") -gt 10 ]; then
        echo "✅ JSON生成成功"
        return 0
    else
        echo "⚠️  LLM生成JSON失败，尝试脚本解析..."
        generate_tasks_json_fallback "$task_file" "$json_file"
    fi
}