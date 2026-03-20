use submora_shared::{
    auth::CurrentUserResponse,
    users::{LinkDiagnostic, UserCacheStatusResponse, UserDiagnosticsResponse},
};

use super::{services, state::CacheDisplay};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsolePageMeta {
    pub active_mode: &'static str,
    pub current_username: String,
    pub selected_username: Option<String>,
    pub selected_route: String,
    pub shell_title: String,
    pub shell_summary: String,
    pub is_authenticated: bool,
}

impl ConsolePageMeta {
    pub fn build(
        mode: &'static str,
        route_user: Option<String>,
        current_user: Option<CurrentUserResponse>,
    ) -> Self {
        let is_authenticated = current_user.is_some();
        let active_mode = if is_authenticated && mode == "login" {
            "dashboard"
        } else {
            mode
        };
        let current_username = current_user.map(|user| user.username).unwrap_or_default();
        let selected_route = route_user
            .as_ref()
            .map(|username| format!("/{username}"))
            .unwrap_or_else(|| "/{username}".to_string());
        let title = match active_mode {
            "account" => "账户".to_string(),
            "user" => "订阅组详情".to_string(),
            _ => "订阅组管理".to_string(),
        };
        let summary = match active_mode {
            "account" => "更新管理员账号和登录密码。",
            "user" => "编辑订阅组源链接，查看公开订阅缓存与抓取诊断。",
            _ => "管理全部订阅组、源链接和公开订阅状态。",
        }
        .to_string();
        let shell_title = if is_authenticated {
            title
        } else {
            "登录".to_string()
        };
        let shell_summary = if is_authenticated {
            summary
        } else {
            "登录后即可管理全部订阅组、源链接和公开订阅状态。".to_string()
        };

        Self {
            active_mode,
            current_username,
            selected_username: route_user,
            selected_route,
            shell_title,
            shell_summary,
            is_authenticated,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserSummary {
    pub selected_username: String,
    pub selected_route: String,
    pub user_count: usize,
    pub selected_link_count: usize,
    pub cache_display: CacheDisplay,
    pub success_count: usize,
    pub error_count: usize,
    pub blocked_count: usize,
    pub pending_count: usize,
}

impl UserSummary {
    pub fn build(
        selected_username: Option<String>,
        user_count: usize,
        links_text: &str,
        cache_status: Option<&UserCacheStatusResponse>,
        diagnostics: Option<&UserDiagnosticsResponse>,
    ) -> Option<Self> {
        let selected_username = selected_username?;
        let (success_count, error_count, blocked_count, pending_count) =
            diagnostic_counts(diagnostics);

        Some(Self {
            selected_route: format!("/{selected_username}"),
            selected_username,
            user_count,
            selected_link_count: services::count_links(links_text),
            cache_display: CacheDisplay::from_status(cache_status),
            success_count,
            error_count,
            blocked_count,
            pending_count,
        })
    }

    pub fn attention_count(&self) -> usize {
        self.error_count + self.blocked_count + self.pending_count
    }

    pub fn cache_badge_class(&self) -> String {
        format!("tag {}", self.cache_display.state_class())
    }
}

fn diagnostic_counts(
    diagnostics: Option<&UserDiagnosticsResponse>,
) -> (usize, usize, usize, usize) {
    let diagnostics = diagnostics
        .map(|payload| payload.diagnostics.as_slice())
        .unwrap_or_default();

    (
        count_diagnostics(diagnostics, "success"),
        count_diagnostics(diagnostics, "error"),
        count_diagnostics(diagnostics, "blocked"),
        count_diagnostics(diagnostics, "pending"),
    )
}

fn count_diagnostics(diagnostics: &[LinkDiagnostic], status: &str) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.status == status)
        .count()
}

