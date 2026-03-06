//! Standard type to use with the `--sandbox` (`-s`) CLI option.
//!
//! This mirrors the variants of [`codex_protocol::protocol::SandboxPolicy`], but
//! without any of the associated data so it can be expressed as a simple flag
//! on the command-line. Users that need to tweak the advanced options for
//! `workspace-write` can continue to do so via `-c` overrides or their
//! `config.toml`.

use clap::ValueEnum;
use codex_protocol::config_types::SandboxMode;

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum SandboxModeCliArg {
    /// 只读沙箱，不允许写入文件。
    ReadOnly,
    /// 工作区可写，其余位置保持受限。
    WorkspaceWrite,
    /// 完全访问，不启用沙箱隔离。
    DangerFullAccess,
}

impl From<SandboxModeCliArg> for SandboxMode {
    fn from(value: SandboxModeCliArg) -> Self {
        match value {
            SandboxModeCliArg::ReadOnly => SandboxMode::ReadOnly,
            SandboxModeCliArg::WorkspaceWrite => SandboxMode::WorkspaceWrite,
            SandboxModeCliArg::DangerFullAccess => SandboxMode::DangerFullAccess,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn maps_cli_args_to_protocol_modes() {
        assert_eq!(SandboxMode::ReadOnly, SandboxModeCliArg::ReadOnly.into());
        assert_eq!(
            SandboxMode::WorkspaceWrite,
            SandboxModeCliArg::WorkspaceWrite.into()
        );
        assert_eq!(
            SandboxMode::DangerFullAccess,
            SandboxModeCliArg::DangerFullAccess.into()
        );
    }
}
