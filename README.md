<p align="center"><code>npm i -g @openai/codex</code><br />or <code>brew install --cask codex</code></p>
<p align="center"><strong>Codex CLI</strong> is a coding agent from OpenAI that runs locally on your computer.
<p align="center">
  <img src="https://github.com/openai/codex/blob/main/.github/codex-cli-splash.png" alt="Codex CLI splash" width="80%" />
</p>
</br>
If you want Codex in your code editor (VS Code, Cursor, Windsurf), <a href="https://developers.openai.com/codex/ide">install in your IDE.</a>
</br>If you want the desktop app experience, run <code>codex app</code> or visit <a href="https://chatgpt.com/codex?app-landing-page=true">the Codex App page</a>.
</br>If you are looking for the <em>cloud-based agent</em> from OpenAI, <strong>Codex Web</strong>, go to <a href="https://chatgpt.com/codex">chatgpt.com/codex</a>.</p>

---

## Quickstart

### Installing and running Codex CLI

Install globally with your preferred package manager:

```shell
# Install using npm
npm install -g @openai/codex
```

```shell
# Install using Homebrew
brew install --cask codex
```

Then simply run `codex` to get started.

<details>
<summary>You can also go to the <a href="https://github.com/openai/codex/releases/latest">latest GitHub Release</a> and download the appropriate binary for your platform.</summary>

Each GitHub Release contains many executables, but in practice, you likely want one of these:

- macOS (Apple Silicon/arm64): `codex-aarch64-apple-darwin.tar.gz`
- macOS (x86_64): `codex-x86_64-apple-darwin.tar.gz`
- Linux (x86_64): `codex-x86_64-unknown-linux-musl.tar.gz`
- Linux (arm64): `codex-aarch64-unknown-linux-musl.tar.gz`

Each archive contains a single entry with the platform baked into the name (for example, `codex-x86_64-unknown-linux-musl`), so you likely want to rename it to `codex` after extracting it.

</details>

### Using Codex with your ChatGPT plan

Run `codex` and select **Sign in with ChatGPT**. We recommend signing into your ChatGPT account to use Codex as part of your Plus, Pro, Team, Edu, or Enterprise plan. [Learn more about what's included in your ChatGPT plan](https://help.openai.com/en/articles/11369540-codex-in-chatgpt).

You can also use Codex with an API key, but this requires [additional setup](https://developers.openai.com/codex/auth#sign-in-with-an-api-key).

## 中文说明（本仓库 Fork）

本仓库是 `openai/codex` 的中文增强版 Fork，目标是：

- 保持与官方主线同步；
- 保持可编译、可发布；
- 提供更完整的中文界面文本与使用说明。

当前仓库版本基线对齐：`0.106.0`。

### 安装方式

1. 直接下载已编译文件（推荐）

- 打开本仓库 `Releases` 页面，按系统下载对应文件：
- Windows: `codex-<tag>-windows-x86_64.zip`
- macOS Intel: `codex-<tag>-macos-x86_64.tar.gz`
- macOS Apple Silicon: `codex-<tag>-macos-aarch64.tar.gz`
- Linux x86_64: `codex-<tag>-linux-x86_64.tar.gz`
- Linux arm64: `codex-<tag>-linux-aarch64.tar.gz`
- Windows 使用 `codex.exe`；macOS/Linux 使用 `./codex`。

2. 从源码编译

```powershell
cargo build --release --locked --manifest-path codex-rs/Cargo.toml -p codex-cli --bin codex
```

编译产物路径：

- `codex-rs/target/release/codex.exe`

### 使用方法

1. 首次启动

```powershell
codex
```

首次运行按界面提示完成登录（ChatGPT 或 API Key）。

2. 查看命令帮助

```powershell
codex --help
```

3. 非交互执行

```powershell
codex exec "请分析当前仓库结构并给出重构建议"
```

4. TUI 内斜杠命令

- 进入交互界面后输入 `/` 可查看命令列表与二级说明。

### 发布二进制到 GitHub

推送版本标签即可触发自动发布工作流：

```powershell
git tag -a v0.106.0-zh.1 -m "Release v0.106.0-zh.1"
git push origin v0.106.0-zh.1
```

工作流会自动编译 `codex.exe` 并上传到该 tag 对应的 Release 资产。

## Docs

- [**Codex Documentation**](https://developers.openai.com/codex)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)
- [**Open source fund**](./docs/open-source-fund.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
