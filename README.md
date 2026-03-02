# codex-cli-zh 使用指南

<p align="center">
  <img src="./docs/images/ui-preview.png" alt="codex-cli-zh 界面预览" width="88%" />
</p>

`codex-cli-zh` 是 `openai/codex` 的中文增强版 Fork，当前版本基线对齐 `0.106.0`。

## 安装方式

### 方式 1：下载本仓库 Releases（推荐）

进入仓库 `Releases` 页面，下载对应系统文件：

- Windows x64：`codex-<tag>-windows-x86_64.zip`
- macOS Intel：`codex-<tag>-macos-x86_64.tar.gz`
- macOS Apple Silicon：`codex-<tag>-macos-aarch64.tar.gz`
- Linux x64：`codex-<tag>-linux-x86_64.tar.gz`
- Linux arm64：`codex-<tag>-linux-aarch64.tar.gz`

解压后：

- Windows 执行 `codex.exe`
- macOS/Linux 执行 `./codex`

### 方式 2：源码编译

```powershell
cargo build --release --locked --manifest-path codex-rs/Cargo.toml -p codex-cli --bin codex
```

编译产物路径：

- Windows：`codex-rs/target/release/codex.exe`
- macOS/Linux：`codex-rs/target/release/codex`

## 全局使用

### Windows

1. 新建目录（示例）：`C:\Tools\codex`
2. 把 `codex.exe` 放入该目录
3. 将 `C:\Tools\codex` 加入系统 `PATH`
4. 重新打开终端执行：

```powershell
codex --version
```

### macOS / Linux

```bash
chmod +x codex
sudo mv codex /usr/local/bin/codex
codex --version
```

## 首次使用

```bash
codex
```

按提示完成登录（ChatGPT 或 API Key）。

## 常用命令

```bash
# 查看帮助
codex --help

# 交互模式（TUI）
codex

# 非交互模式
codex exec "请分析当前仓库并给出重构建议"

# 其它子命令
codex app
codex mcp --help
```

## 交互模式常用操作

1. 启动后直接输入自然语言需求。
2. 输入 `/` 打开斜杠命令列表。
3. 通过回车执行任务，按界面提示完成授权。
4. 根据输出继续追问，直到任务完成。

## 更新

- 二进制安装：下载新版本压缩包替换旧文件。
- 源码安装：拉取最新代码后重新编译。

## 说明

`npm install -g @openai/codex` 安装的是官方 npm 包，不是本仓库的编译产物。

---

License: Apache-2.0
