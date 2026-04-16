# Iteration 2: Task System and Fixtures

## 本轮目标
建立可数据化驱动的任务系统与 fixture 系统，并吸收 opencode-rs 的现实经验，把 execution policy、环境缺失处理和 workspace 生命周期写进 schema，而不是留给后面补锅。

## 前置依赖
- Iteration 1 已完成工程骨架
- 已有任务状态、失败分类、基础路径约定

## 本轮范围
本轮只处理：
- task schema
- fixture schema
- workspace 生命周期
- 配置样本与项目样本
- 最小 smoke tasks
- schema 校验和载入流程

## 职责边界硬约束
- 本仓库只开发 opencode-harness 自身能力
- 本仓库禁止实现、补完或替代 opencode-rs 的产品功能
- 若发现 opencode-rs 与 opencode 不一致，只记录为 mismatch、contract gap、regression candidate 或 report item
- 不得把“修复 opencode-rs”当作本轮任务

## 明确不做
本轮不要做：
- 双实现差分执行
- compare / normalize 真实逻辑
- golden baseline 管理
- regression 治理
- CI gate

## 推荐实现顺序
1. 先定义 task schema 和 assertion model
2. 再定义 execution_policy 字段
3. 再定义 fixture schema 与 workspace 生命周期
4. 再补 schema validator 和 loader
5. 最后写 5 到 10 个高价值 smoke tasks

## 设计要求
1. task 必须数据化，禁止把场景散落硬编码在脚本中。
2. schema 要定义字段类型、必填项、默认值、约束和示例。
3. fixture 必须具备可复现性，避免“看环境脸色”。
4. workspaces 必须与 fixtures 分离，运行时污染不能写回 fixture 原件。
5. task 必须显式表达环境缺失时如何处理，而不是默认失败重试。

## 必须定义的 Task 字段
至少包含：
- id
- title
- category
- fixture_project
- preconditions
- entry_mode
- agent_mode
- provider_mode
- input
- expected_assertions
- allowed_variance
- severity
- tags
- execution_policy
- timeout_seconds
- on_missing_dependency
- expected_outputs

## Assertion Model
至少支持：
- exit_code_equals
- stdout_contains
- stderr_contains
- file_changed
- no_extra_files_changed
- permission_prompt_seen

## 必须定义的 Fixture 约定
至少说明：
- fixture 项目存放位置
- 配置样本存放位置
- transcript/recording 存放位置
- runtime workspace 初始化方式
- fixture 是否允许联网
- fixture 是否允许 dirty git 状态
- fixture 重置策略
- fail 后 workspace 是否保留

## 必须产物
至少产出：
- task schema 文档
- task schema 校验器
- fixture 约定文档
- workspace lifecycle 文档
- 3 到 5 个 fixture 样本
- 5 到 10 个高价值 smoke task 样本
- task 加载与校验最小实现

## 失败处理原则
- schema 非法属于 implementation_failure，阻断本轮
- fixture 缺样本属于实现缺失，阻断本轮
- 外部 binary 缺失不是本轮问题，不应阻断本轮

## 验收标准
1. 任意一个 task 文件可以被解析和校验
2. fixture 可以初始化为独立 workspace
3. smoke task 集能被统一加载
4. execution policy 能表达 manual_check / blocked / skip

## 验收命令
- cargo test task_schema_tests
- cargo test fixture_loader_tests
- cargo test smoke_task_loading_tests

## 下一轮输入
下一轮会在 task/fixture 基础上，实现：
- legacy runner
- rust runner
- differential runner 的最小闭环
