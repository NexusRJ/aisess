use crate::model::{MetadataState, Session};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryError {
    pub message: String,
}

pub trait SessionDiscovery {
    fn discover(&self) -> Result<Vec<Session>, DiscoveryError>;
}

pub struct CombinedDiscovery {
    providers: Vec<Box<dyn SessionDiscovery>>,
}

impl CombinedDiscovery {
    pub fn new(providers: Vec<Box<dyn SessionDiscovery>>) -> Self {
        Self { providers }
    }
}

impl SessionDiscovery for CombinedDiscovery {
    fn discover(&self) -> Result<Vec<Session>, DiscoveryError> {
        let mut sessions = Vec::new();
        for provider in &self.providers {
            match provider.discover() {
                Ok(mut discovered) => sessions.append(&mut discovered),
                Err(error) => return Err(error),
            }
        }
        Ok(sessions)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct EmptyDiscovery;

impl SessionDiscovery for EmptyDiscovery {
    fn discover(&self) -> Result<Vec<Session>, DiscoveryError> {
        Ok(vec![])
    }
}

#[derive(Debug, Clone)]
pub struct ClaudeDiscovery {
    root: PathBuf,
    active_window_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct CodexDiscovery {
    root: PathBuf,
    active_window_seconds: i64,
}

impl CodexDiscovery {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            active_window_seconds: 15 * 60,
        }
    }

    pub fn with_active_window(root: PathBuf, active_window_seconds: i64) -> Self {
        Self {
            root,
            active_window_seconds,
        }
    }

    pub fn default_root() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(".codex/sessions"))
    }

    fn session_files(&self) -> Result<Vec<PathBuf>, DiscoveryError> {
        let mut files = Vec::new();
        visit_jsonl_files(&self.root, &mut files).map_err(|error| DiscoveryError {
            message: format!("failed to scan codex sessions: {error}"),
        })?;
        Ok(files)
    }

    fn parse_session_file(path: &Path) -> Result<Option<Session>, DiscoveryError> {
        let content = fs::read_to_string(path).map_err(|error| DiscoveryError {
            message: format!("failed to read {}: {error}", path.display()),
        })?;

        let mut session_id = None;
        let mut title = None;
        let mut summary = None;
        let mut started_at = None;
        let mut last_active_at = None;
        let mut cwd = None;
        let mut model = None;
        let mut message_count = 0_u64;
        let mut token_usage = None;
        let mut context_length = None;
        let mut source = Some("codex-unknown".to_string());
        let file_timestamp = codex_file_timestamp(path);

        for line in content.lines() {
            let Ok(entry) = serde_json::from_str::<CodexEntry>(line) else {
                continue;
            };

            if started_at.is_none() {
                started_at = entry.timestamp();
            }
            if let Some(timestamp) = entry.timestamp() {
                last_active_at = Some(timestamp);
            }

            if let Some(meta) = entry.session_meta() {
                if session_id.is_none() {
                    session_id = meta.id.clone();
                }
                if cwd.is_none() {
                    cwd = meta.cwd.clone();
                }
                if let Some(originator) = meta.originator.clone() {
                    source = Some(match originator.as_str() {
                        "codex-tui" => "codex-cli".to_string(),
                        "desktop" => "codex-desktop".to_string(),
                        other => format!("codex-{other}"),
                    });
                }
                if title.is_none() {
                    title = meta.cwd.as_deref().and_then(project_name_from_cwd);
                }
            }

            if let Some(context) = entry.turn_context() {
                if cwd.is_none() {
                    cwd = context.cwd.clone();
                }
                if model.is_none() {
                    model = context.model.clone();
                }
                if summary.is_none() {
                    summary = context.summary.clone();
                }
                if token_usage.is_none() {
                    token_usage = context.token_usage.as_ref().and_then(|usage| usage.total);
                }
                if context_length.is_none() {
                    context_length = context
                        .token_usage
                        .as_ref()
                        .and_then(|usage| usage.context_length);
                }
            }

            if let Some(user_text) = entry.user_text() {
                summary = Some(user_text.to_string());
                if title.is_none() {
                    title = Some(truncate_for_title(&user_text));
                }
                message_count += 1;
            }

            if let Some(agent_text) = entry.agent_text() {
                summary = Some(agent_text.to_string());
            }
        }

        let Some(session_id) = session_id else {
            return Ok(None);
        };

        let runtime_seconds = started_at
            .zip(last_active_at)
            .and_then(|(start, end)| (end - start).whole_seconds().try_into().ok());

        let summary = summary.or_else(|| file_timestamp.map(|ts| ts.to_string()));

        Ok(Some(Session {
            session_id,
            provider: Some("codex".to_string()),
            source,
            activity_state: None,
            cwd: cwd.clone(),
            match_reason: None,
            dedupe_key: comparable_cwd(cwd.as_deref()),
            hidden_duplicates: 0,
            title: title.or_else(|| cwd.as_deref().and_then(project_name_from_cwd)),
            summary,
            status: Some("available".to_string()),
            started_at,
            last_active_at,
            runtime_seconds,
            token_usage,
            context_length,
            message_count: Some(message_count),
            metadata_state: MetadataState::Partial,
        }))
    }
}

impl SessionDiscovery for CodexDiscovery {
    fn discover(&self) -> Result<Vec<Session>, DiscoveryError> {
        let mut sessions = Vec::new();
        let now = OffsetDateTime::now_utc();
        let live_processes = codex_processes().unwrap_or_default();
        for file in self.session_files()? {
            if let Some(mut session) = Self::parse_session_file(&file)? {
                let match_reason = (session.source.as_deref() == Some("codex-cli"))
                    .then(|| codex_process_match_reason(&session, &live_processes))
                    .flatten();
                let is_running = match_reason.is_some();
                let is_recent = is_active_session(&session, now, self.active_window_seconds);

                if is_running || is_recent {
                    session.activity_state = Some(if is_running {
                        "running".to_string()
                    } else {
                        "recent".to_string()
                    });
                    session.match_reason = match_reason.or_else(|| Some("recent-log".to_string()));
                    session.dedupe_key = session_project_key(&session);
                    sessions.push(session);
                }
            }
        }
        Ok(deduplicate_codex_sessions(sessions, &live_processes))
    }
}

impl ClaudeDiscovery {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            active_window_seconds: 15 * 60,
        }
    }

    pub fn with_active_window(root: PathBuf, active_window_seconds: i64) -> Self {
        Self {
            root,
            active_window_seconds,
        }
    }

    pub fn default_root() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(".claude/projects"))
    }

    fn session_files(&self) -> Result<Vec<PathBuf>, DiscoveryError> {
        let mut files = Vec::new();
        visit_jsonl_files(&self.root, &mut files).map_err(|error| DiscoveryError {
            message: format!("failed to scan claude sessions: {error}"),
        })?;
        Ok(files)
    }

    fn parse_session_file(path: &Path) -> Result<Option<Session>, DiscoveryError> {
        let content = fs::read_to_string(path).map_err(|error| DiscoveryError {
            message: format!("failed to read {}: {error}", path.display()),
        })?;

        let mut session_id = None;
        let mut title = None;
        let mut summary = None;
        let mut started_at = None;
        let mut last_active_at = None;
        let mut cwd = None;
        let mut message_count = 0_u64;
        let mut token_usage = 0_u64;

        for line in content.lines() {
            let Ok(entry) = serde_json::from_str::<ClaudeEntry>(line) else {
                continue;
            };

            if session_id.is_none() {
                session_id = entry.session_id();
            }

            if started_at.is_none() {
                started_at = entry.timestamp();
            }
            if let Some(timestamp) = entry.timestamp() {
                last_active_at = Some(timestamp);
            }

            if let Some(prompt) = entry.last_prompt() {
                summary = Some(prompt.to_string());
            }

            if cwd.is_none() {
                cwd = entry.cwd();
            }

            if let Some(user_text) = entry.user_text() {
                summary = Some(user_text.to_string());
                if title.is_none() {
                    title = Some(truncate_for_title(user_text));
                }
                message_count += 1;
            }

            if let Some(usage) = entry.usage() {
                token_usage += usage.input_tokens.unwrap_or(0) + usage.output_tokens.unwrap_or(0);
            }
        }

        let Some(session_id) = session_id else {
            return Ok(None);
        };

        let runtime_seconds = started_at
            .zip(last_active_at)
            .and_then(|(start, end)| (end - start).whole_seconds().try_into().ok());

        Ok(Some(Session {
            session_id,
            provider: Some("claude-code".to_string()),
            source: Some("claude-local".to_string()),
            activity_state: None,
            cwd: cwd.clone().or_else(|| decode_claude_project_cwd(path)),
            match_reason: None,
            dedupe_key: cwd
                .as_deref()
                .and_then(|cwd| comparable_cwd(Some(cwd)))
                .or_else(|| {
                    decode_claude_project_cwd(path).and_then(|cwd| comparable_cwd(Some(&cwd)))
                }),
            hidden_duplicates: 0,
            title: title
                .or_else(|| cwd.as_deref().and_then(project_name_from_cwd))
                .or_else(|| decode_claude_project_dir(path)),
            summary,
            status: Some("available".to_string()),
            started_at,
            last_active_at,
            runtime_seconds,
            token_usage: (token_usage > 0).then_some(token_usage),
            context_length: None,
            message_count: Some(message_count),
            metadata_state: MetadataState::Partial,
        }))
    }
}

impl SessionDiscovery for ClaudeDiscovery {
    fn discover(&self) -> Result<Vec<Session>, DiscoveryError> {
        let mut sessions = Vec::new();
        let now = OffsetDateTime::now_utc();
        let live_processes = claude_processes().unwrap_or_default();
        for file in self.session_files()? {
            if let Some(mut session) = Self::parse_session_file(&file)? {
                let match_reason = claude_process_match_reason(&session, &live_processes);
                let is_running = match_reason.is_some();
                let is_recent = is_active_session(&session, now, self.active_window_seconds);
                if is_running || is_recent {
                    session.activity_state = Some(if is_running {
                        "running".to_string()
                    } else {
                        "recent".to_string()
                    });
                    session.match_reason = match_reason.or_else(|| Some("recent-log".to_string()));
                    session.dedupe_key = session_project_key(&session);
                    sessions.push(session);
                }
            }
        }
        Ok(deduplicate_claude_sessions(sessions))
    }
}

fn is_active_session(session: &Session, now: OffsetDateTime, active_window_seconds: i64) -> bool {
    session
        .last_active_at
        .map(|last_active| (now - last_active).whole_seconds() <= active_window_seconds)
        .unwrap_or(false)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodexProcessInfo {
    pid: u32,
    tty: String,
    command: String,
    cwd: Option<String>,
}

fn codex_processes() -> Result<Vec<CodexProcessInfo>, DiscoveryError> {
    let output = Command::new("ps")
        .args(["-axo", "pid=,tty=,command="])
        .output()
        .map_err(|error| DiscoveryError {
            message: format!("failed to inspect processes: {error}"),
        })?;

    if !output.status.success() {
        return Err(DiscoveryError {
            message: "ps command failed".to_string(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut processes = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let Some(pid) = parts.next().and_then(|value| value.parse::<u32>().ok()) else {
            continue;
        };
        let Some(tty) = parts.next() else {
            continue;
        };
        let command = parts.collect::<Vec<_>>().join(" ");
        let lower = command.to_lowercase();
        let is_cli_codex = lower.contains("/bin/codex")
            || lower.contains(" node ") && lower.contains("/bin/codex");
        let is_desktop = lower.contains("codex.app")
            || lower.contains("codex helper")
            || lower.contains("app-server");

        if is_cli_codex && !is_desktop && !lower.contains("rg -i codex") {
            processes.push(CodexProcessInfo {
                pid,
                tty: tty.to_string(),
                command,
                cwd: process_cwd(pid),
            });
        }
    }

    Ok(processes)
}

#[cfg(test)]
fn matches_live_codex_process(session: &Session, processes: &[CodexProcessInfo]) -> bool {
    codex_process_match_reason(session, processes).is_some()
}

fn codex_process_match_reason(session: &Session, processes: &[CodexProcessInfo]) -> Option<String> {
    let session_cwd = comparable_cwd(
        session.cwd.as_deref().or(session
            .title
            .as_deref()
            .filter(|value| value.starts_with('/'))),
    );
    let session_name = session.title.as_deref().unwrap_or_default();

    processes.iter().find_map(|process| {
        let process_cwd = comparable_cwd(process.cwd.as_deref());
        let cwd_match = session_cwd
            .as_deref()
            .zip(process_cwd.as_deref())
            .map(|(session_cwd, process_cwd)| session_cwd == process_cwd)
            .unwrap_or(false);
        if cwd_match {
            return Some("process-cwd".to_string());
        }

        process_cwd
            .as_deref()
            .and_then(project_name_from_cwd)
            .map(|name| name.eq_ignore_ascii_case(&session_name))
            .unwrap_or(false)
            .then(|| "project-name".to_string())
    })
}

fn comparable_cwd(cwd: Option<&str>) -> Option<String> {
    cwd.map(|value| value.trim_end_matches('/').to_lowercase())
        .filter(|value| !value.is_empty())
}

fn project_name_from_cwd(cwd: &str) -> Option<String> {
    let path = Path::new(cwd);
    let leaf = path.file_name().and_then(|s| s.to_str())?;
    if leaf == "rj" || leaf == "Users" || leaf.is_empty() {
        None
    } else {
        Some(leaf.to_string())
    }
}

fn process_cwd(pid: u32) -> Option<String> {
    let output = Command::new("lsof")
        .args(["-a", "-p", &pid.to_string(), "-d", "cwd", "-Fn"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix('n').map(|value| value.to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClaudeProcessInfo {
    pid: u32,
    tty: String,
    command: String,
    cwd: Option<String>,
}

fn claude_processes() -> Result<Vec<ClaudeProcessInfo>, DiscoveryError> {
    let output = Command::new("ps")
        .args(["-axo", "pid=,tty=,command="])
        .output()
        .map_err(|error| DiscoveryError {
            message: format!("failed to inspect claude processes: {error}"),
        })?;

    if !output.status.success() {
        return Err(DiscoveryError {
            message: "ps command failed".to_string(),
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut processes = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let Some(pid) = parts.next().and_then(|value| value.parse::<u32>().ok()) else {
            continue;
        };
        let Some(tty) = parts.next() else {
            continue;
        };
        let command = parts.collect::<Vec<_>>().join(" ");
        let lower = command.to_lowercase();
        if lower == "claude" || lower.ends_with(" claude") || lower.contains("/claude ") {
            processes.push(ClaudeProcessInfo {
                pid,
                tty: tty.to_string(),
                command,
                cwd: process_cwd(pid),
            });
        }
    }

    Ok(processes)
}

#[cfg(test)]
fn matches_live_claude_process(session: &Session, processes: &[ClaudeProcessInfo]) -> bool {
    claude_process_match_reason(session, processes).is_some()
}

fn claude_process_match_reason(
    session: &Session,
    processes: &[ClaudeProcessInfo],
) -> Option<String> {
    let session_name = session.title.as_deref().unwrap_or_default();
    let session_cwd = comparable_cwd(session.cwd.as_deref());

    processes.iter().find_map(|process| {
        let process_cwd = comparable_cwd(process.cwd.as_deref());
        let cwd_match = session_cwd
            .as_deref()
            .zip(process_cwd.as_deref())
            .map(|(session_cwd, process_cwd)| session_cwd == process_cwd)
            .unwrap_or(false);
        if cwd_match {
            return Some("process-cwd".to_string());
        }

        process
            .cwd
            .as_deref()
            .and_then(project_name_from_cwd)
            .map(|name| name.eq_ignore_ascii_case(session_name))
            .unwrap_or(false)
            .then(|| "project-name".to_string())
    })
}

fn deduplicate_claude_sessions(sessions: Vec<Session>) -> Vec<Session> {
    deduplicate_running_sessions_by_project(sessions)
}

fn deduplicate_codex_sessions(
    sessions: Vec<Session>,
    live_processes: &[CodexProcessInfo],
) -> Vec<Session> {
    let mut running = Vec::new();
    let mut recent = Vec::new();

    for session in sessions {
        if session.activity_state.as_deref() == Some("running") {
            running.push(session);
        } else {
            recent.push(session);
        }
    }

    let mut selected = Vec::new();
    let mut used_projects = std::collections::HashSet::new();

    running.sort_by_key(|session| std::cmp::Reverse(session.last_active_at));

    for mut session in running {
        if best_matching_process_index(&session, live_processes).is_some() {
            let project_key = session_project_key(&session);
            if project_key
                .as_ref()
                .is_some_and(|key| used_projects.contains(key))
            {
                continue;
            }
            if let Some(key) = project_key {
                session.dedupe_key = Some(key.clone());
                used_projects.insert(key);
            }
            selected.push(session);
        }
    }

    recent.sort_by_key(|session| std::cmp::Reverse(session.last_active_at));
    let mut seen_titles = selected
        .iter()
        .filter_map(session_project_key)
        .collect::<std::collections::HashSet<_>>();

    for session in recent {
        let key = session_project_key(&session).unwrap_or_else(|| session.session_id.clone());
        if !seen_titles.contains(&key) {
            seen_titles.insert(key);
            selected.push(session);
        }
    }
    selected
}

fn deduplicate_running_sessions_by_project(mut sessions: Vec<Session>) -> Vec<Session> {
    let mut selected: Vec<Session> = Vec::new();
    let mut seen_projects = std::collections::HashMap::<String, usize>::new();

    sessions.sort_by_key(|session| std::cmp::Reverse(session.last_active_at));

    for mut session in sessions {
        let key = session_project_key(&session).unwrap_or_else(|| session.session_id.clone());
        session.dedupe_key = Some(key.clone());
        if let Some(index) = seen_projects.get(&key) {
            selected[*index].hidden_duplicates += 1;
        } else {
            seen_projects.insert(key, selected.len());
            selected.push(session);
        }
    }

    selected
}

fn best_matching_process_index(session: &Session, processes: &[CodexProcessInfo]) -> Option<usize> {
    let session_cwd = comparable_cwd(session.cwd.as_deref());
    let session_name = session.title.as_deref().unwrap_or_default();

    processes
        .iter()
        .enumerate()
        .find(|(_, process)| {
            let process_cwd = comparable_cwd(process.cwd.as_deref());
            let cwd_match = session_cwd
                .as_deref()
                .zip(process_cwd.as_deref())
                .map(|(session_cwd, process_cwd)| session_cwd == process_cwd)
                .unwrap_or(false);

            let repo_match = process
                .cwd
                .as_deref()
                .and_then(project_name_from_cwd)
                .map(|name| name.eq_ignore_ascii_case(session_name))
                .unwrap_or(false);

            cwd_match || repo_match
        })
        .map(|(index, _)| index)
}

fn session_project_key(session: &Session) -> Option<String> {
    comparable_cwd(session.cwd.as_deref())
        .or_else(|| session.title.as_deref().map(|title| title.to_lowercase()))
}

fn codex_file_timestamp(path: &Path) -> Option<OffsetDateTime> {
    let name = path.file_name()?.to_str()?;
    let trimmed = name.strip_prefix("rollout-")?.strip_suffix(".jsonl")?;
    let timestamp = trimmed.get(0..19)?;
    let normalized = timestamp.replace('-', ":").replacen(':', "-", 2);
    OffsetDateTime::parse(
        &format!("{}Z", normalized.replace('T', "T")),
        &time::format_description::well_known::Rfc3339,
    )
    .ok()
}

fn decode_claude_project_dir(path: &Path) -> Option<String> {
    decode_claude_project_cwd(path).and_then(|cwd| project_name_from_cwd(&cwd))
}

fn decode_claude_project_cwd(path: &Path) -> Option<String> {
    let project_dir = path.parent()?.file_name()?.to_str()?;
    let decoded = project_dir.trim_start_matches('-').replace('-', "/");
    Some(format!("/{decoded}"))
}

fn visit_jsonl_files(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            visit_jsonl_files(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
            files.push(path);
        }
    }

    Ok(())
}

fn truncate_for_title(text: &str) -> String {
    let trimmed = text.trim();
    let mut chars = trimmed.chars();
    let title: String = chars.by_ref().take(32).collect();
    if chars.next().is_some() {
        format!("{title}…")
    } else {
        title
    }
}

#[derive(Debug, Deserialize)]
struct ClaudeMessageContent {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ClaudeEntry {
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    timestamp: Option<String>,
    #[serde(rename = "lastPrompt")]
    last_prompt: Option<String>,
    cwd: Option<String>,
    #[serde(rename = "type")]
    entry_type: Option<String>,
    message: Option<ClaudeMessageContent>,
    usage: Option<ClaudeUsage>,
}

#[derive(Debug, Deserialize)]
struct CodexSessionMetaPayload {
    id: Option<String>,
    cwd: Option<String>,
    originator: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexTurnContextPayload {
    cwd: Option<String>,
    model: Option<String>,
    summary: Option<String>,
    token_usage: Option<CodexTokenUsage>,
}

#[derive(Debug, Deserialize)]
struct CodexTokenUsage {
    total: Option<u64>,
    context_length: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CodexMessageContentItem {
    #[serde(rename = "type")]
    content_type: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexResponsePayload {
    #[serde(rename = "type")]
    payload_type: Option<String>,
    role: Option<String>,
    content: Option<Vec<CodexMessageContentItem>>,
}

#[derive(Debug, Deserialize)]
struct CodexEventPayload {
    #[serde(rename = "type")]
    event_type: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CodexEntry {
    timestamp: Option<String>,
    #[serde(rename = "type")]
    entry_type: Option<String>,
    payload: Option<serde_json::Value>,
}

impl CodexEntry {
    fn timestamp(&self) -> Option<OffsetDateTime> {
        self.timestamp.as_deref().and_then(|value| {
            OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok()
        })
    }

    fn session_meta(&self) -> Option<CodexSessionMetaPayload> {
        (self.entry_type.as_deref() == Some("session_meta"))
            .then_some(())
            .and_then(|_| self.payload.clone())
            .and_then(|payload| serde_json::from_value(payload).ok())
    }

    fn turn_context(&self) -> Option<CodexTurnContextPayload> {
        (self.entry_type.as_deref() == Some("turn_context"))
            .then_some(())
            .and_then(|_| self.payload.clone())
            .and_then(|payload| serde_json::from_value(payload).ok())
    }

    fn user_text(&self) -> Option<String> {
        let payload: CodexResponsePayload = (self.entry_type.as_deref() == Some("response_item"))
            .then_some(())
            .and_then(|_| self.payload.clone())
            .and_then(|payload| serde_json::from_value(payload).ok())?;

        if payload.payload_type.as_deref() == Some("message")
            && payload.role.as_deref() == Some("user")
        {
            payload
                .content
                .as_ref()?
                .iter()
                .find(|item| item.content_type.as_deref() == Some("input_text"))
                .and_then(|item| item.text.clone())
        } else {
            None
        }
    }

    fn agent_text(&self) -> Option<String> {
        if self.entry_type.as_deref() == Some("event_msg") {
            let payload: CodexEventPayload = self
                .payload
                .clone()
                .and_then(|payload| serde_json::from_value(payload).ok())?;
            if payload.event_type.as_deref() == Some("agent_message") {
                return payload.message;
            }
        }
        None
    }
}

impl ClaudeEntry {
    fn session_id(&self) -> Option<String> {
        self.session_id.clone()
    }

    fn timestamp(&self) -> Option<OffsetDateTime> {
        self.timestamp.as_deref().and_then(|value| {
            OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok()
        })
    }

    fn last_prompt(&self) -> Option<&str> {
        self.last_prompt.as_deref()
    }

    fn cwd(&self) -> Option<String> {
        self.cwd.clone()
    }

    fn user_text(&self) -> Option<&str> {
        if self.entry_type.as_deref() == Some("user") {
            self.message
                .as_ref()
                .and_then(|message| message.content.as_deref())
        } else {
            None
        }
    }

    fn usage(&self) -> Option<&ClaudeUsage> {
        self.usage.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSessionMetadata {
    pub session_id: String,
    pub provider: Option<String>,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub runtime_seconds: Option<u64>,
    pub token_usage: Option<u64>,
    pub context_length: Option<u64>,
    pub message_count: Option<u64>,
}

pub fn normalize_session(raw: RawSessionMetadata) -> Session {
    let metadata_state = if raw.title.is_some()
        && raw.summary.is_some()
        && raw.runtime_seconds.is_some()
        && raw.token_usage.is_some()
        && raw.context_length.is_some()
        && raw.message_count.is_some()
    {
        MetadataState::Complete
    } else {
        MetadataState::Partial
    };

    Session {
        session_id: raw.session_id,
        provider: raw.provider,
        source: None,
        activity_state: None,
        cwd: None,
        match_reason: None,
        dedupe_key: None,
        hidden_duplicates: 0,
        title: raw.title,
        summary: raw.summary,
        status: None,
        started_at: None,
        last_active_at: None,
        runtime_seconds: raw.runtime_seconds,
        token_usage: raw.token_usage,
        context_length: raw.context_length,
        message_count: raw.message_count,
        metadata_state,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ClaudeDiscovery, CodexDiscovery, RawSessionMetadata, SessionDiscovery,
        best_matching_process_index, codex_processes, deduplicate_claude_sessions,
        deduplicate_codex_sessions, is_active_session, matches_live_codex_process,
        normalize_session,
    };
    use crate::model::MetadataState;
    use std::fs;
    use time::macros::datetime;

    #[test]
    fn normalizes_complete_metadata_into_session() {
        let session = normalize_session(RawSessionMetadata {
            session_id: "session-1".to_string(),
            provider: Some("codex".to_string()),
            title: Some("Local Session".to_string()),
            summary: Some("Reviewing tasks".to_string()),
            runtime_seconds: Some(300),
            token_usage: Some(1200),
            context_length: Some(64000),
            message_count: Some(12),
        });

        assert_eq!(session.session_id, "session-1");
        assert_eq!(session.provider.as_deref(), Some("codex"));
        assert_eq!(session.title.as_deref(), Some("Local Session"));
        assert_eq!(session.metadata_state, MetadataState::Complete);
    }

    #[test]
    fn missing_fields_stay_optional_and_mark_session_partial() {
        let session = normalize_session(RawSessionMetadata {
            session_id: "session-2".to_string(),
            provider: Some("claude-code".to_string()),
            title: None,
            summary: Some("Working".to_string()),
            runtime_seconds: None,
            token_usage: None,
            context_length: Some(128),
            message_count: None,
        });

        assert!(session.title.is_none());
        assert!(session.runtime_seconds.is_none());
        assert_eq!(session.metadata_state, MetadataState::Partial);
    }

    #[test]
    fn claude_discovery_reads_local_jsonl_sessions() {
        let temp_root = std::env::temp_dir().join(format!("aisess-test-{}", std::process::id()));
        let project_dir = temp_root.join("project-a");
        fs::create_dir_all(&project_dir).unwrap();
        let session_file = project_dir.join("session.jsonl");
        fs::write(
            &session_file,
            concat!(
                "{\"type\":\"user\",\"message\":{\"content\":\"Investigate refresh behavior\"},\"usage\":{\"input_tokens\":120,\"output_tokens\":30},\"sessionId\":\"abc\",\"timestamp\":\"2026-05-25T10:00:00Z\"}\n",
                "{\"type\":\"last-prompt\",\"lastPrompt\":\"Investigate refresh behavior\",\"sessionId\":\"abc\",\"timestamp\":\"2026-05-25T10:00:05Z\"}\n"
            ),
        )
        .unwrap();

        let discovery = ClaudeDiscovery::with_active_window(temp_root.clone(), 60 * 60 * 24 * 365);
        let sessions = discovery.discover().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "abc");
        assert_eq!(sessions[0].provider.as_deref(), Some("claude-code"));
        assert_eq!(
            sessions[0].summary.as_deref(),
            Some("Investigate refresh behavior")
        );
        assert_eq!(sessions[0].message_count, Some(1));
        assert_eq!(sessions[0].token_usage, Some(150));

        fs::remove_dir_all(temp_root).unwrap();
    }

    #[test]
    fn active_session_filter_keeps_only_recently_updated_sessions() {
        let mut session = crate::model::Session::minimal("active");
        session.last_active_at = Some(datetime!(2026-05-25 10:10:00 UTC));

        let mut stale = crate::model::Session::minimal("stale");
        stale.last_active_at = Some(datetime!(2026-05-25 09:00:00 UTC));

        let now = datetime!(2026-05-25 10:15:00 UTC);

        assert!(is_active_session(&session, now, 15 * 60));
        assert!(!is_active_session(&stale, now, 15 * 60));
    }

    #[test]
    fn codex_discovery_reads_local_jsonl_sessions() {
        let temp_root =
            std::env::temp_dir().join(format!("aisess-codex-test-{}", std::process::id()));
        let project_dir = temp_root.join("2026/05/25");
        fs::create_dir_all(&project_dir).unwrap();
        let session_file = project_dir.join("rollout-test.jsonl");
        fs::write(
            &session_file,
            concat!(
                "{\"timestamp\":\"2026-05-25T10:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"codex-1\",\"cwd\":\"/tmp/demo\"}}\n",
                "{\"timestamp\":\"2026-05-25T10:00:00Z\",\"type\":\"turn_context\",\"payload\":{\"cwd\":\"/tmp/demo\",\"model\":\"gpt-5\",\"summary\":\"Implement session monitor\",\"token_usage\":{\"total\":2048,\"context_length\":8192}}}\n",
                "{\"timestamp\":\"2026-05-25T10:00:01Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"user\",\"content\":[{\"type\":\"input_text\",\"text\":\"Implement session monitor\"}]}}\n",
                "{\"timestamp\":\"2026-05-25T10:00:02Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"agent_message\",\"message\":\"Started implementation\"}}\n"
            ),
        )
        .unwrap();

        let discovery = CodexDiscovery::with_active_window(temp_root.clone(), 60 * 60 * 24 * 365);
        let sessions = discovery.discover().unwrap();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "codex-1");
        assert_eq!(sessions[0].provider.as_deref(), Some("codex"));
        assert_eq!(sessions[0].cwd.as_deref(), Some("/tmp/demo"));
        assert_eq!(
            sessions[0].summary.as_deref(),
            Some("Started implementation")
        );
        assert_eq!(sessions[0].token_usage, Some(2048));
        assert_eq!(sessions[0].context_length, Some(8192));

        fs::remove_dir_all(temp_root).unwrap();
    }

    #[test]
    fn live_codex_process_match_uses_title_or_summary() {
        let mut session = crate::model::Session::minimal("codex-1");
        session.cwd = Some("/Users/rj/Code/aisess".to_string());
        session.title = Some("aisess".to_string());
        session.summary = Some("implement session monitor".to_string());

        let processes = vec![super::CodexProcessInfo {
            pid: 42,
            tty: "ttys003".to_string(),
            command: "node /usr/local/bin/codex /Users/rj/Code/aisess".to_string(),
            cwd: Some("/Users/rj/Code/aisess".to_string()),
        }];

        assert!(matches_live_codex_process(&session, &processes));
    }

    #[test]
    fn process_listing_function_runs_without_crashing() {
        let _ = codex_processes().unwrap_or_default();
    }

    #[test]
    fn deduplication_keeps_only_one_running_session_per_process_match() {
        let mut first = crate::model::Session::minimal("one");
        first.provider = Some("codex".to_string());
        first.source = Some("codex-cli".to_string());
        first.activity_state = Some("running".to_string());
        first.cwd = Some("/Users/rj/Code/power".to_string());
        first.title = Some("power".to_string());
        first.last_active_at = Some(datetime!(2026-05-25 10:15:00 UTC));

        let mut second = crate::model::Session::minimal("two");
        second.provider = Some("codex".to_string());
        second.source = Some("codex-cli".to_string());
        second.activity_state = Some("running".to_string());
        second.cwd = Some("/Users/rj/Code/power".to_string());
        second.title = Some("power".to_string());
        second.last_active_at = Some(datetime!(2026-05-25 10:10:00 UTC));

        let processes = vec![super::CodexProcessInfo {
            pid: 42,
            tty: "ttys003".to_string(),
            command: "node /usr/local/bin/codex /Users/rj/Code/power".to_string(),
            cwd: Some("/Users/rj/Code/power".to_string()),
        }];

        let sessions = deduplicate_codex_sessions(vec![first, second], &processes);

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "one");
    }

    #[test]
    fn deduplication_hides_duplicate_running_sessions_for_same_project() {
        let mut newest = crate::model::Session::minimal("newest");
        newest.provider = Some("codex".to_string());
        newest.source = Some("codex-cli".to_string());
        newest.activity_state = Some("running".to_string());
        newest.cwd = Some("/Users/rj/Code/aisess".to_string());
        newest.title = Some("aisess".to_string());
        newest.last_active_at = Some(datetime!(2026-05-25 10:20:00 UTC));

        let mut older = crate::model::Session::minimal("older");
        older.provider = Some("codex".to_string());
        older.source = Some("codex-cli".to_string());
        older.activity_state = Some("running".to_string());
        older.cwd = Some("/Users/rj/Code/aisess".to_string());
        older.title = Some("aisess".to_string());
        older.last_active_at = Some(datetime!(2026-05-25 10:10:00 UTC));

        let processes = vec![
            super::CodexProcessInfo {
                pid: 42,
                tty: "ttys003".to_string(),
                command: "node /usr/local/bin/codex /Users/rj/Code/aisess".to_string(),
                cwd: Some("/Users/rj/Code/aisess".to_string()),
            },
            super::CodexProcessInfo {
                pid: 43,
                tty: "ttys004".to_string(),
                command: "node /usr/local/bin/codex /Users/rj/Code/aisess".to_string(),
                cwd: Some("/Users/rj/Code/aisess".to_string()),
            },
        ];

        let sessions = deduplicate_codex_sessions(vec![older, newest], &processes);

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "newest");
    }

    #[test]
    fn claude_parsing_keeps_real_cwd_for_live_matching() {
        let temp_root =
            std::env::temp_dir().join(format!("aisess-claude-cwd-test-{}", std::process::id()));
        let project_dir = temp_root.join("-Users-rj-Code-aisess");
        fs::create_dir_all(&project_dir).unwrap();
        let session_file = project_dir.join("session.jsonl");
        fs::write(
            &session_file,
            concat!(
                "{\"type\":\"user\",\"cwd\":\"/Users/rj/Code/aisess\",\"message\":{\"content\":\"Check matching\"},\"sessionId\":\"claude-1\",\"timestamp\":\"2026-05-25T10:00:00Z\"}\n",
                "{\"type\":\"last-prompt\",\"cwd\":\"/Users/rj/Code/aisess\",\"lastPrompt\":\"Check matching\",\"sessionId\":\"claude-1\",\"timestamp\":\"2026-05-25T10:00:05Z\"}\n"
            ),
        )
        .unwrap();

        let session = ClaudeDiscovery::parse_session_file(&session_file)
            .unwrap()
            .unwrap();
        let processes = vec![super::ClaudeProcessInfo {
            pid: 42,
            tty: "ttys003".to_string(),
            command: "claude".to_string(),
            cwd: Some("/Users/rj/Code/aisess".to_string()),
        }];

        assert_eq!(session.cwd.as_deref(), Some("/Users/rj/Code/aisess"));
        assert!(super::matches_live_claude_process(&session, &processes));

        fs::remove_dir_all(temp_root).unwrap();
    }

    #[test]
    fn claude_deduplication_hides_duplicate_running_sessions_for_same_project() {
        let mut newest = crate::model::Session::minimal("claude-newest");
        newest.provider = Some("claude-code".to_string());
        newest.source = Some("claude-local".to_string());
        newest.activity_state = Some("running".to_string());
        newest.cwd = Some("/Users/rj/Code/aisess".to_string());
        newest.title = Some("aisess".to_string());
        newest.last_active_at = Some(datetime!(2026-05-25 10:20:00 UTC));

        let mut older = crate::model::Session::minimal("claude-older");
        older.provider = Some("claude-code".to_string());
        older.source = Some("claude-local".to_string());
        older.activity_state = Some("running".to_string());
        older.cwd = Some("/Users/rj/Code/aisess".to_string());
        older.title = Some("aisess".to_string());
        older.last_active_at = Some(datetime!(2026-05-25 10:05:00 UTC));

        let sessions = deduplicate_claude_sessions(vec![older, newest]);

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "claude-newest");
    }

    #[test]
    fn best_process_match_prefers_project_name_match() {
        let mut session = crate::model::Session::minimal("one");
        session.title = Some("aisess".to_string());

        let processes = vec![super::CodexProcessInfo {
            pid: 42,
            tty: "ttys003".to_string(),
            command: "node /usr/local/bin/codex /Users/rj/Code/aisess".to_string(),
            cwd: Some("/Users/rj/Code/aisess".to_string()),
        }];

        assert_eq!(best_matching_process_index(&session, &processes), Some(0));
    }
}
