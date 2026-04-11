# Git Commit Message 规范

## 概述

本项目遵循 [Conventional Commits](https://www.conventionalcommits.org/zh-hans/v1.0.0/) 规范，用于创建清晰、一致的 Git 提交信息。

## 提交信息格式

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### 基本规则

1. **标题行**：`<type>: <description>`（必须）
2. **空行**：标题行和正文之间必须有空行
3. **正文**：详细说明改动原因和方式（可选）
4. **脚注**：BREAKING CHANGE 或引用 Issue（可选）

## Type 类型

| Type | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | `feat: add user authentication` |
| `fix` | Bug 修复 | `fix: resolve login timeout issue` |
| `docs` | 文档更新 | `docs: update API documentation` |
| `style` | 代码格式（不影响代码运行） | `style: format code with rustfmt` |
| `refactor` | 代码重构 | `refactor: simplify request handler` |
| `perf` | 性能优化 | `perf: optimize database query` |
| `test` | 测试相关 | `test: add unit tests for auth module` |
| `build` | 构建系统或外部依赖 | `build: update tokio to v1.0` |
| `ci` | CI/CD 配置 | `ci: add GitHub Actions workflow` |
| `chore` | 其他不修改源代码的改动 | `chore: update .gitignore` |
| `revert` | 撤销之前的提交 | `revert: feat: add user authentication` |

## Scope 范围（可选）

用于标明改动影响的模块或功能范围：

```
feat(auth): add JWT token validation
fix(ui): resolve button alignment issue
perf(api): optimize response serialization
```

## Description 描述

- 使用祈使句（"add" 而不是 "added" 或 "adds"）
- 不要大写首字母
- 不要在末尾加句号
- 描述"做了什么"，而不是"怎么做"
- 长度不超过 72 个字符

## Body 正文（可选）

- 详细说明改动的动机和实现方式
- 每行不超过 72 个字符
- 使用中文或英文均可

## Footer 脚注（可选）

### BREAKING CHANGE

```
feat: change API endpoint structure

BREAKING CHANGE: /api/v1/users 已迁移至 /api/v2/users
```

### 引用 Issue

```
fix: resolve memory leak in request handler

Closes #123
```

## 完整示例

### 简单示例
```
feat: add request history panel with save, load, and clear functionality
```

```
fix: resolve ScrollArea ID conflict warning in response panel
```

### 复杂示例
```
feat(auth): add JWT token validation

Implement JWT-based authentication to replace the current
session-based approach. This improves security and enables
stateless API access.

BREAKING CHANGE: Authentication header format changed
from `Authorization: Session <id>` to `Authorization: Bearer <token>`

Closes #45
```

## 详细版提交信息规范

对于重要的性能优化、架构重构或涉及多个模块的重大改动,推荐使用详细版提交信息格式,以便团队成员和未来的自己快速理解变更内容。

### 何时使用详细版

建议在以下情况使用详细版提交信息:

- **性能优化**: 涉及多处性能改进的提交
- **架构调整**: 修改核心数据结构或模块职责
- **多文件变更**: 同时修改 5 个以上文件
- **重要功能**: 添加关键功能或破坏性变更
- **复杂修复**: 涉及多个关联问题的修复

### 详细版格式

```
<type>: <description>

【核心优化】(或【新功能】/【Bug 修复】/【架构调整】等分类标题)

1. 优化点一 (影响类别)
   - 具体改动说明
   - 实现方式或策略
   - 预期效果或收益

2. 优化点二 (影响类别)
   - 具体改动说明
   - 实现方式或策略
   - 预期效果或收益

【代码质量】(可选)

- 代码检查工具验证结果
- 测试覆盖情况
- 文档更新状态

【影响范围】

修改文件:
  - 文件名: 具体改动内容
  - 文件名: 具体改动内容
```

### 详细版示例

```
perf: 优化 HTTP 客户端复用、历史记录限制、URL 验证和响应数据共享

【核心优化】

1. HTTP 客户端复用 (性能提升)
   - 在 ApiClientApp 结构体中持有 reqwest::Client 实例
   - 避免每次请求都创建新客户端,减少连接池重建开销
   - 修改 send_request 函数签名,接收 &Client 参数

2. 历史记录数量限制 (内存保护)
   - 添加 MAX_HISTORY_SIZE 常量 (100 条)
   - 实现 FIFO 策略自动移除最旧记录
   - 为 RequestHistory 添加 add/clear/len/is_empty 方法
   - 防止无限增长导致内存泄漏

3. URL 验证与自动补全 (健壮性提升)
   - 添加 ApiRequest::validate_and_normalize_url 方法
   - 验证 URL 非空且格式合法
   - 自动补全缺失的 https:// 协议前缀
   - 添加 url crate 依赖用于格式验证

4. Arc 共享响应数据 (减少 clone 开销)
   - ApiResponse.headers 改为 Arc<Vec<(String, String)>>
   - HistoryItem.response 改为 Option<Arc<ApiResponse>>
   - 避免历史记录和 UI 渲染时的大型数据 clone
   - 显著降低内存分配和拷贝开销

【代码质量】

- 修复所有 Clippy 警告 (collapsible_if, manual_range_contains, doc_overindented_list_items)
- cargo check 编译通过
- cargo clippy -- -D warnings 零警告

【影响范围】

修改文件:
  - Cargo.toml: 添加 url 依赖
  - Cargo.lock: 更新依赖树
  - src/app.rs: 添加 http_client 字段,更新请求/历史逻辑
  - src/http.rs: 修改函数签名,使用 Arc 包装 headers
  - src/models.rs: 添加验证/Arc/历史记录管理
  - src/ui/history_panel.rs: 使用新的历史 API
  - src/ui/response_panel.rs: 优化数据访问模式
```

### 详细版编写建议

1. **分类标题**: 使用中文方括号【】标注,突出改动的主题
2. **编号列表**: 使用阿拉伯数字编号,每条优化点独立说明
3. **影响类别**: 在圆括号中标注 (性能提升)/(内存保护)/(健壮性提升) 等
4. **具体描述**: 使用短句,每行不超过 72 个字符
5. **技术细节**: 包含关键类型、函数名、常量名等,便于代码审查
6. **验证结果**: 列出通过的检查和测试,增强可信度的同时记录

### 简化版 vs 详细版选择指南

| 场景 | 推荐格式 | 示例 |
|------|----------|------|
| 小功能/小修复 | 简单版 | `fix: resolve button alignment issue` |
| 单模块改动 | 简单版 + Body | `feat: add history panel\n\nImplement...` |
| 多模块性能优化 | **详细版** | 见上方示例 |
| 架构重构 | **详细版** | 包含改动前后对比 |
| 重大功能添加 | **详细版** | 包含设计决策说明 |
| 复杂 Bug 修复 | **详细版** | 包含根因分析和修复策略 |

## 使用建议

1. **提交前检查**：
   - 运行 `git status` 确认改动的文件
   - 运行 `git diff HEAD` 查看所有变更
   - 确认提交信息符合规范

2. **原子提交**：
   - 每个提交只做一件事
   - 相关的改动可以放在一起
   - 避免"大杂烩"式的提交

3. **中文 vs 英文**：
   - type 和 scope 使用英文
   - description 可以使用中文或英文
   - 保持项目内的一致性

## 工具集成

### 使用 commitlint（可选）

可以配置 commitlint 自动检查提交信息：

```bash
# 安装
npm install --save-dev @commitlint/cli @commitlint/config-conventional

# 配置 commitlint.config.js
module.exports = {
  extends: ['@commitlint/config-conventional']
}
```

### Git Hook 配置

使用 husky 在提交前自动检查：

```bash
npx husky add .husky/commit-msg 'npx --no -- commitlint --edit $1'
```

## 参考资源

- [Conventional Commits 官方文档](https://www.conventionalcommits.org/)
- [Angular 提交信息规范](https://github.com/angular/angular/blob/main/CONTRIBUTING.md#commit)
- [Git 提交信息最佳实践](https://cbea.ms/git-commit/)
