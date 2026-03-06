//! Standard type to use with the `--approval-mode` CLI option.

use clap::ValueEnum;

use codex_protocol::protocol::AskForApproval;

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum ApprovalModeCliArg {
    /// 仅对“可信”命令（如 `ls`、`cat`、`sed`）免审批执行。
    /// 若模型提出不在可信集合中的命令，则会升级为向用户请求批准。
    Untrusted,

    /// 已弃用：执行所有命令时不主动请求用户批准。
    /// 仅当命令执行失败时才会请求批准，以允许无沙箱执行。
    /// 交互式场景建议使用 `on-request`，非交互式场景建议使用 `never`。
    OnFailure,

    /// 由模型决定何时向用户请求批准。
    OnRequest,

    /// 从不请求用户批准。
    /// 执行失败会立即返回给模型。
    Never,
}

impl From<ApprovalModeCliArg> for AskForApproval {
    fn from(value: ApprovalModeCliArg) -> Self {
        match value {
            ApprovalModeCliArg::Untrusted => AskForApproval::UnlessTrusted,
            ApprovalModeCliArg::OnFailure => AskForApproval::OnFailure,
            ApprovalModeCliArg::OnRequest => AskForApproval::OnRequest,
            ApprovalModeCliArg::Never => AskForApproval::Never,
        }
    }
}
