# opencode-harness PRD Overview

## 定位
opencode-harness 在 v1 阶段首先服务于 opencode-rs 对齐 opencode 的现实工作流，而不是优先追求通用平台化。

## 6 轮迭代关系
1. Foundation, Scope and Repo Structure
2. Task System and Fixtures
3. Runners and Differential Execution
4. Contracts, State Machine and Side Effects
5. Golden, Regression and Governance
6. Reporting, Metrics and CI Gates

## 全局约束
- 必须支持任务状态：todo / in_progress / done / manual_check / blocked / skipped
- 必须支持失败分类：implementation_failure / dependency_missing / environment_not_supported / infra_failure / flaky_suspected
- 必须支持环境预检查
- 必须显式区分环境问题与实现问题
- 必须支持人工介入，不得假设全自动成功闭环

## 使用方式
每轮 iterate 只应使用当前轮文档作为主输入，可引用上一轮产物摘要，但不应重新拼接整份 PRD。
