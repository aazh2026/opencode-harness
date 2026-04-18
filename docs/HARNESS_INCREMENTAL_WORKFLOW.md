# opencode-harness 增量分析 / PRD / 迭代工作流

## 目标

让 `opencode-harness` 不只是零散补 task，而是形成稳定闭环：

> 盘点测试覆盖现状 → 分析测试缺口 → 形成 Harness 增量 PRD → 启动 Harness 增量迭代 → 提升测试能力 → 再去测试 `opencode-rs`

---

## 标准流程

### 1. 先盘点 Harness 当前测试覆盖现状
先看当前 `harness/tasks/` 下到底已有多少真实覆盖，而不是拍脑袋。

至少要检查：
- 当前有哪些测试域（cli/api/session/workspace/permissions/recovery/web 等）
- 每个域各有多少 task
- 哪些只是 smoke
- 哪些只是任务骨架
- 哪些域完全空缺

本阶段的目标是回答：

> Harness 现在到底覆盖了什么？

---

### 2. 做测试用例完整性缺口分析
不是只数 task 文件，而是分析结构性缺口。

至少要分析：
- 哪些域完全没覆盖
- 哪些域只有最小 smoke，没有 deeper coverage
- 哪些域缺少 edge / negative / regression / recovery case
- 哪些域缺少跨域组合路径（如 session + workspace，API + events，permissions + filesystem）
- 哪些已有 task 只是“存在”，但还不能算“可信可用覆盖”

本阶段的目标是回答：

> Harness 距离最小完整测试集还差哪些块？

---

### 3. 基于缺口分析生成 Harness 自己的增量 PRD
不要直接零散补 task，而是先把缺口分析收敛成一轮可执行 PRD。

PRD 至少应包含：
- 背景
- 本轮目标
- 本轮范围
- 明确不做
- 设计要求
- 必须产物
- 验收标准
- 验收命令
- 下一轮输入

PRD 文件位置通常在：

```text
/Users/openclaw/Documents/github/opencode-harness/docs/PRD/
```

本阶段目标是：

> 把“测试缺口分析”变成一轮可直接迭代执行的增量需求。

---

### 4. 用新的增量 PRD 启动 Harness 新一轮迭代
基于生成的 PRD 直接启动：

```bash
cd /Users/openclaw/Documents/github/opencode-harness
./iterate-prd.sh docs/PRD/<新增PRD文件名>.md
```

本阶段目标是推动：
- 覆盖地图
- 最小完整测试集
- P0/P1/P2 补齐顺序
- 高优先级 task skeleton / coverage structure

---

### 5. 迭代过程中检查状态时，必须看最近代码修改时间
这条规则是硬要求。

每次检查 Harness 任务状态，都必须同时看：
- 最近 git commit 时间
- 关键源码文件 mtime
- 最新 `tasks_v*.json` 更新时间
- 迭代日志更新时间

#### 判定原则
- 单次 `gap-analysis.md` 首次生成后触发一次重试，不一定是故障
- 如果长时间没有代码更新、任务文件不更新、日志无推进，则视为卡住
- 一旦判定卡住，必须自动重启当前任务，而不是只做观察汇报

---

### 6. Harness 迭代完成后，进入下一层用途
Harness 自己补齐能力后，不是结束，而是进入下一层：

1. 用更完整的测试能力去真实测试 `opencode-rs`
2. 汇总差异
3. 为 `opencode-rs` 生成新的增量 PRD
4. 驱动 `opencode-rs` 继续做增量迭代

也就是说，Harness 的增强最终目的是：

> 更准确、更系统地驱动 `opencode-rs` 的后续修复和对齐。

---

## 一句话流程图

> 盘点覆盖 → 分析缺口 → 生成 Harness 增量 PRD → 启动 Harness 迭代 → 提升覆盖能力 → 用 Harness 去测 `opencode-rs`

---

## 与 `opencode-rs` 工作流的区别

### Harness 工作流关注的是：
- 测试覆盖是否完整
- 测试能力还缺什么
- 如何补齐 task / suite / coverage 结构

### `opencode-rs` 工作流关注的是：
- 被 Harness 测出来的差异如何修复
- 如何基于差异生成增量 PRD
- 如何继续做实现迭代

---

## 一句话总结

`opencode-harness` 的正确工作流不是“看到差异就零散补 task”，而是：

> **先分析测试覆盖完整性，基于分析形成 Harness 增量 PRD，启动 Harness 自己的新一轮迭代，把测试能力补齐后，再用这套更强的测试能力去测试和驱动 `opencode-rs`。**
