# opencode-harness 独立测试项目 PRD

## 1. 文档信息

### 项目名称
opencode-harness

### 项目定位
一个独立于 opencode 与 opencode-rs 的测试与验证项目，用于持续验证 opencode-rs 在高价值产品能力上是否与 opencode 保持功能一致。

### 文档版本
v1.1

### 文档类型
PRD / 产品需求文档

### 目标读者
测试架构师、Harness 负责人、opencode-rs 核心研发、opencode 维护者、CI/CD 与质量平台团队、发布负责人。

## 2. 项目背景

opencode 当前公开形态至少包含以下特征：
- 以 client/server architecture 为核心，TUI 只是其中一个客户端。
- 支持 serve 启动 headless HTTP server，并对外暴露 OpenAPI 3.1 接口。
- 支持 web、acp、session 管理、stats、export/import 等命令。
- 具备 agent 体系，至少存在 Build、Plan 等 primary agents，且权限语义不同。
- 桌面端当前基于 Tauri v2。

因此，把 opencode 对齐或迁移到 Rust 版本时，真正需要验证的不是“代码是否能运行”，而是：
- 协议是否一致
- 状态机是否一致
- 权限语义是否一致
- 工具调用行为是否一致
- 文件系统与 Git 副作用是否一致
- 多入口下的关键能力是否一致

但结合 opencode-rs 当前实际推进情况，这个问题还有更现实的一面：
- opencode-rs 仍处于快速补能力与收敛行为阶段
- iterate 开发流程会遇到任务状态混乱、verification 卡死、环境依赖缺失等现实问题
- 并不是所有 mismatch 都值得同等优先级修复
- 并不是所有验证任务都适合阻断主线推进

因此，opencode-harness 的首要使命，不是先做成一个“通用完备测试平台”，而是先成为：

> **服务 opencode-rs 对齐工作的高价值差分裁判系统。**

## 3. 项目愿景

构建一个独立的、工程化的、可持续演进的 Harness 项目，使其成为：

> opencode 与 opencode-rs 之间高价值能力一致性的权威验证层。

该项目不是普通测试仓，也不是单纯 E2E 集合，而是一个具备以下能力的独立验证层：
- 统一描述测试任务与基线
- 统一驱动参考实现与目标实现
- 统一采集协议、事件、状态与副作用
- 统一做 normalize、diff、判定与分级
- 统一沉淀 golden、contract、regression 资产
- 统一为 PR、nightly、release 与人工迭代提供依据

## 4. 项目目标

### 4.1 总体目标
确保 opencode-rs 在可验证的高价值产品能力范围内，与 opencode 行为保持一致，并支持持续回归验证、人工排障与发布准入。

### 4.2 具体目标
1. 建立一套独立于实现仓库的能力基线体系
2. 建立一套以高价值任务为核心的差分测试执行框架
3. 建立协议 / 状态 / 副作用比较规则
4. 建立 golden / regression 资产库
5. 为 PR、nightly、release 提供分层门禁
6. 为 opencode-rs 的迭代开发提供客观反馈回路

## 5. 非目标

本项目不负责：
- 替代 opencode 仓内单元测试
- 替代 opencode-rs 仓内 Rust 原生单元/集成/性能测试
- 评估模型“聪明程度”或主观回答质量
- 对真实 LLM 提供商做横向基准排行
- 在 v1 阶段实现完整通用 AI Harness 平台

本项目首要职责，是做 opencode ↔ opencode-rs 高价值能力等价验证。

## 6. 产品原则

### 6.1 中立裁判
项目必须独立于任一实现仓，规则与基线独立维护。

### 6.1.1 双边执行与差分验证
同一份 task / fixture 必须能够同时驱动 `opencode` 与 `opencode-rs`，并对双边执行结果进行统一采集、normalize、compare、verify 与 report。Harness 不以“只测 opencode-rs”作为目标，而以“用同一任务验证 opencode-rs 是否与 opencode 行为一致”作为核心工作流。

### 6.2 行为优先
优先验证可观察行为，而不是内部实现方式。

### 6.3 副作用优先于文本相似
对于 coding-agent 能力，默认遵循：

> 副作用一致性 > 状态语义一致性 > 协议一致性 > 文本输出相似性

### 6.4 规则优先
判定必须尽量由 deterministic 规则完成，而不是把 LLM 当裁判。

### 6.5 回归资产优先
每一个已确认问题都应尽量沉淀为长期 regression 资产。

### 6.6 分层验证
contract、state machine、side-effect、smoke E2E 各自分层，不混成单一大套件。

### 6.7 人工介入是正式流程
人工确认、人工跳过、人工审阅不是异常，而是 Harness 正式工作流的一部分。

## 7. 对齐优先级模型

结合 opencode-rs 当前实际情况，v1 必须采用分层对齐策略。

### Tier 1: 核心阻断能力
优先验证：
- CLI contract
- Server/API contract
- session / state
- permission semantics
- workspace / filesystem / Git side effects

### Tier 2: 高价值交互能力
在 Tier 1 稳定后补充：
- streaming / event ordering
- ACP basic compatibility
- reconnect / recovery
- web smoke

### Tier 3: 外围与非功能能力
后置处理：
- desktop smoke
- benchmark / profiling
- 性能与资源使用对齐
- 更复杂多实例场景

## 8. 差异与结果分类

不是所有差异都应同等处理。

### 8.1 差异级别
- Blocking mismatch：必须优先修复
- High-value mismatch：应进入近期迭代
- Allowed variance：允许短期存在，但需记录
- Environment-limited：受环境限制，非实现缺陷
- Manual-check-only：暂时只能人工确认

### 8.2 任务状态
Harness 必须支持以下任务状态：
- todo
- in_progress
- done
- manual_check
- blocked
- skipped

### 8.3 失败分类
至少区分：
- implementation_failure
- dependency_missing
- environment_not_supported
- infra_failure
- flaky_suspected

## 9. 环境限制与人工检查策略

结合 opencode-rs 的实际迭代经验，环境问题必须从实现问题中剥离。

### 9.1 环境预检查必须覆盖
- opencode binary 是否可用
- opencode-rs binary 是否可用
- cargo audit 等外部工具是否存在
- profiling / browser / network 等运行环境是否满足
- deterministic / replay provider 是否准备好

### 9.2 处理原则
- 核心依赖缺失：标记为 blocked
- 非核心依赖缺失：可标记为 manual_check 或 skipped
- 环境不满足：不得直接判定为实现不兼容
- verification 任务不得因为外部工具缺失而无限修复循环

## 10. 与迭代开发流程的关系

opencode-harness 不只是 CI 工具，也应服务于 opencode-rs 的实际迭代闭环。

它应支持：
- 差距识别
- 高价值 mismatch 暴露
- regression 沉淀
- hourly 进度汇报所需的工件读取
- 后续 PRD/task generation 的输入回灌

也就是说，Harness 既是验证层，也是迭代反馈系统。

## 11. 产品范围

### 11.1 v1 必须覆盖
- CLI contract
- Server/OpenAPI contract
- Session / project / thread 核心状态机
- Agent / permission model
- Tool orchestration 基本链路
- Workspace / filesystem / Git side effects
- Golden corpus 最小闭环
- Differential runner
- Replay / deterministic provider
- PR / nightly / release 报告输出

### 11.2 v1.5 建议覆盖
- Web smoke
- ACP basic compatibility
- Attach / reconnect
- Interrupted recovery
- Config / local state
- Multi-instance concurrency

### 11.3 v2 再考虑覆盖
- Desktop smoke
- 更复杂 IDE 交互链路
- 性能与资源使用对齐报告
- 智能 mismatch 聚类与趋势推荐

## 12. 项目结构

逻辑上包含五层（除 docs/、scripts/ 等辅助目录外，核心项目目录统一位于 harness/ 下）：
1. 资产层：fixtures、tasks、golden、regression
2. 契约层：CLI/API/permission/state/side-effect contracts
3. 执行层：legacy runner、rust runner、replay provider
4. 比较层：normalizer、comparator、verifier、diff classifier
5. 治理层：报告、白名单、门禁、趋势分析

## 13. 核心能力需求

### 13.1 任务定义系统
系统必须支持以数据化方式定义测试任务，而不是把场景硬编码到测试脚本里。

任务建议字段至少包含：
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

### 13.2 Runner 系统
需要 legacy runner 与 rust runner，并对外暴露等价接口。

其中：
- legacy runner 负责驱动 `opencode`
- rust runner 负责驱动 `opencode-rs`
- 两者必须接受同一 task / fixture / assertion 输入模型
- 两者输出必须能进入同一套 differential runner 与 verifier 流程

### 13.3 Differential Runner
读取 task，用相同输入驱动双边，收集 raw artifacts，执行 normalize / compare / verify，输出结果。

标准流程必须是：
1. 准备同一份 task 与 fixture
2. 执行 `opencode`
3. 执行 `opencode-rs`
4. 采集双边 artifacts
5. 归一化无业务意义差异
6. 输出 parity verdict、mismatch 分类与报告

### 13.4 Normalizer
只归一化无业务意义差异，且应支持审计输出，说明应用了哪些规则。

### 13.5 Comparator
至少包含 CLI、API、Event stream、Session state、Permission、Workspace、Git 比较器。

### 13.6 Side-effect Verifier
验证文件树变化、目标文件集、误改文件、patch 语义、git status/diff、cwd 隔离、tmpdir 清理、config 目录保护。

### 13.7 State Machine Verifier
验证 session lifecycle、thread lifecycle、project switch、agent switch、attach/reconnect、interruption/recovery、tool failure recovery、permission flow。

### 13.8 Golden 管理
支持 raw transcript 录制、normalized transcript、baseline 版本化、审批与差异可视化。

### 13.9 Regression 管理
每个确认问题都要沉淀为 regression case，并支持长期运行。

### 13.10 报告与门禁
输出 CLI summary、JSON、JUnit、HTML 报告，并支持 suite 分层执行。

## 14. 关键测试对象设计

### 14.1 CLI Contract
至少覆盖：serve、session、stats、export、import、web、acp、uninstall、upgrade。

### 14.2 Server/API Contract
至少覆盖：server 启停、/doc、认证、CORS、session/project 接口、message/command/file/tool 关键路径、event 订阅。

### 14.3 Agent / Permission Contract
至少覆盖：Build 允许写文件、Plan 不直接改文件、ask/deny/approve、agent switch 权限生效。

### 14.4 State Machine Contract
覆盖新 session、恢复、project 绑定、thread 切换、reconnect、interruption、recovery、tool error。

### 14.5 Side-effect Contract
覆盖目标文件修改、patch apply 冲突、git dirty workspace、tmpdir cleanup、config 目录保护。

## 15. 首批高价值测试用例建议

### CLI
- opencode --help
- serve --port/--hostname/--cors
- web 基本启动
- acp 基本启动
- session list
- export/import 最小闭环

### Agent/Permission
- Build 写文件成功
- Plan 写文件被阻止
- ask -> deny
- ask -> approve
- agent switch 后权限切换生效

### Session/State
- 新建 session
- 恢复 session
- project switch
- reconnect
- tool failure recovery

### Workspace
- 仅修改目标文件
- patch apply 冲突
- git dirty workspace
- tmpdir cleanup
- config 目录保护

### Web/Server
- /doc 可访问
- basic auth
- event stream 基本订阅
- web smoke

## 16. 核心指标

v1 优先落地：
- pass_rate
- P0 mismatch count
- P1 mismatch count
- flake_count
- avg_runtime
- environment-limited count
- manual_check count

## 17. CI/CD 集成设计

### 17.1 PR Gate
快速、稳定、硬门禁，只跑核心 smoke 与高价值 parity。

### 17.2 Nightly
扩大覆盖、跑全量 golden / regression、观察趋势。

### 17.3 Release Qualification
发布前最终准入，要求 P0 mismatch = 0，高风险路径全绿，白名单已审计。

## 18. 权限与治理

### 18.1 baseline 更新权限
仅限指定 reviewer 组。

### 18.2 白名单管理
每个白名单项必须包含：
- 差异说明
- 影响范围
- 风险等级
- 过期时间
- owner

### 18.3 regression 删除规则
原则上禁止删除；如需删除，必须说明原因并审批。

## 19. 验收标准

项目满足以下条件时视为 v1 达标：
1. 可独立运行，不依附于任一实现仓
2. 能同时驱动 opencode 与 opencode-rs
3. 能覆盖核心高价值能力面，而不是只追求任务数量
4. 支持 golden baseline 生成、对比与审批
5. 支持 regression 长期运行
6. 支持 CLI/API/session/permission/workspace 五大类核心验证
7. 能输出 PR/nightly/release 三类报告
8. 能作为 opencode-rs 对齐进度的主要客观依据

## 20. 实施建议

### 阶段 1：骨架搭建
- 建 repo
- 建目录
- 建 runner 接口
- 建任务格式
- 建最小报告格式

### 阶段 2：最小能力闭环
- 先打通 CLI/API/session/permission/workspace 五类验证
- 先做 20 个以内高价值 task

### 阶段 3：基线沉淀
- 从 opencode 录制首批 golden
- 建 regression 机制
- 接入 PR/nightly

### 阶段 4：能力扩展
- attach/reconnect
- web smoke
- 多实例
- 更复杂恢复路径

## 21. 迭代可用性里程碑

为了避免长期停留在“骨架完成但无法实际测试”的状态，项目应以可测试能力为核心里程碑推进。

### Milestone A：Iteration 2 结束
- 完成 task system 与 fixture system
- 可以稳定描述最小真实对比任务
- 仍以人工执行双边命令、人工判读为主

### Milestone B：Iteration 3 结束
- Harness 首次具备**最小自动化真实对比能力**
- 至少支持少量 CLI contract case 的自动双边执行
- 至少支持收集双边 stdout/stderr/exit code 与基础 artifacts
- 至少支持输出最小 parity verdict

### Milestone C：Iteration 4 结束
- Harness 具备**可持续扩展的自动化对比能力**
- 可稳定覆盖 CLI / API / contract / side-effect 的基础场景
- comparator / verifier / normalizer 开始具备实际判定价值

### Milestone D：Iteration 5-6
- 扩展为 golden / regression / governance / CI gate 完整闭环
- 持续减少人工检查比例，但不追求完全消灭人工介入

## 22. 最终结论

opencode-harness 应该是一个独立项目，但它在 v1 阶段首先应服务于 opencode-rs 的现实对齐工作，而不是追求过早平台化。它的职责不是替代 opencode 或 opencode-rs 仓内测试，而是作为两者之间的中立能力一致性验证层。它必须以 contract + differential + state machine + side-effect 为主，以少量 smoke E2E 为辅；必须承认环境限制、人工介入与任务状态混乱是现实问题，并将这些现实因素设计进工作流本身。

