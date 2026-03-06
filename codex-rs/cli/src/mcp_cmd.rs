use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use clap::ArgGroup;
use codex_core::config::Config;
use codex_core::config::edit::ConfigEditsBuilder;
use codex_core::config::find_codex_home;
use codex_core::config::load_global_mcp_servers;
use codex_core::config::types::McpServerConfig;
use codex_core::config::types::McpServerTransportConfig;
use codex_core::mcp::McpManager;
use codex_core::mcp::auth::McpOAuthLoginSupport;
use codex_core::mcp::auth::compute_auth_statuses;
use codex_core::mcp::auth::oauth_login_support;
use codex_core::plugins::PluginsManager;
use codex_protocol::protocol::McpAuthStatus;
use codex_rmcp_client::delete_oauth_tokens;
use codex_rmcp_client::perform_oauth_login;
use codex_utils_cli::CliConfigOverrides;
use codex_utils_cli::format_env_display::format_env_display;

/// 子命令：
/// - `list`：列出已配置的服务器（支持 `--json`）
/// - `get`：显示单个服务器配置（支持 `--json`）
/// - `add`：向 `~/.codex/config.toml` 添加服务器启动配置
/// - `remove`：删除服务器配置
/// - `login`：通过 OAuth 登录 MCP 服务器
/// - `logout`：移除 MCP 服务器的 OAuth 凭据
#[derive(Debug, clap::Parser)]
pub struct McpCli {
    #[clap(flatten)]
    pub config_overrides: CliConfigOverrides,

    #[command(subcommand)]
    pub subcommand: McpSubcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum McpSubcommand {
    /// 列出已配置的 MCP 服务器。
    List(ListArgs),
    /// 显示单个 MCP 服务器配置。
    Get(GetArgs),
    /// 添加 MCP 服务器配置。
    Add(AddArgs),
    /// 移除 MCP 服务器配置。
    Remove(RemoveArgs),
    /// 通过 OAuth 登录 MCP 服务器。
    Login(LoginArgs),
    /// 移除 MCP 服务器的 OAuth 凭据。
    Logout(LogoutArgs),
}

#[derive(Debug, clap::Parser)]
pub struct ListArgs {
    /// 以 JSON 格式输出已配置的服务器。
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
pub struct GetArgs {
    /// 要显示的 MCP 服务器名称。
    pub name: String,

    /// 以 JSON 格式输出服务器配置。
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, clap::Parser)]
#[command(override_usage = "codex mcp add [OPTIONS] <NAME> (--url <URL> | -- <COMMAND>...)")]
pub struct AddArgs {
    /// MCP 服务器配置名称。
    pub name: String,

    #[command(flatten)]
    pub transport_args: AddMcpTransportArgs,
}

#[derive(Debug, clap::Args)]
#[command(
    group(
        ArgGroup::new("transport")
            .args(["command", "url"])
            .required(true)
            .multiple(false)
    )
)]
pub struct AddMcpTransportArgs {
    #[command(flatten)]
    pub stdio: Option<AddMcpStdioArgs>,

    #[command(flatten)]
    pub streamable_http: Option<AddMcpStreamableHttpArgs>,
}

#[derive(Debug, clap::Args)]
pub struct AddMcpStdioArgs {
    /// 启动 MCP 服务器的命令。
    /// 若为可流式 HTTP 服务器，请使用 `--url`。
    #[arg(
            trailing_var_arg = true,
            num_args = 0..,
        )]
    pub command: Vec<String>,

    /// 启动服务器时设置的环境变量。
    /// 仅适用于 stdio 服务器。
    #[arg(
        long,
        value_parser = parse_env_pair,
        value_name = "KEY=VALUE",
    )]
    pub env: Vec<(String, String)>,
}

#[derive(Debug, clap::Args)]
pub struct AddMcpStreamableHttpArgs {
    /// 可流式 HTTP MCP 服务器的 URL。
    #[arg(long)]
    pub url: String,

    /// 可选：用于读取 Bearer Token 的环境变量。
    /// 仅适用于可流式 HTTP 服务器。
    #[arg(
        long = "bearer-token-env-var",
        value_name = "ENV_VAR",
        requires = "url"
    )]
    pub bearer_token_env_var: Option<String>,
}

#[derive(Debug, clap::Parser)]
pub struct RemoveArgs {
    /// 要移除的 MCP 服务器配置名称。
    pub name: String,
}

#[derive(Debug, clap::Parser)]
pub struct LoginArgs {
    /// 要通过 OAuth 登录的 MCP 服务器名称。
    pub name: String,

    /// 需要请求的 OAuth scope 列表，使用逗号分隔。
    #[arg(long, value_delimiter = ',', value_name = "SCOPE,SCOPE")]
    pub scopes: Vec<String>,
}

#[derive(Debug, clap::Parser)]
pub struct LogoutArgs {
    /// 要退出登录的 MCP 服务器名称。
    pub name: String,
}

impl McpCli {
    pub async fn run(self) -> Result<()> {
        let McpCli {
            config_overrides,
            subcommand,
        } = self;

        match subcommand {
            McpSubcommand::List(args) => {
                run_list(&config_overrides, args).await?;
            }
            McpSubcommand::Get(args) => {
                run_get(&config_overrides, args).await?;
            }
            McpSubcommand::Add(args) => {
                run_add(&config_overrides, args).await?;
            }
            McpSubcommand::Remove(args) => {
                run_remove(&config_overrides, args).await?;
            }
            McpSubcommand::Login(args) => {
                run_login(&config_overrides, args).await?;
            }
            McpSubcommand::Logout(args) => {
                run_logout(&config_overrides, args).await?;
            }
        }

        Ok(())
    }
}

async fn run_add(config_overrides: &CliConfigOverrides, add_args: AddArgs) -> Result<()> {
    // Validate any provided overrides even though they are not currently applied.
    let overrides = config_overrides
        .parse_overrides()
        .map_err(anyhow::Error::msg)?;
    let config = Config::load_with_cli_overrides(overrides)
        .await
        .context("加载配置失败")?;

    let AddArgs {
        name,
        transport_args,
    } = add_args;

    validate_server_name(&name)?;

    let codex_home = find_codex_home().context("解析 CODEX_HOME 失败")?;
    let mut servers = load_global_mcp_servers(&codex_home)
        .await
        .with_context(|| format!("从 {} 加载 MCP 服务器失败", codex_home.display()))?;

    let transport = match transport_args {
        AddMcpTransportArgs {
            stdio: Some(stdio), ..
        } => {
            let mut command_parts = stdio.command.into_iter();
            let command_bin = command_parts
                .next()
                .ok_or_else(|| anyhow!("必须提供命令"))?;
            let command_args: Vec<String> = command_parts.collect();

            let env_map = if stdio.env.is_empty() {
                None
            } else {
                Some(stdio.env.into_iter().collect::<HashMap<_, _>>())
            };
            McpServerTransportConfig::Stdio {
                command: command_bin,
                args: command_args,
                env: env_map,
                env_vars: Vec::new(),
                cwd: None,
            }
        }
        AddMcpTransportArgs {
            streamable_http:
                Some(AddMcpStreamableHttpArgs {
                    url,
                    bearer_token_env_var,
                }),
            ..
        } => McpServerTransportConfig::StreamableHttp {
            url,
            bearer_token_env_var,
            http_headers: None,
            env_http_headers: None,
        },
        AddMcpTransportArgs { .. } => bail!("必须且只能提供 `--command` 或 `--url` 其中之一"),
    };

    let new_entry = McpServerConfig {
        transport: transport.clone(),
        enabled: true,
        required: false,
        disabled_reason: None,
        startup_timeout_sec: None,
        tool_timeout_sec: None,
        enabled_tools: None,
        disabled_tools: None,
        scopes: None,
        oauth_resource: None,
    };

    servers.insert(name.clone(), new_entry);

    ConfigEditsBuilder::new(&codex_home)
        .replace_mcp_servers(&servers)
        .apply()
        .await
        .with_context(|| format!("写入 MCP 服务器到 {} 失败", codex_home.display()))?;

    println!("已添加全局 MCP 服务器“{name}”。");

    match oauth_login_support(&transport).await {
        McpOAuthLoginSupport::Supported(oauth_config) => {
            println!("检测到 OAuth 支持，正在启动 OAuth 流程……");
            perform_oauth_login(
                &name,
                &oauth_config.url,
                config.mcp_oauth_credentials_store_mode,
                oauth_config.http_headers,
                oauth_config.env_http_headers,
                &Vec::new(),
                None,
                config.mcp_oauth_callback_port,
                config.mcp_oauth_callback_url.as_deref(),
            )
            .await?;
            println!("登录成功。");
        }
        McpOAuthLoginSupport::Unsupported => {}
        McpOAuthLoginSupport::Unknown(_) => {
            println!("无法确定 MCP 服务器是否需要登录。可运行 `codex mcp login {name}` 进行登录。")
        }
    }

    Ok(())
}

async fn run_remove(config_overrides: &CliConfigOverrides, remove_args: RemoveArgs) -> Result<()> {
    config_overrides
        .parse_overrides()
        .map_err(anyhow::Error::msg)?;

    let RemoveArgs { name } = remove_args;

    validate_server_name(&name)?;

    let codex_home = find_codex_home().context("解析 CODEX_HOME 失败")?;
    let mut servers = load_global_mcp_servers(&codex_home)
        .await
        .with_context(|| format!("从 {} 加载 MCP 服务器失败", codex_home.display()))?;

    let removed = servers.remove(&name).is_some();

    if removed {
        ConfigEditsBuilder::new(&codex_home)
            .replace_mcp_servers(&servers)
            .apply()
            .await
            .with_context(|| format!("写入 MCP 服务器到 {} 失败", codex_home.display()))?;
    }

    if removed {
        println!("已移除全局 MCP 服务器“{name}”。");
    } else {
        println!("未找到名为“{name}”的 MCP 服务器。");
    }

    Ok(())
}

async fn run_login(config_overrides: &CliConfigOverrides, login_args: LoginArgs) -> Result<()> {
    let overrides = config_overrides
        .parse_overrides()
        .map_err(anyhow::Error::msg)?;
    let config = Config::load_with_cli_overrides(overrides)
        .await
        .context("加载配置失败")?;
    let mcp_manager = McpManager::new(Arc::new(PluginsManager::new(config.codex_home.clone())));
    let mcp_servers = mcp_manager.effective_servers(&config, None);

    let LoginArgs { name, scopes } = login_args;

    let Some(server) = mcp_servers.get(&name) else {
        bail!("未找到名为“{name}”的 MCP 服务器。");
    };

    let (url, http_headers, env_http_headers) = match &server.transport {
        McpServerTransportConfig::StreamableHttp {
            url,
            http_headers,
            env_http_headers,
            ..
        } => (url.clone(), http_headers.clone(), env_http_headers.clone()),
        _ => bail!("OAuth 登录仅支持可流式 HTTP 服务器。"),
    };

    let mut scopes = scopes;
    if scopes.is_empty() {
        scopes = server.scopes.clone().unwrap_or_default();
    }

    perform_oauth_login(
        &name,
        &url,
        config.mcp_oauth_credentials_store_mode,
        http_headers,
        env_http_headers,
        &scopes,
        server.oauth_resource.as_deref(),
        config.mcp_oauth_callback_port,
        config.mcp_oauth_callback_url.as_deref(),
    )
    .await?;
    println!("已成功登录 MCP 服务器“{name}”。");
    Ok(())
}

async fn run_logout(config_overrides: &CliConfigOverrides, logout_args: LogoutArgs) -> Result<()> {
    let overrides = config_overrides
        .parse_overrides()
        .map_err(anyhow::Error::msg)?;
    let config = Config::load_with_cli_overrides(overrides)
        .await
        .context("加载配置失败")?;
    let mcp_manager = McpManager::new(Arc::new(PluginsManager::new(config.codex_home.clone())));
    let mcp_servers = mcp_manager.effective_servers(&config, None);

    let LogoutArgs { name } = logout_args;

    let server = mcp_servers
        .get(&name)
        .ok_or_else(|| anyhow!("配置中未找到名为“{name}”的 MCP 服务器。"))?;

    let url = match &server.transport {
        McpServerTransportConfig::StreamableHttp { url, .. } => url.clone(),
        _ => bail!("OAuth 退出登录仅支持 `streamable_http` 传输方式。"),
    };

    match delete_oauth_tokens(&name, &url, config.mcp_oauth_credentials_store_mode) {
        Ok(true) => println!("已移除“{name}”的 OAuth 凭据。"),
        Ok(false) => println!("“{name}”没有已保存的 OAuth 凭据。"),
        Err(err) => return Err(anyhow!("删除 OAuth 凭据失败：{err}")),
    }

    Ok(())
}

async fn run_list(config_overrides: &CliConfigOverrides, list_args: ListArgs) -> Result<()> {
    let overrides = config_overrides
        .parse_overrides()
        .map_err(anyhow::Error::msg)?;
    let config = Config::load_with_cli_overrides(overrides)
        .await
        .context("加载配置失败")?;
    let mcp_manager = McpManager::new(Arc::new(PluginsManager::new(config.codex_home.clone())));
    let mcp_servers = mcp_manager.effective_servers(&config, None);

    let mut entries: Vec<_> = mcp_servers.iter().collect();
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    let auth_statuses =
        compute_auth_statuses(mcp_servers.iter(), config.mcp_oauth_credentials_store_mode).await;

    if list_args.json {
        let json_entries: Vec<_> = entries
            .into_iter()
            .map(|(name, cfg)| {
                let auth_status = auth_statuses
                    .get(name.as_str())
                    .map(|entry| entry.auth_status)
                    .unwrap_or(McpAuthStatus::Unsupported);
                let transport = match &cfg.transport {
                    McpServerTransportConfig::Stdio {
                        command,
                        args,
                        env,
                        env_vars,
                        cwd,
                    } => serde_json::json!({
                        "type": "stdio",
                        "command": command,
                        "args": args,
                        "env": env,
                        "env_vars": env_vars,
                        "cwd": cwd,
                    }),
                    McpServerTransportConfig::StreamableHttp {
                        url,
                        bearer_token_env_var,
                        http_headers,
                        env_http_headers,
                    } => {
                        serde_json::json!({
                            "type": "streamable_http",
                            "url": url,
                            "bearer_token_env_var": bearer_token_env_var,
                            "http_headers": http_headers,
                            "env_http_headers": env_http_headers,
                        })
                    }
                };

                serde_json::json!({
                    "name": name,
                    "enabled": cfg.enabled,
                    "disabled_reason": cfg.disabled_reason.as_ref().map(ToString::to_string),
                    "transport": transport,
                    "startup_timeout_sec": cfg
                        .startup_timeout_sec
                        .map(|timeout| timeout.as_secs_f64()),
                    "tool_timeout_sec": cfg
                        .tool_timeout_sec
                        .map(|timeout| timeout.as_secs_f64()),
                    "auth_status": auth_status,
                })
            })
            .collect();
        let output = serde_json::to_string_pretty(&json_entries)?;
        println!("{output}");
        return Ok(());
    }

    if entries.is_empty() {
        println!("尚未配置任何 MCP 服务器。可以尝试运行 `codex mcp add my-tool -- my-command`。");
        return Ok(());
    }

    let mut stdio_rows: Vec<[String; 7]> = Vec::new();
    let mut http_rows: Vec<[String; 5]> = Vec::new();

    for (name, cfg) in entries {
        match &cfg.transport {
            McpServerTransportConfig::Stdio {
                command,
                args,
                env,
                env_vars,
                cwd,
            } => {
                let args_display = if args.is_empty() {
                    "-".to_string()
                } else {
                    args.join(" ")
                };
                let env_display = format_env_display(env.as_ref(), env_vars);
                let cwd_display = cwd
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "-".to_string());
                let status = format_mcp_status(cfg);
                let auth_status = auth_statuses
                    .get(name.as_str())
                    .map(|entry| format_auth_status(entry.auth_status))
                    .unwrap_or_else(|| format_auth_status(McpAuthStatus::Unsupported));
                stdio_rows.push([
                    name.clone(),
                    command.clone(),
                    args_display,
                    env_display,
                    cwd_display,
                    status,
                    auth_status,
                ]);
            }
            McpServerTransportConfig::StreamableHttp {
                url,
                bearer_token_env_var,
                ..
            } => {
                let status = format_mcp_status(cfg);
                let auth_status = auth_statuses
                    .get(name.as_str())
                    .map(|entry| format_auth_status(entry.auth_status))
                    .unwrap_or_else(|| format_auth_status(McpAuthStatus::Unsupported));
                let bearer_token_display =
                    bearer_token_env_var.as_deref().unwrap_or("-").to_string();
                http_rows.push([
                    name.clone(),
                    url.clone(),
                    bearer_token_display,
                    status,
                    auth_status,
                ]);
            }
        }
    }

    if !stdio_rows.is_empty() {
        let mut widths = [
            "名称".len(),
            "命令".len(),
            "参数".len(),
            "环境变量".len(),
            "工作目录".len(),
            "状态".len(),
            "认证".len(),
        ];
        for row in &stdio_rows {
            for (i, cell) in row.iter().enumerate() {
                widths[i] = widths[i].max(cell.len());
            }
        }

        println!(
            "{name:<name_w$}  {command:<cmd_w$}  {args:<args_w$}  {env:<env_w$}  {cwd:<cwd_w$}  {status:<status_w$}  {auth:<auth_w$}",
            name = "名称",
            command = "命令",
            args = "参数",
            env = "环境变量",
            cwd = "工作目录",
            status = "状态",
            auth = "认证",
            name_w = widths[0],
            cmd_w = widths[1],
            args_w = widths[2],
            env_w = widths[3],
            cwd_w = widths[4],
            status_w = widths[5],
            auth_w = widths[6],
        );

        for row in &stdio_rows {
            println!(
                "{name:<name_w$}  {command:<cmd_w$}  {args:<args_w$}  {env:<env_w$}  {cwd:<cwd_w$}  {status:<status_w$}  {auth:<auth_w$}",
                name = row[0].as_str(),
                command = row[1].as_str(),
                args = row[2].as_str(),
                env = row[3].as_str(),
                cwd = row[4].as_str(),
                status = row[5].as_str(),
                auth = row[6].as_str(),
                name_w = widths[0],
                cmd_w = widths[1],
                args_w = widths[2],
                env_w = widths[3],
                cwd_w = widths[4],
                status_w = widths[5],
                auth_w = widths[6],
            );
        }
    }

    if !stdio_rows.is_empty() && !http_rows.is_empty() {
        println!();
    }

    if !http_rows.is_empty() {
        let mut widths = [
            "名称".len(),
            "地址".len(),
            "Bearer Token 环境变量".len(),
            "状态".len(),
            "认证".len(),
        ];
        for row in &http_rows {
            for (i, cell) in row.iter().enumerate() {
                widths[i] = widths[i].max(cell.len());
            }
        }

        println!(
            "{name:<name_w$}  {url:<url_w$}  {token:<token_w$}  {status:<status_w$}  {auth:<auth_w$}",
            name = "名称",
            url = "地址",
            token = "Bearer Token 环境变量",
            status = "状态",
            auth = "认证",
            name_w = widths[0],
            url_w = widths[1],
            token_w = widths[2],
            status_w = widths[3],
            auth_w = widths[4],
        );

        for row in &http_rows {
            println!(
                "{name:<name_w$}  {url:<url_w$}  {token:<token_w$}  {status:<status_w$}  {auth:<auth_w$}",
                name = row[0].as_str(),
                url = row[1].as_str(),
                token = row[2].as_str(),
                status = row[3].as_str(),
                auth = row[4].as_str(),
                name_w = widths[0],
                url_w = widths[1],
                token_w = widths[2],
                status_w = widths[3],
                auth_w = widths[4],
            );
        }
    }

    Ok(())
}

async fn run_get(config_overrides: &CliConfigOverrides, get_args: GetArgs) -> Result<()> {
    let overrides = config_overrides
        .parse_overrides()
        .map_err(anyhow::Error::msg)?;
    let config = Config::load_with_cli_overrides(overrides)
        .await
        .context("加载配置失败")?;
    let mcp_manager = McpManager::new(Arc::new(PluginsManager::new(config.codex_home.clone())));
    let mcp_servers = mcp_manager.effective_servers(&config, None);

    let Some(server) = mcp_servers.get(&get_args.name) else {
        bail!("未找到名为“{name}”的 MCP 服务器。", name = get_args.name);
    };

    if get_args.json {
        let transport = match &server.transport {
            McpServerTransportConfig::Stdio {
                command,
                args,
                env,
                env_vars,
                cwd,
            } => serde_json::json!({
                "type": "stdio",
                "command": command,
                "args": args,
                "env": env,
                "env_vars": env_vars,
                "cwd": cwd,
            }),
            McpServerTransportConfig::StreamableHttp {
                url,
                bearer_token_env_var,
                http_headers,
                env_http_headers,
            } => serde_json::json!({
                "type": "streamable_http",
                "url": url,
                "bearer_token_env_var": bearer_token_env_var,
                "http_headers": http_headers,
                "env_http_headers": env_http_headers,
            }),
        };
        let output = serde_json::to_string_pretty(&serde_json::json!({
            "name": get_args.name,
            "enabled": server.enabled,
            "disabled_reason": server.disabled_reason.as_ref().map(ToString::to_string),
            "transport": transport,
            "enabled_tools": server.enabled_tools.clone(),
            "disabled_tools": server.disabled_tools.clone(),
            "startup_timeout_sec": server
                .startup_timeout_sec
                .map(|timeout| timeout.as_secs_f64()),
            "tool_timeout_sec": server
                .tool_timeout_sec
                .map(|timeout| timeout.as_secs_f64()),
        }))?;
        println!("{output}");
        return Ok(());
    }

    if !server.enabled {
        if let Some(reason) = server.disabled_reason.as_ref() {
            println!("{name}（已禁用：{reason}）", name = get_args.name);
        } else {
            println!("{name}（已禁用）", name = get_args.name);
        }
        return Ok(());
    }

    println!("{}", get_args.name);
    println!("  已启用：{}", server.enabled);
    let format_tool_list = |tools: &Option<Vec<String>>| -> String {
        match tools {
            Some(list) if list.is_empty() => "[]".to_string(),
            Some(list) => list.join(", "),
            None => "-".to_string(),
        }
    };
    if server.enabled_tools.is_some() {
        let enabled_tools_display = format_tool_list(&server.enabled_tools);
        println!("  已启用工具：{enabled_tools_display}");
    }
    if server.disabled_tools.is_some() {
        let disabled_tools_display = format_tool_list(&server.disabled_tools);
        println!("  已禁用工具：{disabled_tools_display}");
    }
    match &server.transport {
        McpServerTransportConfig::Stdio {
            command,
            args,
            env,
            env_vars,
            cwd,
        } => {
            println!("  传输方式：stdio");
            println!("  命令：{command}");
            let args_display = if args.is_empty() {
                "-".to_string()
            } else {
                args.join(" ")
            };
            println!("  参数：{args_display}");
            let cwd_display = cwd
                .as_ref()
                .map(|path| path.display().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "-".to_string());
            println!("  工作目录：{cwd_display}");
            let env_display = format_env_display(env.as_ref(), env_vars);
            println!("  环境变量：{env_display}");
        }
        McpServerTransportConfig::StreamableHttp {
            url,
            bearer_token_env_var,
            http_headers,
            env_http_headers,
        } => {
            println!("  传输方式：streamable_http");
            println!("  地址：{url}");
            let bearer_token_display = bearer_token_env_var.as_deref().unwrap_or("-");
            println!("  Bearer Token 环境变量：{bearer_token_display}");
            let headers_display = match http_headers {
                Some(map) if !map.is_empty() => {
                    let mut pairs: Vec<_> = map.iter().collect();
                    pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
                    pairs
                        .into_iter()
                        .map(|(k, _)| format!("{k}=*****"))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
                _ => "-".to_string(),
            };
            println!("  HTTP 头：{headers_display}");
            let env_headers_display = match env_http_headers {
                Some(map) if !map.is_empty() => {
                    let mut pairs: Vec<_> = map.iter().collect();
                    pairs.sort_by(|(a, _), (b, _)| a.cmp(b));
                    pairs
                        .into_iter()
                        .map(|(k, var)| format!("{k}={var}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
                _ => "-".to_string(),
            };
            println!("  环境变量 HTTP 头：{env_headers_display}");
        }
    }
    if let Some(timeout) = server.startup_timeout_sec {
        println!("  启动超时（秒）：{}", timeout.as_secs_f64());
    }
    if let Some(timeout) = server.tool_timeout_sec {
        println!("  工具超时（秒）：{}", timeout.as_secs_f64());
    }
    println!("  删除命令：codex mcp remove {}", get_args.name);

    Ok(())
}

fn parse_env_pair(raw: &str) -> Result<(String, String), String> {
    let mut parts = raw.splitn(2, '=');
    let key = parts
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "环境变量条目必须采用 KEY=VALUE 格式".to_string())?;
    let value = parts
        .next()
        .map(str::to_string)
        .ok_or_else(|| "环境变量条目必须采用 KEY=VALUE 格式".to_string())?;

    Ok((key.to_string(), value))
}

fn validate_server_name(name: &str) -> Result<()> {
    let is_valid = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');

    if is_valid {
        Ok(())
    } else {
        bail!("无效的服务器名称“{name}”（仅允许字母、数字、`-`、`_`）");
    }
}

fn format_mcp_status(config: &McpServerConfig) -> String {
    if config.enabled {
        "已启用".to_string()
    } else if let Some(reason) = config.disabled_reason.as_ref() {
        format!("已禁用：{reason}")
    } else {
        "已禁用".to_string()
    }
}

fn format_auth_status(status: McpAuthStatus) -> String {
    match status {
        McpAuthStatus::Unsupported => "不支持".to_string(),
        McpAuthStatus::NotLoggedIn => "未登录".to_string(),
        McpAuthStatus::BearerToken => "Bearer Token".to_string(),
        McpAuthStatus::OAuth => "OAuth".to_string(),
    }
}
