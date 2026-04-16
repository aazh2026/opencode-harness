# opencode-harness 独立测试项目 PRD

## 1. 文档信息

### 项目名称
opencode-harness

### 项目定位
一个独立于 opencode 与 opencode-rs 的测试与验证平台项目，用于持续验证 opencode-rs 在能力、行为、协议、状态机与副作用层面与 opencode 保持一致。

### 文档版本
v1.0

### 文档类型
PRD / 产品需求文档

### 目标读者
测试架构师、Harness 平台负责人、AI Coding 产品负责人、opencode / opencode-rs 核心研发、CI/CD 与质量平台团队

## 2. 项目背景

opencode 不是传统单体 CLI，也不是普通 Web 服务。它当前公开形态至少包含以下特征：
- 以 client/server architecture 为核心，TUI 只是其中一个客户端。
- 支持 opencode serve 启动 headless HTTP server，对外暴露 OpenAPI 3.1 接口。
- 支持 opencode web 以浏览器界面方式使用，并共享认证、端口、CORS、mDNS 等配置能力。
- 支持 acp、session 管理、stats、export/import 等命令。
- 具备 agent 体系，内建 Build、Plan 等 primary agents，Plan 明确用于“分析与建议、不做代码改动”，Build 工具权限更高。
- 桌面端当前为 Tauri v2。

因此，把 opencode 迁移或重构为 Rust 版本时，测试目标不能仅是“代码是否能运行”，而必须验证：
- 协议是否一致
- 状态机是否一致
- 权限语义是否一致
- 工具调用行为是否一致
- 文件系统与 Git 副作用是否一致
- 多客户端入口下的关键能力是否一致

如果这些验证散落在 opencode 仓或 opencode-rs 仓中，会有几个明显问题：
- 测试资产会偏向某一侧实现，不利于充当“中立裁判”
- 双方版本演进时，差分测试难以稳定治理
- regression 资产会被实现细节污染，难形成长期能力
- CI 只能验证各自“自洽”，难验证“跨实现一致”

因此需要一个独立测试项目，作为中立验证层与能力一致性控制面。

## 3. 项目愿景

构建一个独立的、工程化的、可持续演进的 Harness 项目，使其成为：

> opencode 与 opencode-rs 之间能力一致性的唯一权威验证层。

该项目不是普通测试仓，也不是单纯 E2E 集合，而是一个具备以下能力的测试平台：
- 统一描述测试任务与基线
- 统一驱动旧实现与新实现
- 统一采集协议、事件、状态与副作用
- 统一做 normalize、diff、判定与分级
- 统一沉淀 golden、contract、regression 资产
- 统一为 CI/CD、发布门禁、灰度验证提供依据

## 4. 项目目标

### 4.1 总体目标
确保 opencode-rs 在可验证的产品能力范围内，与 opencode 行为保持一致，支持持续回归验证与发布准入。

### 4.2 具体目标
1. 建立一套独立于实现仓库的能力基线体系
2. 建立一套可复用的差分测试执行框架
3. 建立一套独立的协议 / 状态 / 副作用比较规则
4. 建立一套长期维护的 golden / regression 资产库
5. 为 PR、nightly、release 提供不同层级的质量门禁
6. 让 opencode-rs 的迭代可以量化地向 opencode 对齐

## 5. 非目标

本项目不负责：
- 替代 opencode 仓内单元测试
- 替代 opencode-rs 仓内 Rust 原生单元/集成/性能测试
- 评估模型“聪明程度”或回答质量主观优劣
- 对真实 LLM 提供商做基准排行
- 充当通用 AI agent 测试平台的完整产品化版本

本项目首要职责，是做 opencode ↔ opencode-rs 能力等价验证。

## 6. 核心问题定义

### 6.1 为什么必须独立成项目
因为这里要解决的不是某一实现的正确性，而是两套实现之间的行为等价问题。独立项目可以天然具备：
- 中立性
- 双实现可插拔性
- 版本矩阵能力
- 测试资产集中治理能力
- 统一报告与门禁能力

### 6.2 为什么不能只做单测
对于 opencode 这类系统，核心风险并不只存在于函数内部，而在：
- session 生命周期
- server/API 行为
- streaming event 顺序
- 权限模型
- 文件副作用
- Git 变化
- reconnect / attach / recovery
- 多入口一致性

这类问题单靠实现仓内 unit test 很难覆盖。

### 6.3 为什么不能只做 UI/E2E
只做 UI/E2E 会导致：
- 执行慢
- 诊断差
- 脆弱
- 难定位差异来源
- 无法精确比较协议与副作用

所以独立项目必须以 contract + differential + state machine + side-effect 为主，以少量 smoke E2E 为辅。

## 7. 用户与使用场景

### 7.1 核心用户
- 用户 A：opencode-rs 核心研发
- 用户 B：opencode 原项目维护者
- 用户 C：测试架构与平台团队
- 用户 D：发布负责人

### 7.2 典型场景
- Rust 版本新增 serve 接口后，验证其与参考实现 API 行为是否一致
- Rust 版本实现 agent 切换后，验证 Build/Plan 语义是否一致
- Rust 版本修复某个工具副作用问题后，跑 regression 验证是否回归
- nightly 跑全量 golden corpus，输出 mismatch 趋势
- release 前对关键路径进行全面对齐验证

## 8. 产品范围

### 8.1 v1 必须覆盖
- CLI contract
- Server/OpenAPI contract
- Session / project / thread 核心状态机
- Agent / permission model
- Tool orchestration 基本链路
- Workspace / filesystem / Git side effects
- Golden corpus
- Differential runner
- Replay / deterministic provider
- CI gate / 报告输出

### 8.2 v1.5 建议覆盖
- Web smoke
- ACP basic compatibility
- Attach / reconnect
- Interrupted recovery
- Config / local state
- Multi-instance concurrency

### 8.3 v2 再考虑覆盖
- Desktop smoke
- 更复杂 IDE 交互链路
- 性能与资源使用对齐报告
- 智能 regression 推荐与 mismatch 聚类

## 9. 产品原则
- 中立裁判
- 行为优先
- 规则优先
- 回归资产优先
- 分层验证
- 可演进

## 10. 产品形态

opencode-harness 是一个独立仓库，逻辑上包含五层：
1. 资产层：fixtures、tasks、golden、regression
2. 契约层：CLI/API/permission/state/side-effect contracts
3. 执行层：legacy runner、rust runner、replay provider
4. 比较层：normalizer、comparator、verifier、diff classifier
5. 治理层：报告、白名单、门禁、趋势分析

## 11. 信息架构与仓库结构

```text
opencode-harness/
 docs/
 prd/architecture/
 contracts/
 fixtures/
 projects/
 configs/
 workspaces/
 tasks/
 cli/
 api/
 session/
 permissions/
 workspace/
 recovery/
 web/
 golden/
 raw/
 normalized/
 baselines/
 contracts/
 cli/
 api/
 events/
 permissions/
 state_machine/
 side_effects/
 runners/
 legacy/
 rust/
 shared/
 providers/
 replay/
 deterministic/
 normalizers/
 comparators/
 verifiers/
 regression/
 issues/
 bugs/
 incidents/
 reports/
 templates/
 schemas/
 scripts/
 ci/
```

## 12. 核心能力需求

### 12.1 任务定义系统
系统必须支持以数据化方式定义测试任务，而不是把场景硬编码到测试脚本里。

每个任务建议字段：
- id
- title
- category
- fixture_project
- preconditions
- entry_mode：cli / api / web / acp
- agent_mode
- provider_mode
- input
- expected_assertions
- allowed_variance
- severity
- tags

### 12.2 参考实现驱动器
需要一个 legacy runner，用统一方式拉起并驱动 opencode。

### 12.3 Rust 实现驱动器
需要一个 rust runner，用统一方式拉起并驱动 opencode-rs。

### 12.4 Differential Runner
读取 task，双边执行，采集、normalize、diff、报告。

### 12.5 Normalizer
统一规范化无业务意义差异：timestamp、UUID、临时路径、token usage、provider metadata、平台路径分隔符、日志噪声等。

### 12.6 Comparator
需要专门比较器：CLI / API / Event stream / Session state / Permission / Workspace / Git。

### 12.7 Side-effect Verifier
验证文件树变化、目标文件集、误改文件、patch 语义、git status/diff、cwd 隔离、tmpdir 清理、config 目录保护。

### 12.8 State Machine Verifier
验证 session lifecycle、thread lifecycle、project switch、agent switch、attach/reconnect、interruption/recovery、tool failure recovery、permission flow。

### 12.9 Golden Corpus 管理
支持录制 raw transcript、normalized transcript、baseline 版本化、审批与差异可视化。

### 12.10 Regression 管理
每个确认问题都应沉淀为 regression case。

### 12.11 报告与门禁
输出 CLI summary、JSON、JUnit、HTML 报告，并包含通过率、mismatch 分类、白名单与趋势。

## 13. 关键测试对象设计

### 13.1 CLI Contract
至少覆盖：serve、session、stats、export、import、web、acp、uninstall、upgrade。

### 13.2 Server/API Contract
至少覆盖：server 启停、/doc、认证、CORS、session/project 接口、message/command/file/tool 关键路径、event 订阅。

### 13.3 Agent / Permission Contract
至少覆盖：Build 允许写文件、Plan 不直接改文件、ask/deny/approve、agent switch 权限生效。

### 13.4 State Machine Contract
覆盖新 session、恢复、project 绑定、thread 切换、reconnect、interruption、recovery、tool error。

### 13.5 Side-effect Contract
覆盖目标文件修改、patch apply 冲突、git dirty workspace、tmpdir cleanup、config 目录保护。

## 14. 典型用户故事
- US-01：Rust 核心研发查看对齐情况
- US-02：测试架构师快速创建 regression case
- US-03：发布负责人查看高风险 mismatch 是否为 0
- US-04：维护者更新 golden baseline 且需要审批

## 15. 关键功能流程
- 15.1 差分测试执行流程
- 15.2 Golden 基线更新流程
- 15.3 Regression 入库流程

## 16. 核心指标

### 16.1 质量指标
- 能力覆盖率
- contract 覆盖率
- regression 覆盖率
- 高风险路径通过率
- P0 mismatch 数量
- P1 mismatch 数量

### 16.2 稳定性指标
- suite flake rate
- nightly 成功率
- baseline 更新误伤率
- 假阳性率

### 16.3 工程效率指标
- 单 task 平均执行时长
- PR gate 平均耗时
- 新 regression 入库耗时
- mismatch 定位时长

## 17. 版本规划

### v1
建立最小可用差分验证平台。

### v1.5
补齐高风险恢复与多实例场景。

### v2
升级成更完整的 Harness 平台。

## 18. CI/CD 集成设计

### 18.1 PR Gate
快速、稳定、硬门禁。

### 18.2 Nightly
扩大覆盖与发现趋势。

### 18.3 Release Qualification
发布前最终准入。

## 19. 权限与治理
- baseline 更新权限仅限指定 reviewer
- 白名单差异需要说明、owner、过期时间
- regression 原则上禁止删除

## 20. 风险与应对
- 真实 LLM 非确定性，v1 以 deterministic provider / replay provider 为主
- 测试资产增长，靠任务数据化与分级治理
- normalize 误掩盖问题，靠白名单审计与强比较
- 独立项目与版本演进脱节，靠版本矩阵与 nightly 联动
- UI 自动化脆弱，UI 只做 smoke

## 21. 项目依赖

### 外部依赖
- opencode 可执行参考实现
- opencode-rs 可执行目标实现
- deterministic / replay provider
- Git 与工作区环境
- CI 运行环境

### 内部依赖
- 任务语料维护机制
- baseline 审批机制
- regression 入库流程
- 报告消费方

## 22. 验收标准
1. 可独立运行，不依附于任一实现仓
2. 能同时驱动 opencode 与 opencode-rs
3. 支持至少 50 个高价值 task 的差分执行
4. 支持 golden baseline 生成、对比与审批
5. 支持 regression 长期运行
6. 支持 CLI/API/session/permission/workspace 五大类核心验证
7. 能输出 PR/nightly/release 三类报告
8. 能作为 opencode-rs 对齐进度的主要客观依据

## 23. 首批高价值测试用例建议

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

## 24. 实施建议
1. 阶段 1：骨架搭建
2. 阶段 2：最小能力闭环
3. 阶段 3：基线沉淀
4. 阶段 4：能力扩展

## 25. 最终结论

opencode-harness 应该是一个独立项目。它的职责不是替代 opencode 或 opencode-rs 仓内测试，而是作为两者之间的中立能力一致性验证层。它必须以 contract + differential + state machine + side-effect 为主，以少量 smoke E2E 为辅；必须以 deterministic 规则为裁判，以 golden 与 regression 为长期资产，以 CI/CD 报告与门禁为落地方向。
