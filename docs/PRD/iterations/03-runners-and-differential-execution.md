# Iteration 3: Runners and Differential Execution

## 本轮目标
打通最小差分执行闭环，让 harness 可以用同一份 task 分别驱动 legacy 与 rust 两边实现，并且明确区分实现失败、环境失败、依赖缺失，避免重演 opencode-rs 中 verification 误判成代码问题的情况。

## 前置依赖
- 已有 task schema
- 已有 fixture loader
- 已有 workspace 初始化逻辑

## 本轮范围
本轮只处理：
- runner interface
- legacy runner 最小实现
- rust runner 最小实现
- differential runner skeleton
- raw artifacts 采集
- 最小失败分类与结果对象
- capability reporting

## 职责边界硬约束
- 本仓库只开发 opencode-harness 自身能力
- 本仓库禁止实现、补完或替代 opencode-rs 的产品功能
- 若发现 opencode-rs 与 opencode 不一致，只记录为 mismatch、contract gap、regression candidate 或 report item
- 不得把“修复 opencode-rs”当作本轮任务

## 明确不做
本轮不要做：
- 完整 normalize 规则
- 完整 comparator 规则
- 完整 side-effect verifier
- 完整 state machine verifier
- golden baseline 管理
- 多实例并发调度
- web / desktop 深度自动化

## 推荐实现顺序
1. 先定义 runner input/output contract
2. 再定义 capability reporting
3. 再实现 legacy runner 最小 CLI 路径
4. 再实现 rust runner 最小 CLI 路径
5. 最后实现 differential runner 和 raw artifact 落盘

## 设计要求
1. legacy runner 与 rust runner 对外必须暴露等价接口。
2. differential runner 优先顺序执行，先求清晰。
3. 原始工件必须完整保留，避免过早丢失诊断信息。
4. 必须区分 implementation_failure、dependency_missing、environment_not_supported、infra_failure。

## 必须定义的 Runner Input
至少包含：
- task
- prepared_workspace_path
- env_overrides
- timeout_seconds
- binary_path
- provider_mode
- capture_options

## 必须定义的执行结果对象
至少包含：
- exit_code
- stdout_path / stderr_path
- artifact_paths
- session_metadata
- event_log_path
- side_effect_snapshot_path
- duration
- failure_kind
- capability_summary

## Artifact 目录约定
必须统一：
- `artifacts/run-<id>/legacy/...`
- `artifacts/run-<id>/rust/...`
- `artifacts/run-<id>/diff/...`

## 失败处理原则
- binary 缺失：dependency_missing
- 环境不支持：environment_not_supported
- runner 崩溃：infra_failure
- 双边可运行但行为异常：implementation_failure

## 验收标准
1. 同一 smoke task 可被 legacy/rust 双边执行
2. 双边 raw artifacts 都会落盘
3. differential runner 能输出最小 diff result
4. 单边失败时能正确分类，而不是直接误判不兼容

## 验收命令
- cargo test legacy_runner_smoke_tests
- cargo test rust_runner_smoke_tests
- cargo test differential_runner_smoke_tests

## 下一轮输入
下一轮将在 raw artifacts 基础上实现：
- normalizer
- comparator
- side-effect verifier
- state machine verifier
