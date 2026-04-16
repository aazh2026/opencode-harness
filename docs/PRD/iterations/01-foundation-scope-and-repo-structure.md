# Iteration 1: Foundation, Scope and Repo Structure

## 本轮目标
建立 opencode-harness 的最小可运行工程骨架，并把后续真实运行会踩到的状态、日志、环境探测、命名约定提前定死，避免像 opencode-rs 那样在后期因为状态混乱和工件失控而返工。

## 前置依赖
- 已有主 PRD
- 已确认本项目优先服务 opencode-rs 对齐工作，而非追求过早平台化

## 本轮范围
本轮只处理：
- 仓库结构与模块边界
- 最小工程入口
- 基础配置加载模型
- 目录约定与文档骨架
- 日志、工件、任务状态的基础约定
- 环境探测接口占位

## 职责边界硬约束
- 本仓库只开发 opencode-harness 自身能力
- 本仓库禁止实现、补完或替代 opencode-rs 的产品功能
- 若发现 opencode-rs 与 opencode 不一致，只记录为 mismatch、contract gap、regression candidate 或 report item
- 不得把“修复 opencode-rs”当作本轮任务

## 明确不做
本轮不要做：
- 真正的 legacy runner / rust runner
- 真正的 differential execution
- 真正的 golden / regression 录制
- 真正的 comparator / verifier
- 真正的 CI gate
- 复杂 UI / web / desktop 自动化

## 推荐实现顺序
1. 先确定 Rust workspace / crate 结构
2. 再建立最小 CLI 与基础配置
3. 再定义错误类型、路径约定、日志目录约定
4. 再定义任务状态、失败分类、环境探测接口占位
5. 最后补 README 与 architecture overview

## 设计要求
1. 项目必须保持“中立裁判”定位，不偏向任一实现仓。
2. 工程结构必须明确区分：资产、执行、比较、治理，核心项目目录统一收拢在 harness/ 下。
3. 所有后续模块都应能挂在清晰的 crate/package 边界下。
4. 从第一轮开始就支持真实世界状态：manual_check、blocked、skipped。

## 必须产物
至少产出并落盘以下内容：
- 根 README
- 工程根配置（固定为 Rust workspace）
- 最小 CLI 入口
- docs/architecture/overview.md
- 基础目录全部创建完成并有必要的占位文件
- 公共配置类型、路径约定类型、错误类型的初始定义
- 任务状态枚举与失败分类枚举
- 环境探测接口/类型占位
- 日志与工件命名约定文档

## 文件与命名约定
本轮必须定下：
- `artifacts/run-<id>/...`
- `sessions/iteration-<n>/...`
- `harness/reports/<suite>/<timestamp>.json`
- `harness/tasks/**/*.yaml|json`
- `harness/fixtures/projects/<name>/`

## 失败处理原则
- 如果基础工程无法 build，阻断本轮完成
- 如果外部二进制缺失，不应阻断本轮，因为本轮只定义探测能力，不要求调用

## 验收标准
1. 项目可以成功 build / test 最小骨架
2. 目录结构与 README 一致
3. 基础错误类型、配置类型、路径约定、任务状态已存在
4. 后续迭代无需再回头重建项目结构

## 验收命令
- cargo build
- cargo test
- cargo run -- --help

## 下一轮输入
下一轮将基于本轮骨架，开始定义：
- task schema
- fixture schema
- workspace 生命周期
- execution policy
