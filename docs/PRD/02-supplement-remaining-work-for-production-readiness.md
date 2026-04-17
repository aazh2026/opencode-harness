# opencode-harness 补充 PRD：距离正式可用仍需完成的工作

## 1. 文档目的

本补充 PRD 不重复主 PRD 的总体目标，而是专门回答一个更现实的问题：

> **项目：`opencode-harness` 距离“正式可用的自动化对比测试项目”还差哪些工作？**

这里的“正式可用”指：
- 能稳定执行 `opencode` 与 `opencode-rs` 的真实双边测试
- 能稳定收集、归一化、比较并判定结果
- 能以较低人工介入成本持续运行
- 能作为 `opencode-rs` 与 `opencode` 一致性验证的长期保障项目

## 2. 当前状态判断

截至当前，`opencode-harness` 已完成多轮骨架和执行层建设，已经具备：
- task / fixture / report 的基础 schema
- runner / differential runner / comparator / verifier / normalizer 的初步实现
- 初步的 artifact 与 verdict 模型
- 基本可运行的工程结构与测试体系

但它**仍未达到正式可用**。

当前更准确的定位是：

> **可以支持真实测试能力开发与试跑，但还不是稳定、可信、可持续的自动化对比测试平台。**

## 3. 正式可用的定义

`opencode-harness` 达到“正式可用”时，应满足以下最低标准：

1. 能以同一份 task / fixture 稳定驱动 `opencode` 与 `opencode-rs`
2. 能稳定收集 stdout / stderr / exit code / artifacts / side effects
3. 能对最小高价值 case 给出可信 verdict
4. 能区分真实不一致、环境问题、人工检查项
5. 能持续输出结构化报告
6. 能在定时巡检中自动提交、自动 push、自动汇报
7. 不因目录脏数据、脚本重试、命名不一致等问题反复中断

## 4. 剩余工作总览

剩余工作按优先级分为三层：
- **P0：正式可用前的主阻塞项**
- **P1：正式可用后必须尽快补齐的可靠性项**
- **P2：增强项与规模化项**

---

## 5. P0：正式可用前的主阻塞项

### P0-1. 稳定打通真实双边 Runner

#### 目标
让 `LegacyRunner` 与 `RustRunner` 真正、稳定地执行双边目标，而不是停留在局部可跑或测试专用路径上。

#### 必须完成
- `LegacyRunner` 稳定调用 `opencode`
- `RustRunner` 稳定调用 `opencode-rs`
- 统一 RunnerInput 模型
- 统一 ExecutionResult 模型
- 支持 working directory / env / timeout / binary_path / args
- 正确处理 stdout / stderr / exit code / duration

#### 验收标准
- 至少 3 个 CLI smoke case 可稳定双边执行
- 连续多次运行结果结构一致
- 不因临时路径、命令拼接或测试命名错误而失败

---

### P0-2. 打通 Differential Runner 最小闭环

#### 目标
让 harness 不只是“能跑两边”，而是真正形成自动化差分闭环。

#### 必须完成
- 同一 task + fixture 驱动双边执行
- 双边 artifacts 统一落盘
- 执行完成后自动进入 normalize / compare / verify
- 输出统一 parity verdict

#### 最小闭环流程
1. 准备 task
2. 准备 fixture/workspace
3. 执行 `opencode`
4. 执行 `opencode-rs`
5. 收集双边 artifacts
6. normalize
7. compare
8. verify
9. 输出 report/verdict

#### 验收标准
- 至少 1 个 smoke task 能完整走完以上 9 步
- 能输出 `pass / fail / manual_check / blocked` 中的有效 verdict

---

### P0-3. 让 Task System 从“可描述”变成“可执行”

#### 目标
当前 task schema 已经存在，但必须进一步成为 runner 可以真正消费的输入，而不是只做静态描述。

#### 必须完成
- `entry_mode` 真正影响执行路径
- `agent_mode` / `provider_mode` 真正可传入执行层
- `expected_assertions` 真正进入验证逻辑
- `execution_policy` 真正决定重试 / 跳过 / manual_check 行为
- `on_missing_dependency` 真正驱动 blocked / manual_check / skip
- `expected_outputs` 真正进入 artifact 校验

#### 验收标准
- 至少 5 个真实 smoke task 能被统一加载与执行
- task 字段不再停留在“定义了但没被使用”状态

---

### P0-4. 落地 Fixture / Workspace 生命周期

#### 目标
建立真实可用的执行环境隔离与副作用控制模型。

#### 必须完成
- fixture 初始化
- workspace 复制/创建
- 执行前快照
- 执行后快照
- 失败后保留策略
- 清理策略
- dirty workspace 控制

#### 验收标准
- 至少 1 个 workspace case 可证明 fixture 原件不被污染
- 至少 1 个失败 case 可保留现场用于诊断

---

### P0-5. 最小真实 CLI Case 自动化跑通

#### 目标
先拿最小高价值 case 跑通真实自动化，而不是一开始追求复杂场景。

#### 推荐首批用例
- `--version`
- `--help`
- `models --help`
- `session --help`
- `serve --help`

#### 验收标准
- 上述至少 3 个 case 可自动双边执行
- 能输出 structured diff / verdict
- 能复现之前手工对比中发现的 CLI contract mismatch

---

### P0-6. 稳定化执行流程，消除当前脚本级脆弱点

#### 目标
解决当前已经暴露出来、会阻断正式可用的流程问题。

#### 已知问题
- 文件生成经常第一次失败后重试
- 测试目标命名偶尔错误
- 目录编号与 PRD 阶段编号错位
- artifacts 容易污染仓库
- 自动提交/推送逻辑偶尔滞后于实际状态

#### 必须完成
- 为 `generate_if_missing` 增加更稳的检查与失败信息
- 对测试命令目标名做校验
- 明确区分“PRD 阶段编号”和“目录编号”
- artifacts 路径稳定隔离，不污染源码目录
- 自动 commit / push 流程固定化

#### 验收标准
- 同一轮迭代不再频繁因文件落盘重试而显著拖慢
- 不再因为 test target 名字写错而卡一轮

---

## 6. P1：正式可用后应尽快补齐的可靠性项

### P1-1. Normalizer 真实规则集

需要从 trait 升级为真实规则：
- 时间戳归一化
- 随机 ID 归一化
- 临时路径归一化
- 无关顺序归一化
- 空白/格式噪音归一化

### P1-2. Comparator 真实判定逻辑

至少要有：
- CLI 文本比较器
- JSON 结构比较器
- command surface 比较器
- option surface 比较器
- 基础 artifact 比较器

### P1-3. Verifier 真实 verdict 规则

需要完善：
- fail
- pass
- pass_with_allowed_variance
- manual_check
- blocked
- environment_limited

### P1-4. Side-effect Verifier 最小可用版

至少支持：
- 文件新增/删除/修改检测
- 非目标文件误改检测
- 受保护目录检查
- git status / diff 检查

### P1-5. State Machine Verifier 最小可用版

至少支持：
- session 建立
- session 恢复
- 基本 attach/reconnect
- tool failure recovery 的最小路径

### P1-6. 自动报告可读性与结构化输出增强

至少补强：
- structured mismatch summary
- failure type breakdown
- artifact 路径索引
- 人工检查项列表

---

## 7. P2：增强项与规模化项

### P2-1. Golden / Regression 体系完整化
- baseline 录制
- baseline 审批
- regression 回灌
- whitelist 过期机制

### P2-2. CI / Gate 自动化
- PR gate
- nightly gate
- release qualification

### P2-3. API / session / permission / side-effect 大规模扩展
- 更复杂 contract
- 更复杂 workspace 场景
- 更复杂多会话状态场景

### P2-4. 更强的审计与趋势分析
- mismatch 趋势
- regression 命中率
- environment-limited 占比
- manual_check 占比

---

## 8. 推荐实施顺序

### 第一阶段：先到“最小正式可用”
优先顺序：
1. P0-1 Runner
2. P0-2 Differential Runner
3. P0-3 可执行 Task System
4. P0-4 Fixture/Workspace 生命周期
5. P0-5 最小 CLI case 自动化
6. P0-6 流程稳定化

### 第二阶段：再到“基础可靠可用”
优先顺序：
1. P1-1 Normalizer
2. P1-2 Comparator
3. P1-3 Verifier
4. P1-4 Side-effect verifier
5. P1-5 State machine verifier
6. P1-6 报告增强

### 第三阶段：再到“持续集成可用”
优先顺序：
1. Golden / Regression
2. CI / Gate
3. 大规模覆盖扩展
4. 趋势分析

---

## 9. 与当前迭代的关系

### 当前判断
`opencode-harness` 当前正在推进的迭代，主要在解决：
- 真实 Runner
- Differential execution
- 最小 verdict 与 artifacts

这正好落在本补充 PRD 的 **P0 主阻塞项** 上。

也就是说：

> **当前迭代正在做的，就是“距离正式可用最近、优先级最高”的那一批工作。**

### 实际含义
如果当前执行层迭代做扎实，项目会从：
- “可试跑、可开发”

推进到：
- “最小正式可用”

但如果这一轮只做了表面接口，没有形成稳定闭环，那么项目依然不能称为正式可用。

---

## 10. 最终结论

`opencode-harness` 当前还不是正式可用的自动化对比测试平台。

距离正式可用，最关键的工作不是继续补抽象，而是完成以下核心闭环：
- 真实双边 Runner
- Differential Runner 完整闭环
- 可执行的 task system
- fixture/workspace 生命周期
- 最小真实 CLI case 自动化
- 脚本与流程稳定化

当这些 P0 项目完成后，`opencode-harness` 才能第一次被视为：

> **一个真正可以开始承担自动化对比测试职责的项目。**
