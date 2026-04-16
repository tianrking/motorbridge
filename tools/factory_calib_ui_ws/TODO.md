# factory_calib_ui_ws TODO

> 当前阶段：仅记录待办，不立即实施重构。

## P0 低风险先做

- [ ] 删除重复 URDF 资源目录（二选一保留）
  - `public/resources/arm01/`
  - `public/urdf/reBot-DevArm_description_fixend/`
  - 目标：去除字节级重复资源，减小体积与维护成本

- [ ] 提取通用基础能力（机械抽离）
  - [ ] `usePersistedState(key, defaultValue)`（替代多处 localStorage effect）
  - [ ] `<ProgressBar />` 组件（统一进度条 JSX）
  - [ ] `<CollapsibleSection />` 组件（统一可折叠区域）
  - [ ] `bulkOp(rows, fn, gapMs)` 工具（统一批量 enable/disable/zero）

- [ ] 加入 Error Boundary（防止组件异常导致整页白屏）

## P1 核心结构重构

- [ ] 拆分 `src/hooks/useMotorStudio.js`（当前 God Hook）
  - [ ] `useConnectionState`：WS 连接与配置
  - [ ] `useScanState`：扫描相关状态与流程
  - [ ] `useMotorControl`：电机控制操作
  - [ ] `useRobotArm`：机械臂批量流程
  - [ ] `usePreferences`：UI 偏好与持久化
  - [ ] 引入 Context，减少 App 层 60+ props 透传

- [ ] 拆分 `src/components/RobotArmPage.jsx`（当前巨型组件）
  - [ ] `JointList`
  - [ ] `JointControlPanel`
  - [ ] `ParamTable`
  - [ ] `SelfCheckReport`
  - [ ] `ZeroConfirmDialog`

## P2 可维护性收口

- [ ] 硬编码常量集中管理
  - [ ] WS URL 默认值（`ws://127.0.0.1:9002`）
  - [ ] 默认通道（`can0`）
  - [ ] 前端端口（`18110`）
  - [ ] 超时常量（3000~12000）
  - [ ] 达妙寄存器 ID（10, 24-28）
  - [ ] 关节限位配置迁移到独立模块（如 `robotArm.js`）

- [ ] Vendor 逻辑聚合
  - [ ] 把达妙/RobStride 逻辑从页面与 hook 内联 if 分支抽到 vendor 模块
  - [ ] 统一模型候选、扫描 payload、寄存器映射

## P3 工程化增强（后置）

- [ ] TypeScript 迁移（逐步）
- [ ] 测试基建（Vitest/Jest + 核心 hook/组件测试）
- [ ] ESLint + Prettier 统一规范

## 执行顺序（建议）

1. 删除重复资源（零风险）
2. 公共组件/工具抽离（低风险）
3. `useMotorStudio` 拆分（高影响）
4. `RobotArmPage` 拆分（高影响）
5. 常量与 vendor 聚合（中风险）
6. TypeScript/测试/规范化（后置）

