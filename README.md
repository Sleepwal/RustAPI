# API Client

一个使用 Rust 和 egui 构建的简单 GUI API 客户端。

## 功能特性

- **HTTP 方法支持**: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
- **自定义请求头**: 轻松添加、编辑和删除请求头
- **请求体编辑**: 支持 POST/PUT/PATCH 方法的请求体，带 JSON 格式化功能
- **响应查看**: 清晰显示响应状态码、响应时间和响应体
- **响应头查看**: 独立的标签页查看响应头信息
- **JSON 格式化**: 自动美化 JSON 响应和请求体

## 截图

（待添加）

## 快速开始

### 环境要求

- [Rust](https://rustup.rs/) (1.70+)

### 运行

```bash
# 克隆项目
git clone <repository-url>
cd rust_api

# 运行
cargo run

# 构建发布版本
cargo build --release
```

### 使用说明

1. 从下拉菜单选择 HTTP 方法
2. 在 URL 输入框中输入目标地址
3. 点击 "Headers" 折叠面板添加自定义请求头（可选）
4. 对于 POST/PUT/PATCH 请求，点击 "Body" 折叠面板添加请求体
5. 点击 "Send" 按钮发送请求
6. 在下方查看响应结果，可在 "Body" 和 "Headers" 标签页之间切换

## 技术栈

- [egui](https://github.com/emilk/egui) - 即时模式 GUI 框架
- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) - egui 的框架支持
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP 客户端
- [tokio](https://tokio.rs/) - 异步运行时
- [serde](https://serde.rs/) - 序列化/反序列化

## 项目结构

```
src/
├── main.rs          # 程序入口
├── app.rs           # 应用状态和逻辑
├── http.rs          # HTTP 请求处理
├── models.rs        # 数据模型（请求、响应、HTTP 方法等）
└── ui/
    ├── mod.rs       # UI 模块
    ├── request_panel.rs   # 请求面板 UI
    └── response_panel.rs  # 响应面板 UI
```

## 许可证

本项目使用 [MIT 许可证](LICENSE)。

## 贡献

欢迎提交 Issue 和 Pull Request！
