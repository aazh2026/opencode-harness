# Iteration 6: Reporting, Metrics and CI Gates

## 本轮目标
完成 v1 的交付闭环，让 harness 产出对人和机器都可消费的报告，并能服务 PR、nightly、release 门禁，同时也能支撑 hourly 进度汇报等真实工作流。

## 前置依赖
- 已有 baseline / regression / whitelist
- 已有 verdict model 与失败分类
- 已有 artifacts 目录约定

## 本轮范围
本轮只处理：
- report schema
- suite 定义
- PR gate
- nightly gate
- release qualification gate
- 关键指标输出
- 进度汇报所需的统计工件约定

## 职责边界硬约束
- 本仓库只开发 opencode-harness 自身能力
- 本仓库禁止实现、补完或替代 opencode-rs 的产品功能
- 若发现 opencode-rs 与 opencode 不一致，只记录为 mismatch、contract gap、regression candidate 或 report item
- 不得把“修复 opencode-rs”当作本轮任务

## 明确不做
本轮不要做：
- 智能 mismatch 聚类
- 高级数据看板
- 复杂性能分析平台
- 桌面深度自动化

## 推荐实现顺序
1. 先定义 report schema
2. 再定义 suite 配置与选择规则
3. 再输出 CLI / JSON / JUnit
4. 再实现 PR/nightly/release gate
5. 最后补 hourly progress 所需统计输出

## 设计要求
1. 报告必须支持 CLI summary、JSON、JUnit，HTML 可作为增强。
2. gate 必须分层，不同阶段跑不同集合。
3. release gate 必须有硬规则。
4. 指标优先少而硬。
5. 报告必须显式区分 implementation failure、environment failure、manual_check。

## 必须定义的 Suite
至少拆分：
- pr-smoke
- nightly-full
- release-qualification

每个 suite 必须明确：
- 包含哪些 task 类别
- 是否允许 whitelist
- 是否允许 skipped / manual_check
- 失败后如何输出 artifacts

## 必须定义的 Report Schema
至少包含：
- summary
- task_results
- mismatch_counts
- severity_aggregation
- whitelist_applied
- artifact_links
- suite_info
- failure_type_breakdown
- manual_check_count
- environment_limited_count

## Hourly Progress 工件要求
必须支持读取：
- done / in_progress / todo / manual_check / blocked / skipped 统计
- 当前运行任务
- 最近日志尾部
- 最近 artifacts / report 路径

## 失败处理原则
- 环境受限任务不应直接导致 release gate fail，除非被标记为核心阻断项
- report 生成失败属于 infra_failure
- gate 必须清楚区分 fail / blocked / manual_check

## 验收标准
1. 同一执行结果可输出 CLI + JSON + JUnit
2. PR/nightly/release 三类 suite 可单独执行
3. gate 失败时能输出关键 mismatch 与 artifacts 路径
4. 可以产出 hourly progress 所需的统计信息

## 验收命令
- cargo test report_schema_smoke_tests
- cargo test suite_selection_smoke_tests
- cargo test ci_gate_smoke_tests

## v1 完成判定
当本轮完成后，项目应满足：
- 能独立驱动 opencode 与 opencode-rs
- 能执行高价值差分任务
- 能沉淀 baseline 与 regression
- 能生成 CI 消费报告
- 能作为对齐进度与迭代反馈的客观门禁
