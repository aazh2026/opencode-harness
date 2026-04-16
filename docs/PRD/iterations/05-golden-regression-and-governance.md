# Iteration 5: Golden, Regression and Governance

## 本轮目标
把一次性的差分能力升级为可长期维护的测试资产体系，并吸收 opencode-rs 的现实经验，让失败工件可以反向沉淀为 regression，而不是只停留在日志里。

## 前置依赖
- 已有稳定 raw artifacts
- 已有 verdict model
- 已有 mismatch 分类

## 本轮范围
本轮只处理：
- golden baseline 最小闭环
- regression case 模板与入库机制
- whitelist / allowed variance 文件治理
- baseline 元数据与版本绑定
- 从失败工件生成 regression 的最小路径

## 职责边界硬约束
- 本仓库只开发 opencode-harness 自身能力
- 本仓库禁止实现、补完或替代 opencode-rs 的产品功能
- 若发现 opencode-rs 与 opencode 不一致，只记录为 mismatch、contract gap、regression candidate 或 report item
- 不得把“修复 opencode-rs”当作本轮任务

## 明确不做
本轮不要做：
- 复杂审批平台
- 数据库或管理后台
- 智能推荐系统
- 自动聚类与趋势分析大系统

## 推荐实现顺序
1. 先定义 baseline metadata
2. 再实现 baseline record / compare
3. 再定义 regression case 模板
4. 再实现 whitelist 文件格式与过期逻辑
5. 最后实现从失败工件生成 regression 候选

## 设计要求
1. golden 是长期资产，不是一次性录制产物。
2. regression 必须最小复现、最小 fixture、可稳定重放。
3. whitelist 必须可追踪、可过期、可审计。
4. baseline 必须明确绑定 reference 版本、target 版本、normalizer 版本。

## Baseline Metadata 必须包含
- source_impl_version
- target_impl_version
- task_version
- fixture_version
- normalizer_version
- approved_by
- approved_at

## Regression 模板要求
每个 regression 至少包含：
- issue / bug 链接
- 背景说明
- root cause 摘要
- 最小 fixture
- task
- expected result
- severity
- execution_level（always_on / nightly_only / release_only）

## Whitelist 字段要求
至少包含：
- id
- scope
- reason
- owner
- expires_at
- linked_issue

## 失败处理原则
- baseline 缺版本信息：阻断本轮完成
- whitelist 无 owner / expires_at：不允许通过
- 某问题无法自动沉淀为 regression：允许 manual_check，但必须记录原因

## 验收标准
1. 某个 task 可以录制 baseline 并再次对比
2. 某个历史问题可以作为 regression 长期保存
3. whitelist 差异可被识别、审计并支持过期
4. baseline 元数据能说明基线来源与版本绑定

## 验收命令
- cargo test baseline_record_smoke_tests
- cargo test baseline_compare_smoke_tests
- cargo test regression_case_smoke_tests
- cargo test whitelist_rules_smoke_tests

## 下一轮输入
下一轮将在 baseline / regression 基础上实现：
- report schema
- suite 定义
- PR / nightly / release gate
