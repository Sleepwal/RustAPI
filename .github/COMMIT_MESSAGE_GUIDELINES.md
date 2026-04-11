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

## 更新日志规范

对于涉及多个文件、功能复杂或需要详细记录的改动,推荐在提交信息中使用简洁版,同时将详细信息记录在更新日志中。

### 更新日志位置

项目根目录的 `CHANGELOG.md` 文件。

### 更新日志格式

```markdown
## [Unreleased]

### YYYY-MM-DD - 版本描述或功能分类

#### 新增
- 功能 A 的详细描述
- 功能 B 的详细描述

#### 优化
- 性能优化点 1 的具体实现和影响
- 性能优化点 2 的具体实现和收益

#### 修复
- Bug 修复的详细描述和根因分析

#### 变更
- 破坏性变更或重要架构调整说明
```

### 提交信息 vs 更新日志

| 内容 | 位置 | 详细程度 |
|------|------|----------|
| 提交标题 | Git 提交信息 | 简洁,一句话概括 |
| 改动摘要 | Git 提交信息(可选 Body) | 中等,2-3 句说明 |
| 技术细节 | CHANGELOG.md | 详细,包含实现策略、影响范围、验证结果 |
| 代码级说明 | 代码注释/文档 | 非常详细,包含示例和使用说明 |

### 工作流程

1. **提交代码时**: 使用简洁版提交信息
   ```
   perf: 优化 HTTP 客户端复用和响应数据共享
   ```

2. **更新 CHANGELOG.md**: 在同一 PR 或提交中添加详细记录
   ```markdown
   ## [Unreleased]

   ### 2026-04-11 - 性能优化

   #### 优化
   - HTTP 客户端复用: 在 ApiClientApp 中持有 reqwest::Client 实例,
     避免每次请求创建新客户端,减少连接池重建开销
   - Arc 共享响应数据: ApiResponse.headers 改为 Arc 包装,
     HistoryItem.response 改为 Option<Arc<ApiResponse>>,
     避免大型数据 clone,降低内存分配和拷贝开销
   - 历史记录数量限制: 添加 MAX_HISTORY_SIZE=100 限制,
     使用 FIFO 策略自动移除最旧记录,防止内存泄漏

   #### 代码质量
   - 修复所有 Clippy 警告
   - 添加 url crate 依赖用于 URL 格式验证
   ```

3. **代码审查时**: 审查者可以同时查看 Git 提交信息和 CHANGELOG.md,
   获取完整的上下文信息。

### 提交信息示例

**简单改动**:
```
fix: resolve button alignment issue
```

**中等复杂度**:
```
feat: add request history panel

Implement scrollable history list with save, load, and clear functionality.
Display method colors and relative timestamps.
```

**复杂改动(简洁版)**:
```
perf: optimize HTTP client reuse, history limits, and Arc sharing

See CHANGELOG.md for detailed optimization descriptions.
```

### 更新日志编写建议

1. **分组清晰**: 使用 新增/优化/修复/变更 等分类
2. **技术细节**: 包含具体的类型名、函数名、常量名
3. **影响说明**: 说明改动对性能、内存、用户体验的影响
4. **验证结果**: 列出通过的检查和测试
5. **更新时机**: 在功能开发完毕、准备提交时同步更新

### 何时使用更新日志

建议在以下情况使用更新日志:

- **性能优化**: 多处性能改进,需要详细说明策略和收益
- **架构调整**: 修改核心数据结构,需要说明设计决策
- **多文件变更**: 涉及 5 个以上文件的复杂改动
- **重要功能**: 需要详细说明使用方法和注意事项
- **复杂修复**: 需要说明根因分析和多种修复策略

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
