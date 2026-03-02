# 发布到 GitHub 并提供可下载的 `exe`

本文档说明如何把当前项目发布到你自己的 GitHub 仓库，并让用户在 Releases 页面直接下载编译好的 Windows 可执行文件。

## 1. 首次推送到你的 GitHub 仓库

在 GitHub 先创建一个空仓库（例如 `yourname/codex`），然后在本地执行：

```powershell
git remote remove origin
git remote add origin https://github.com/<你的用户名>/<你的仓库名>.git
git branch -M main
git push -u origin main
```

如果你已经配置了 `origin`，只需要确认 `git remote -v` 指向你的仓库即可。

## 2. 触发自动编译并发布 Release

仓库已经包含工作流：

- `.github/workflows/release-windows-binary.yml`

它会在两种情况下触发：

- 推送 tag（`v*`，例如 `v1.0.0`）
- 在 GitHub Actions 页面手动运行（`workflow_dispatch`）

推荐用 tag 发布：

```powershell
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

推送后，GitHub Actions 会自动：

1. 编译 `codex.exe`
2. 打包为 `codex-v1.0.0-windows-x86_64.zip`
3. 创建/更新对应的 GitHub Release 并上传该压缩包

## 3. 用户如何下载并直接使用

用户进入你的仓库页面：

- `Releases` -> 选择版本 -> 下载 `codex-vX.Y.Z-windows-x86_64.zip`

解压后即可在 PowerShell 使用：

```powershell
.\codex.exe --help
```

可选：把 `codex.exe` 所在目录加入 `PATH`，之后可在任意目录直接运行 `codex`。

## 4. 本地手动编译时，`exe` 输出位置

如果你自己本地编译：

```powershell
cargo build --release --locked --manifest-path codex-rs/Cargo.toml -p codex-cli --bin codex
```

生成文件路径：

- `codex-rs/target/release/codex.exe`
