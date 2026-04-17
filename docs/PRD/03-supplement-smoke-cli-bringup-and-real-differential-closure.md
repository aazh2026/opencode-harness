# opencode-harness 增量 PRD：Smoke CLI 跑通与真实差分闭环

## 1. 背景

基于 2026-04-18 的真实试跑，`opencode-harness` 已从“框架自身跑不通”推进到“能够真实暴露被测系统问题”的阶段，但最小 smoke CLI 链路尚未形成稳定、可持续、可复用的闭环。

当前已确认事实：

- `SMOKE-CLI-001` 已可同时驱动 `opencode` 与 `opencode-rs`
- Harness 已可稳定拿到双边 exit code
- Harness 已可生成真实 differential verdict
- `SMOKE-CLI-001` 当前暴露的是 help 文本输出流差异：
  - `opencode` 主要将帮助文本输出到 `stderr`
  - `opencode-rs` 主要将帮助文本输出到 `stdout`
- `SMOKE-CLI-002` / `003` / `004` 当前暴露的不是 Harness 假问题，而是 `opencode-rs` 在 `cargo run -q -p opencode-cli -- ...` 路径下存在真实编译失败，集中出现在 `crates/tui/src/app.rs`

这说明：

> Harness 最小真实执行链路已经被打通，但最小 smoke CLI 流程尚未达到“可稳定用于迭代反馈与差异发现”的标准。

本增量 PRD 的职责，是把这条最小闭环真正做实。

---

## 2. 本轮目标

将 `opencode-harness` 提升到以下最低可用状态：

1. 最小 CLI smoke 差分链路可连续执行
2. Harness 能区分“框架问题”、“环境问题”、“实现问题”
3. Harness 对 help / version / invalid-arg 类基础 CLI case 的 verdict 更可信
4. Harness 能把 `opencode-rs` 当前真实编译失败明确沉淀为可复现、可报告、可回归的问题资产

---

## 3. 本轮范围

### 必做范围

#### 3.1 最小 smoke CLI 闭环固化
至少保证以下任务形成稳定执行闭环：

- `SMOKE-CLI-001` `--help`
- `SMOKE-CLI-002` `--version`
- `SMOKE-CLI-003` `workspace --help`
- `SMOKE-CLI-004` `--invalid-option`

#### 3.2 Runner 输入与工作目录约定固化
明确并统一：

- `/project` 到真实 fixture 路径的映射规则
- `fixture_project` 的根路径约定
- `opencode` 与 `opencode-rs` 的 provider-specific binary 选择规则
- `opencode-rs` 不在 PATH 时的 fallback 执行策略

#### 3.3 差异判定可信度提升
至少补齐：

- help 类输出的 `stdout/stderr` 分流归一化策略
- 对编译失败 / 启动失败的 failure classification 改进
- verdict 中对“实现失败”与“输出差异”的分层表达

#### 3.4 真实失败资产化
必须把当前已发现的真实问题沉淀为：

- 明确的 smoke 回归任务结果
- 可读 report
- 问题摘要文档或 regression 记录

---

## 4. 明确不做

本轮不做：

- 不修复 `opencode-rs` 产品代码本身
- 不在 Harness 仓库内替 `opencode-rs` 打补丁
- 不扩展到 web / API / ACP / desktop
- 不追求通用平台抽象
- 不引入 LLM 裁判机制
- 不为所有 CLI case 一次性做复杂 normalization DSL

---

## 5. 设计要求

### 5.1 先把最小链路打硬，再扩范围
不要继续堆抽象。先把 `SMOKE-CLI-001..004` 做成可靠闭环，再考虑更大覆盖。

### 5.2 编译失败必须被视为一等结果
当 `opencode-rs` 因编译失败无法完成 CLI 命令时，Harness 必须：

- 明确记录 rust side 为实现失败
- 保留 stderr / 退出码 / 触发命令
- 不把该结果误判为环境缺失

### 5.3 help 类比较要以“可观察语义”优先
对于 `--help` / `subcommand --help`，至少允许在一定范围内将 `stdout + stderr` 合并后再比较，避免仅因输出流差异导致误报。

### 5.4 调试信息必须够用
报告与 artifact 至少要能回答：

- 实际执行了什么命令
- 在什么 cwd 执行
- 双边 exit code 是多少
- stdout/stderr 分别多长
- 若 rust 侧失败，是编译失败还是运行失败

---

## 6. 必须产物

本轮结束后，仓库内至少应新增或完善：

1. 一个增量 PRD（即本文档）
2. 与 smoke CLI 闭环相关的实现改动
3. 更新后的 differential / report / normalization 逻辑
4. 至少一份总结当前 smoke CLI 真实问题的报告或 regression 资产
5. 能证明 `SMOKE-CLI-001..004` 已被重新执行的输出工件

---

## 7. 验收标准

### 7.1 最低验收
满足以下全部条件才算本轮完成：

1. `SMOKE-CLI-001` 不再因为 runner/cwd/路径问题失败
2. `SMOKE-CLI-001` 的 verdict 能体现 help 文本流差异的真实含义，而非框架误判
3. `SMOKE-CLI-002..004` 能稳定输出 rust side 真实编译失败信息
4. 报告中能明确区分：
   - Harness 问题
   - 环境问题
   - `opencode-rs` 实现问题
5. 最小 smoke CLI 流程可连续执行，不再停留在单点手工排错

### 7.2 更优验收
若能额外做到以下内容更好：

- 对 help 类输出完成基本 normalization
- 生成 regression 记录，直指 `crates/tui/src/app.rs` 当前编译断裂
- 可批量执行最小 smoke CLI 任务目录并汇总结果

---

## 8. 验收命令

建议至少使用以下命令完成验收：

```bash
cargo run -- run --task harness/tasks/cli/SMOKE-CLI-001.yaml
cargo run -- run --task harness/tasks/cli/SMOKE-CLI-002.yaml
cargo run -- run --task harness/tasks/cli/SMOKE-CLI-003.yaml
cargo run -- run --task harness/tasks/cli/SMOKE-CLI-004.yaml
```

如支持目录级执行，也应补充：

```bash
cargo run -- run --task harness/tasks/cli
```

必要时直接验证 rust side 真实行为：

```bash
cd /Users/openclaw/Documents/github/opencode-rs/opencode-rust
cargo run -q -p opencode-cli -- --help
cargo run -q -p opencode-cli -- --version
cargo run -q -p opencode-cli -- workspace --help
cargo run -q -p opencode-cli -- --invalid-option
```

---

## 9. 下一轮输入

若本轮完成，下一轮可继续进入：

1. CLI smoke 批量执行与汇总
2. CLI 差异归一化规则扩展
3. regression / incident 资产沉淀
4. 将最小 smoke 流程接入更正式的 suite / gate / report 体系

---

## 10. 职责边界

再次强调：

- 本轮只增强 Harness 自身的执行、比较、报告与回归能力
- 不在 Harness 仓库中实现或修复 `opencode-rs` 产品逻辑
- Harness 的职责是更准确地暴露、归类、记录和追踪问题，而不是替被测项目完成开发
