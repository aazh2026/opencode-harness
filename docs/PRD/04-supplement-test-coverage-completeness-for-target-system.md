# opencode-harness 增量 PRD：目标系统测试用例完整性补齐

## 1. 背景

截至当前阶段，`opencode-harness` 已经具备最小真实差分执行能力，并且已开始在 smoke CLI 场景中暴露真实问题。但从目标系统验证角度看，现有 task 集仍明显不完整。

目前仓库内虽已有 `cli`、`api`、`session`、`permissions`、`workspace` 等分类任务，以及若干 smoke case，但这些内容仍更接近：

- 最小可启动样例
- 初始任务骨架
- 局部能力试跑资产

而不是一个对目标系统形成系统化覆盖的测试用例体系。

如果不尽快补齐覆盖框架，将出现以下问题：

1. Harness 能执行，但发现问题的范围有限
2. 差异报告容易被个别 case 主导，缺少整体代表性
3. CI / gate / regression 后续都会建立在不完整样本上
4. opencode-rs 与 opencode 的真实高价值差异无法被持续、分层、稳定地看见

因此，在最小 smoke CLI 流程逐步跑通后，必须尽快补一轮专注于“测试用例完整性”的增量迭代。

---

## 2. 本轮目标

建立面向目标系统的测试用例完整性框架，使 Harness 从“有一些任务”升级到“有可持续扩展的覆盖地图与任务集”。

本轮不要求一次性补齐所有任务实现，但必须至少完成：

1. 目标系统测试覆盖地图
2. 覆盖优先级分层
3. 缺口识别
4. 最小必备任务清单
5. 可执行的后续补齐顺序

---

## 3. 本轮范围

### 3.1 建立目标系统测试覆盖地图
至少覆盖以下一级域：

- CLI contract
- Session / thread / continue / attach
- Workspace / filesystem / Git side effects
- Permissions / approvals / policy semantics
- Serve / API / OpenAPI contract
- Models / providers / auth
- Tool orchestration
- Error handling / recovery
- Export / import / stats / project / config 类辅助命令
- Regression / incident cases

### 3.2 为每个域建立覆盖分层
每个域至少拆为：

- smoke
- core
- regression
- edge / recovery

### 3.3 标出当前已有覆盖与缺失覆盖
必须明确区分：

- 已存在并可执行的 task
- 已存在但不可信/不稳定的 task
- 尚未存在的 task
- 当前不应优先覆盖的 task

### 3.4 形成“最小完整集”定义
至少定义出一个**目标系统最小完整测试集**，用于后续迭代。

这个最小完整集必须能支撑：

- 差异发现
- PR / hourly / manual follow-up
- regression 沉淀
- smoke gate 的基础输入

---

## 4. 明确不做

本轮不做：

- 不要求实现所有缺失 task
- 不要求一次性打通 web / desktop / ACP 全量验证
- 不在 Harness 仓库中修复目标系统产品问题
- 不以任务数量为目标做表面堆砌
- 不为了“看起来完整”引入大量低价值 case

---

## 5. 设计要求

### 5.1 完整性优先于数量
关键不是任务文件越多越好，而是：

- 覆盖面是否成体系
- 高价值路径是否被纳入
- 缺口是否明确
- 后续迭代顺序是否可执行

### 5.2 以目标系统真实价值为中心
测试用例完整性必须围绕 opencode / opencode-rs 的高价值行为，而不是围绕实现细节或低价值命令堆数量。

### 5.3 必须区分“存在”与“可用”
有 task 文件，不等于这个覆盖点已经真正建立。

覆盖地图中必须区分：

- 文件存在
- 能跑
- 能稳定跑
- verdict 可信
- 能持续回归

### 5.4 输出要能指导下一轮真实开发
本轮产物必须能直接作为后续迭代输入，告诉系统：

- 先补哪里
- 为什么先补
- 哪些 case 属于 P0 / P1 / P2

---

## 6. 必须产物

本轮结束后，至少应产出：

1. 一份覆盖地图文档
2. 一份当前覆盖差距分析
3. 一份最小完整测试集定义
4. 一份按优先级排序的补齐任务清单
5. 必要时补充新的任务骨架文件或目录规范

---

## 7. 验收标准

### 7.1 最低验收
满足以下条件才算完成：

1. 明确列出目标系统一级测试域
2. 明确每个域下当前已有覆盖 / 缺失覆盖
3. 明确最小完整测试集定义
4. 明确后续补齐顺序
5. 输出内容能直接驱动下一轮迭代开发

### 7.2 更优验收
如能额外做到以下更好：

- 直接补入一批高优先级 task skeleton
- 将已有任务与覆盖地图自动关联
- 输出适合 suite / gate 接入的覆盖摘要

---

## 8. 验收命令

本轮重点是文档、结构与任务规划，验收可结合：

```bash
find harness/tasks -maxdepth 3 -type f | sort
cargo run -- run --task harness/tasks/cli/SMOKE-CLI-001.yaml
cargo run -- run --task harness/tasks/cli/SMOKE-CLI-002.yaml
cargo run -- run --task harness/tasks/cli/SMOKE-CLI-003.yaml
cargo run -- run --task harness/tasks/cli/SMOKE-CLI-004.yaml
```

必要时可补充统计脚本，用于对任务分布进行汇总。

---

## 9. 下一轮输入

若本轮完成，后续可直接进入：

1. 按覆盖地图逐域补齐 task
2. 先补 P0 最小完整集
3. 再补 regression / recovery / edge cases
4. 最终把 coverage summary 接入 suite / gate / report

---

## 10. 职责边界

再次强调：

- 本轮目标是补齐 Harness 的测试用例完整性框架
- 不是在 Harness 仓库中实现目标系统产品功能
- 不是为了制造大量 task 文件而牺牲价值密度
- Harness 的职责是构建真实、分层、可持续的验证覆盖体系
