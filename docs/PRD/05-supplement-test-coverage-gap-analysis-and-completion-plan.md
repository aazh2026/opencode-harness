# opencode-harness 增量 PRD：测试用例完整性缺口分析与补齐计划

## 1. 背景

当前 `opencode-harness` 已具备最小真实差分执行能力，并且已经在 CLI smoke suite 中成功暴露 `opencode` 与 `opencode-rs` 的真实差异。但从“目标系统测试用例完整性”角度看，现有任务集仍明显偏向最小 smoke 和初期骨架，尚未形成足够完整、可持续扩展的覆盖体系。

当前 `harness/tasks` 中实际已有任务数量如下：

- `api`: 10
- `cli`: 6
- `permissions`: 4
- `recovery`: 3
- `session`: 4
- `workspace`: 5
- `web`: 0

总的来看，现状是：

1. 已有 CLI / API / session / workspace / permissions / recovery 的最小 smoke 基础
2. Web 方向完全空缺
3. 回归用例、edge case、负向用例、跨域组合用例明显不足
4. 许多任务仍更接近“任务骨架”而非系统性覆盖地图的一部分
5. 测试集还不足以称为“目标系统的最小完整验证集”

因此，需要单独开一轮 Harness 增量迭代，专门解决：

> **测试覆盖地图不完整、覆盖分层不清、任务缺口未系统化定义**

---

## 2. 本轮目标

建立一份真实可执行的测试覆盖缺口分析，并据此补齐 Harness 的最小完整测试集框架，使后续迭代不再靠零散补 task，而是沿着明确的覆盖地图推进。

本轮目标分三层：

1. 明确目标系统应覆盖的测试域
2. 明确当前已覆盖 / 未覆盖 / 覆盖不足的部分
3. 产出可直接执行的补齐计划与优先级顺序

---

## 3. 覆盖现状分析

### 3.1 当前已具备的覆盖域

#### A. CLI 基础 smoke
已有：
- `--help`
- `--version`
- `workspace --help`
- `--invalid-option`
- `config show`
- `--verbose --help`

现状评价：
- 已能暴露真实 CLI contract 差异
- 但仅覆盖基础命令层，远未完整

#### B. API 基础 smoke
已有 10 个 API smoke 任务。

现状评价：
- 已开始覆盖 server/API 方向
- 但很多测试仍依赖运行中的 API server，当前更多是结构具备，长期稳定性与真实双边验证仍待做实

#### C. Session 基础 smoke
已有 4 个 session 方向任务。

现状评价：
- 方向正确
- 覆盖深度不够，尤其缺少 thread/continue/attach/reconnect 的组合路径

#### D. Workspace 基础 smoke
已有 5 个 workspace 方向任务。

现状评价：
- 能开始暴露 workspace command contract 差异
- 但 filesystem side effects / git 副作用 / init 之后状态验证明显不足

#### E. Permissions
已有 4 个 permissions 方向任务。

现状评价：
- 具备最小骨架
- 但 approval flow / deny / allow / external directory 等高价值路径覆盖不足

#### F. Recovery
已有 3 个 recovery 方向任务。

现状评价：
- 已开始触及恢复能力
- 但断连、重试、部分状态恢复、幂等性等仍远不足够

---

## 4. 主要缺口

### 4.1 Web 测试完全空缺
当前 `web` 目录为 0。

这是明显缺口，因为目标系统后续需要验证：
- web 启动能力
- web 入口是否可达
- 基础交互 smoke
- server + web 协同路径

### 4.2 CLI 覆盖仍过浅
当前 CLI 只覆盖了顶层少量命令，仍缺少：
- `models`
- `providers`
- `run`
- `serve`
- `agent`
- `plugin`
- `completion`
- `project`
- `files`
- `prompt`
- `quick`
- `stats`
- `export/import`
- `attach`
- `thread`
- `workspace` 更多子命令

### 4.3 Session / Workspace 缺少组合路径
当前更偏单命令 smoke，缺少：
- session start → continue → attach
- workspace init → file mutation → state verification
- session 与 workspace 副作用联合验证

### 4.4 Permissions 覆盖未触及高价值语义边界
仍缺少：
- allow / deny 的具体行为差异
- 外部目录权限
- 自动批准与显式批准路径
- 多轮权限状态变化

### 4.5 API 覆盖缺少 contract 深化
当前已有数量，但仍缺：
- OpenAPI schema 级验证
- 错误响应一致性
- 认证/授权边界
- 长连接 / streaming / event ordering

### 4.6 Regression / Incident 资产明显不足
当前差异虽然已发现，但还没有系统地沉淀成：
- regression cases
- known mismatch catalog
- incident-based fixtures

### 4.7 Edge / Negative / Recovery case 比例不足
目前任务主要还是 smoke 风格，缺少：
- 错误输入
- 边界条件
- 恢复与重入
- 部分成功 / 部分失败场景

---

## 5. 最小完整测试集定义（建议）

为了让 Harness 从“能跑一些任务”升级为“具备最小完整验证能力”，建议先补齐以下最小完整集。

### Tier 1 - 必须先补齐

#### CLI
- 顶层帮助 / 版本 / 错误参数
- `models`
- `providers`
- `config`
- `workspace`
- `session`
- `run`
- `serve`

#### Session / Workspace
- start / continue / attach
- init / status / mutation
- file system side effects

#### Permissions
- allow
- deny
- external directory
- approval transitions

#### API
- 基础健康检查
- 关键资源 CRUD
- tools endpoints
- events / subscribe

### Tier 2 - 很快补
- recovery
- export/import
- stats
- project/files/prompt/quick
- plugin/completion

### Tier 3 - 后续补
- web smoke
- streaming/event ordering
- regression corpus
- edge / incident / recovery 深化

---

## 6. 本轮范围

本轮至少要完成：

1. 一份系统化覆盖地图
2. 一份覆盖缺口分析
3. 一份最小完整测试集定义
4. 一份按 P0 / P1 / P2 排序的补齐任务清单
5. 必要时补入部分高优先级 task skeleton

---

## 7. 明确不做

本轮不要求：
- 一次性实现所有缺失 task
- 一次性跑通所有新 task
- 在 Harness 中修复 opencode-rs 产品行为
- 为了数量好看堆大量低价值任务

---

## 8. 设计要求

### 8.1 完整性优先于任务数量
重点不是生成更多 YAML，而是形成真正可执行的覆盖框架。

### 8.2 必须区分“有文件”和“可用覆盖”
有 task 文件不等于覆盖完成，至少要明确：
- 文件存在
- 可执行
- verdict 可信
- 可回归

### 8.3 后续补齐顺序要可直接驱动迭代
产出必须能直接作为下一轮 Harness 迭代的输入，而不是停留在分析层。

---

## 9. 必须产物

本轮完成后至少要有：

1. 覆盖地图文档
2. 覆盖缺口分析
3. 最小完整测试集定义
4. P0/P1/P2 补齐清单
5. 如有必要，一批高优先级 task skeleton

---

## 10. 验收标准

### 最低验收
满足以下条件即算完成：

1. 明确列出目标系统的一级测试域
2. 明确每个域的现有覆盖与主要缺口
3. 定义最小完整测试集
4. 定义后续补齐优先级顺序
5. 产物可直接驱动下一轮 Harness 增量迭代

### 更优验收
- 直接补入若干 P0 任务骨架
- 输出面向 suite/gate 的 coverage summary
- 为 web / plugin / completion / export/import 等缺口建立初始结构

---

## 11. 验收命令

```bash
find harness/tasks -maxdepth 2 -type f | sort
cargo run -- run --task harness/tasks/cli
cargo run -- run --task harness/tasks/session
cargo run -- run --task harness/tasks/workspace
```

必要时增加统计脚本，用于任务覆盖汇总。

---

## 12. 一句话

本轮的目的不是再零散补几个 task，而是：

> **把 opencode-harness 的测试用例覆盖，从“已有一批 smoke/task 骨架”提升为“有覆盖地图、有最小完整集、有补齐顺序的系统化验证框架”。**
