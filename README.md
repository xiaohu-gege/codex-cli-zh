# codex-cli-zh

`codex-cli-zh` 是 `openai/codex` 的中文增强版 Fork。

目标：

- 保持与官方版本同步（当前对齐 `0.106.0`）。
- 持续完善中文界面文案（菜单、提示、斜杠命令说明等）。
- 提供可直接下载的多平台二进制文件。

## 1. 下载与安装

### 1.1 从 Releases 直接下载（推荐）

进入仓库 `Releases` 页面，下载对应系统文件：

- Windows x64: `codex-<tag>-windows-x86_64.zip`
- macOS Intel: `codex-<tag>-macos-x86_64.tar.gz`
- macOS Apple Silicon: `codex-<tag>-macos-aarch64.tar.gz`
- Linux x64: `codex-<tag>-linux-x86_64.tar.gz`
- Linux arm64: `codex-<tag>-linux-aarch64.tar.gz`

解压后：

- Windows 可执行文件名是 `codex.exe`
- macOS/Linux 可执行文件名是 `codex`

### 1.2 使用 npm 全局安装

```bash
npm install -g @openai/codex
```

安装后可在任意目录直接执行：

```bash
codex --help
```

### 1.3 从源码编译

```powershell
cargo build --release --locked --manifest-path codex-rs/Cargo.toml -p codex-cli --bin codex
```

编译产物路径：

- Windows: `codex-rs/target/release/codex.exe`
- macOS/Linux: `codex-rs/target/release/codex`

## 2. 全局使用（重点）

### 2.1 Windows 全局使用

方式 A（推荐）：放到固定目录并加入 PATH。

1. 新建目录，例如 `C:\Tools\codex`。
2. 把 `codex.exe` 放进去。
3. 将 `C:\Tools\codex` 加入系统 `PATH`。
4. 重新打开终端后执行：

```powershell
codex --version
```

方式 B：使用 npm 全局安装（自动放入可执行路径）。

### 2.2 macOS 全局使用

```bash
chmod +x codex
sudo mv codex /usr/local/bin/codex
codex --version
```

### 2.3 Linux 全局使用

```bash
chmod +x codex
sudo mv codex /usr/local/bin/codex
codex --version
```

## 3. 使用教程（从 0 到 1）

### 3.1 首次启动

```bash
codex
```

按界面提示完成认证：

- `Sign in with ChatGPT`（推荐）
- 或使用 API Key

### 3.2 查看命令帮助

```bash
codex --help
```

### 3.3 交互模式（TUI）

```bash
codex
```

进入后常见操作：

- 输入自然语言需求直接执行。
- 输入 `/` 查看斜杠命令及二级菜单说明。
- 在对话中持续迭代修改代码、运行命令、查看结果。

### 3.4 非交互模式（适合脚本/自动化）

```bash
codex exec "请分析当前仓库并输出重构建议"
```

也可以将提示词通过标准输入传入。

### 3.5 常见子命令示例

```bash
codex app
codex exec "..."
codex mcp --help
```

## 4. 更新到新版本

### 4.1 二进制用户更新

- 到 `Releases` 下载新版本压缩包，替换旧文件。

### 4.2 npm 用户更新

```bash
npm update -g @openai/codex
```

## 5. Release 页面没有文件时怎么处理

如果 `Releases` 页面没有附件，通常是发布工作流失败或被跳过：

1. 打开 `Actions` 页面。
2. 查看 `release-binaries` 工作流是否成功。
3. 检查失败日志（常见是 Linux 依赖缺失）。
4. 修复后重新打 tag 触发发布。

## 6. 维护者发布步骤

### 6.1 提交并推送代码

```bash
git add -A
git commit -m "your message"
git push origin main
```

### 6.2 打发布标签（会自动编译并上传资产）

```bash
git tag -a v0.106.0-zh.3 -m "Release v0.106.0-zh.3"
git push origin v0.106.0-zh.3
```

发布工作流会自动完成：

- 多平台构建
- 生成压缩包
- 上传到 GitHub Release

## 7. 与官方同步

本地建议保留两个远程：

- `origin` -> 你的 fork
- `upstream` -> `openai/codex`

同步官方更新常用流程：

```bash
git fetch upstream
git checkout main
git merge upstream/main
git push origin main
```

---

License: Apache-2.0
