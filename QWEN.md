# QWEN.md - API Client 项目上下文

## 项目概述

**rust_api** 是一个使用 Rust 和 egui 构建的 GUI API 客户端工具，类似于轻量级的 Postman。它提供了一个图形化界面，用于发送 HTTP 请求并查看响应结果。

### 核心功能

- 支持 7 种 HTTP 方法：GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
- 自定义请求头编辑（添加、编辑、删除）
- 请求体编辑（POST/PUT/PATCH），支持 JSON 格式化
- 响应查看：状态码（带颜色标识）、响应时间、响应体
- 响应头查看（独立的标签页）
- JSON 自动美化格式化

### 技术栈

- **GUI 框架**: egui 0.28 + eframe 0.28（即时模式 GUI 框架）
- **HTTP 客户端**: reqwest 0.12（带 json 特性）
- **异步运行时**: tokio 1.x（full 特性）
- **序列化**: serde 1.x + serde_json 1.x
- **异步轮询**: poll-promise 0.3（带 tokio 特性）

### 架构设计

```
┌──────────────┐     ┌─────────────────┐     ┌──────────────┐
│  main.rs     │────>│  ApiClientApp   │────>│  http.rs     │
│  (入口)      │     │  (应用状态)     │     │  (HTTP发送)  │
└──────────────┘     └────────┬────────┘     └──────────────┘
                              │
                     ┌────────┴────────┐
                     │   ui/           │
                     │  (界面渲染)     │
                     └─────────────────┘
```

### 模块说明

| 模块 | 文件 | 职责 |
|------|------|------|
| 入口 | `src/main.rs` | 程序入口，配置窗口大小，启动 eframe |
| 应用核心 | `src/app.rs` | 应用状态管理、异步请求调度、UI 协调 |
| HTTP | `src/http.rs` | HTTP 请求发送与响应处理 |
| 模型 | `src/models.rs` | 数据结构定义（HttpMethod, ApiRequest, ApiResponse 等） |
| UI | `src/ui/mod.rs` | UI 模块声明 |
| 请求面板 | `src/ui/request_panel.rs` | 请求配置面板渲染 |
| 响应面板 | `src/ui/response_panel.rs` | 响应展示面板渲染 |

## 构建与运行

### 环境要求

- Rust 1.70+

### 常用命令

```bash
# 运行开发版本
cargo run

# 构建发布版本
cargo build --release

# 运行测试（如有）
cargo test

# 检查代码（编译检查）
cargo check

# 代码格式化
cargo fmt

# 代码检查（clippy）
cargo clippy
```

### 窗口配置

- 默认窗口大小：900×700 像素
- 最小窗口大小：600×400 像素

## 开发规范

### 代码风格

- 所有模块和公共结构体/枚举都有完整的文档注释
- 使用函数式 UI 渲染模式（接收 `&mut ApiClientApp` 和 `&mut Ui` 参数）
- 模块职责清晰，每个子模块负责一个独立的 UI 区域
- 使用 Rust 2024 edition

### 异步请求机制

采用 `poll-promise` 实现非阻塞异步请求：

1. UI 线程调用 `send_request()` 将异步任务提交到 tokio 运行时
2. 每帧调用 `check_response()` 轮询 Promise 是否完成
3. 请求进行中，每 100ms 请求一次重绘以检测完成状态
4. 请求完成后更新响应/错误消息，清除 Promise

### 数据结构

- **HttpMethod**: HTTP 方法枚举（7 种方法）
- **ApiRequest**: 请求配置（method, url, headers, body）
- **Header**: 请求头键值对
- **ApiResponse**: 响应数据（status, status_text, headers, body, duration_ms）
- **RequestHistory**: 请求历史记录容器
- **HistoryItem**: 单条历史记录条目
- **ResponseTab**: 响应面板标签页枚举（Body, Headers）

### 关键不变量

- `url` 在发送请求前不应为空（由 `send_request` 方法校验）
- `body` 仅在 POST/PUT/PATCH 方法时有效
- 请求头 `key` 不应为空字符串（发送时自动跳过空 key）
- Content-Type 自动检测：未显式设置时，JSON 请求体自动设置 `application/json`

## 项目状态

当前版本：0.1.0
Edition：2024

### 待完善功能

- 请求历史记录 UI 展示（数据结构已定义但未在 UI 中展示）
- 截图文档（README 中标记为"待添加"）

## 许可证

MIT 许可证
