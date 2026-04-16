# Iteration 4: Contracts, State Machine and Side Effects

## 本轮目标
把“能跑”升级为“能判断对不对”，建立 harness 的核心裁判能力，并且显式支持 manual_check / allowed_variance / environment-limited，而不是一刀切 pass/fail。

## 前置依赖
- 已有双边 raw artifacts
- 已有 differential runner 结果对象
- 已有失败分类

## 本轮范围
本轮只处理：
- normalizer 最小规则
- comparator 基础框架
- side-effect verifier 基础框架
- state machine verifier 基础框架
- 高优先级 contract 覆盖
- verdict model

## 职责边界硬约束
- 本仓库只开发 opencode-harness 自身能力
- 本仓库禁止实现、补完或替代 opencode-rs 的产品功能
- 若发现 opencode-rs 与 opencode 不一致，只记录为 mismatch、contract gap、regression candidate 或 report item
- 不得把“修复 opencode-rs”当作本轮任务

## 明确不做
本轮不要做：
- 完整 Web / ACP / desktop 全覆盖
- 复杂 long-session/多实例场景
- 自动 baseline 审批
- 智能 mismatch 聚类

## 推荐实现顺序
1. 先定义 verdict model
2. 再定义 normalizer rule 审计结构
3. 再实现 CLI / permission / workspace comparator
4. 再实现 side-effect verifier
5. 最后实现 state machine 最小主链验证

## 设计要求
1. 只归一化无业务意义差异，绝不掩盖真实不兼容。
2. comparator 必须区分强一致、语义一致、允许差异、严重不兼容。
3. side-effect 验证必须以文件和 git 结果为核心。
4. state machine 先覆盖主链路，不要贪多。
5. normalizer 必须输出审计信息，说明应用了哪些规则。

## Verdict Model
至少支持：
- pass
- pass_with_allowed_variance
- warn
- fail
- manual_check
- blocked

## 本轮优先覆盖的 Contract
建议先做：
- CLI contract
- permission contract
- workspace side-effect contract
- session 最小状态机 contract

## Side-effect 快照要求
至少定义：
- 执行前文件树快照
- 执行后文件树快照
- git status 快照
- git diff / patch 快照
- 非目标文件变更检查
- 受保护目录检查

## 失败处理原则
- 规则未定义导致无法自动判定：manual_check
- 环境导致无法比较：blocked 或 environment-limited
- 比较规则已具备且发现不一致：fail

## 验收标准
1. 至少一个 smoke task 可完成 normalize + compare + verdict
2. side-effect verifier 能识别误改文件
3. permission 流程能做最小状态判断
4. mismatch 可被分类为 fail / allowed variance / manual_check

## 验收命令
- cargo test normalizer_smoke_tests
- cargo test comparator_smoke_tests
- cargo test side_effect_verifier_smoke_tests
- cargo test state_machine_smoke_tests

## 下一轮输入
下一轮将在 verdict 能力基础上实现：
- golden baseline
- regression case
- whitelist / governance
