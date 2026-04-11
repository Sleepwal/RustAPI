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
