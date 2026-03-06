use clap::Parser;
use clap::ValueHint;
use codex_utils_cli::ApprovalModeCliArg;
use codex_utils_cli::CliConfigOverrides;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version)]
pub struct Cli {
    /// 可选的初始提示词，用于启动会话。
    #[arg(value_name = "PROMPT", value_hint = clap::ValueHint::Other)]
    pub prompt: Option<String>,

    /// 可选的图片，会附加到首条消息中。
    #[arg(long = "image", short = 'i', value_name = "FILE", value_delimiter = ',', num_args = 1..)]
    pub images: Vec<PathBuf>,

    // Internal controls set by the top-level `codex resume` subcommand.
    // These are not exposed as user flags on the base `codex` command.
    #[clap(skip)]
    pub resume_picker: bool,

    #[clap(skip)]
    pub resume_last: bool,

    /// 内部参数：按 id（UUID）恢复指定会话。
    /// 由顶层 `codex resume <SESSION_ID>` 包装命令设置，不作为公开参数暴露。
    #[clap(skip)]
    pub resume_session_id: Option<String>,

    /// 内部参数：显示全部会话（禁用 cwd 过滤并显示 CWD 列）。
    #[clap(skip)]
    pub resume_show_all: bool,

    // Internal controls set by the top-level `codex fork` subcommand.
    // These are not exposed as user flags on the base `codex` command.
    #[clap(skip)]
    pub fork_picker: bool,

    #[clap(skip)]
    pub fork_last: bool,

    /// 内部参数：按 id（UUID）分叉指定会话。
    /// 由顶层 `codex fork <SESSION_ID>` 包装命令设置，不作为公开参数暴露。
    #[clap(skip)]
    pub fork_session_id: Option<String>,

    /// 内部参数：显示全部会话（禁用 cwd 过滤并显示 CWD 列）。
    #[clap(skip)]
    pub fork_show_all: bool,

    /// 指定代理使用的模型。
    #[arg(long, short = 'm')]
    pub model: Option<String>,

    /// 便捷参数：选择本地开源模型提供方。
    /// 等价于 `-c model_provider=oss`；会检查本地 LM Studio 或 Ollama 服务是否已启动。
    #[arg(long = "oss", default_value_t = false)]
    pub oss: bool,

    /// 指定本地提供方（`lmstudio` 或 `ollama`）。
    /// 若与 `--oss` 一起使用且未指定，则使用配置默认值或弹出选择。
    #[arg(long = "local-provider")]
    pub oss_provider: Option<String>,

    /// 从 `config.toml` 中选择配置档案作为默认选项。
    #[arg(long = "profile", short = 'p')]
    pub config_profile: Option<String>,

    /// 选择执行模型生成 shell 命令时使用的沙箱策略。
    #[arg(long = "sandbox", short = 's')]
    pub sandbox_mode: Option<codex_utils_cli::SandboxModeCliArg>,

    /// 配置模型在什么情况下需要人工批准后才能执行命令。
    #[arg(long = "ask-for-approval", short = 'a')]
    pub approval_policy: Option<ApprovalModeCliArg>,

    /// 便捷参数：启用低摩擦沙箱自动执行。
    /// 等价于 `-a on-request --sandbox workspace-write`。
    #[arg(long = "full-auto", default_value_t = false)]
    pub full_auto: bool,

    /// 跳过所有确认提示，并在无沙箱模式下执行命令。
    /// 风险极高，仅适用于外部已做好隔离的环境。
    #[arg(
        long = "dangerously-bypass-approvals-and-sandbox",
        alias = "yolo",
        default_value_t = false,
        conflicts_with_all = ["approval_policy", "full_auto"]
    )]
    pub dangerously_bypass_approvals_and_sandbox: bool,

    /// 指定代理使用的工作根目录。
    #[clap(long = "cd", short = 'C', value_name = "DIR")]
    pub cwd: Option<PathBuf>,

    /// 启用实时网页搜索。
    /// 启用后，模型可直接使用原生 Responses `web_search` 工具（无需逐次批准）。
    #[arg(long = "search", default_value_t = false)]
    pub web_search: bool,

    /// 除主工作区外，额外允许写入的目录。
    #[arg(long = "add-dir", value_name = "DIR", value_hint = ValueHint::DirPath)]
    pub add_dir: Vec<PathBuf>,

    /// 禁用备用屏幕模式。
    ///
    /// 以行内模式运行 TUI，保留终端滚动历史。
    /// 这在像 Zellij 这类严格遵循 xterm 规范、会禁用备用屏幕滚动历史的终端复用器中很有用。
    #[arg(long = "no-alt-screen", default_value_t = false)]
    pub no_alt_screen: bool,

    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,
}
