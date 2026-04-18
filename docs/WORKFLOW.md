# opencode-harness / opencode-rs 增量测试与迭代工作流

## 目标

用 `opencode-harness` 对 `opencode` 与 `opencode-rs` 做真实差分测试，基于测试结果为 `opencode-rs` 生成增量 PRD，再用该 PRD 驱动新的增量迭代，最后回到 Harness 复测，形成闭环。

---

## 标准流程

### 1. 先用 Harness 跑真实测试
不要只看代码或猜测行为，必须直接运行真实 task。

典型入口：

```bash
cd /Users/openclaw/Documents/github/opencode-harness
cargo run -- run --task harness/tasks/cli
cargo run -- run --task harness/tasks/session
cargo run -- run --task harness/tasks/workspace
```

必要时也可以单独跑某个 task：

```bash
cargo run -- run --task harness/tasks/cli/SMOKE-CLI-001.yaml
```

本阶段必须拿到：
- legacy exit code
- rust exit code
- stdout / stderr
- differential verdict

---

### 2. 汇总差异
将 Harness 跑出来的结果按差异类型整理，不要混成一团。

至少区分：

- 输出文本 / 输出流差异
- 退出码语义差异
- 子命令契约差异
- 编译失败 / 实现失败
- 环境问题 / infra 问题

每条差异都应尽量记录：
- task id
- 命令
- legacy 结果
- rust 结果
- verdict
- 简要解释

---

### 3. 基于测试结果生成 `opencode-rs` 的增量 PRD
不要泛泛写“修 CLI”，而是根据真实差异生成增量 PRD。

PRD 文件位置：

```text
/Users/openclaw/Documents/github/opencode-rs/docs/PRD/
```

PRD 内容至少要包含：
- 背景（来源于 Harness 差异报告）
- 本轮目标
- 本轮范围
- 明确不做
- 设计要求
- 必须产物
- 验收标准
- 验收命令
- 下一轮输入

---

### 4. 用新的增量 PRD 启动 `opencode-rs` 新一轮迭代
用生成好的 PRD 驱动下一轮真实实现，而不是停留在报告层。

典型命令：

```bash
cd /Users/openclaw/Documents/github/opencode-rs
./iterate-prd.sh docs/PRD/<新增PRD文件名>.md
```

如果当前已有正在进行中的主线 PRD，需要先判断：
- 是否应等待当前轮完成
- 是否已卡住需要重启
- 是否需要挂到自动续跑链中

---

### 5. 检查任务状态时，必须同时检查最近代码修改时间
这条规则是硬要求。

每次检查任务状态，都必须同时检查：

- 最近 git commit 时间
- 关键源码文件 mtime
- 最新 `tasks_v*.json` 更新时间
- 迭代日志更新时间

不能只看进程是否存在。

#### 判定原则
- 如果只是第一次生成 `gap-analysis.md` 后进入一次重试，不必立刻判故障
- 如果长时间没有代码更新、任务文件不更新、日志无推进，则视为卡住
- 一旦判定卡住，必须自动重启任务，而不是只汇报状态

---

### 6. 任务卡住时自动重启
如果判断任务卡住，应执行：

1. 清理旧的 iterate 残留进程
2. 必要时清理坏掉的 iteration 目录
3. 重新启动当前 PRD 或当前 iteration
4. 重新观察是否有实质推进

注意：
- 不要因为一次正常的首次重试就误判故障
- 但也不能让长时间无更新的任务一直挂着不动

---

### 7. 迭代完成后回到 Harness 复测
`opencode-rs` 完成增量修复后，必须回到 Harness 重新执行相关 task 或 suite。

典型命令：

```bash
cd /Users/openclaw/Documents/github/opencode-harness
cargo run -- run --task harness/tasks/cli
```

目标是验证：
- 差异是否消失
- 差异是否缩小
- 是否引入了新的可观察问题

---

## 职责分工

### `opencode-harness`
负责：
- 真实测试
- 暴露差异
- 形成报告
- 驱动 `opencode-rs` 的增量 PRD 输入

### `opencode-rs`
负责：
- 根据增量 PRD 做真实修复
- 改善 CLI / session / workspace / config 等契约行为
- 接受 Harness 复测

---

## 一句话闭环

> Harness 测试 → 汇总差异 → 为 opencode-rs 生成增量 PRD → 启动增量迭代 → 检查更新时间并在卡住时自动重启 → 回到 Harness 复测。
