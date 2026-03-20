use std::collections::HashMap;

use dioxus::prelude::*;
use submora_shared::{
    auth::CurrentUserResponse,
    users::{UserCacheStatusResponse, UserDiagnosticsResponse, UserLinksResponse, UserSummary},
};
use time::{OffsetDateTime, format_description::parse};

use super::services;

#[derive(Clone, Copy, PartialEq)]
pub struct FeedbackSignals {
    pub status_message: Signal<Option<String>>,
    pub error_message: Signal<Option<String>>,
}

impl FeedbackSignals {
    pub fn clear(mut self) {
        self.error_message.set(None);
        self.status_message.set(None);
    }

    pub fn set_status(mut self, message: impl Into<String>) {
        self.status_message.set(Some(message.into()));
    }

    pub fn set_error(mut self, message: String) {
        self.error_message.set(Some(message));
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct RefreshState {
    pub auth: Signal<u32>,
    pub users: Signal<u32>,
    pub links: Signal<u32>,
    pub diagnostics: Signal<u32>,
    pub cache: Signal<u32>,
}

impl RefreshState {
    pub fn bump_auth(mut self) {
        self.auth.set((self.auth)() + 1);
    }

    pub fn bump_users(mut self) {
        self.users.set((self.users)() + 1);
    }

    pub fn bump_links(mut self) {
        self.links.set((self.links)() + 1);
    }

    pub fn bump_diagnostics(mut self) {
        self.diagnostics.set((self.diagnostics)() + 1);
    }

    pub fn bump_cache(mut self) {
        self.cache.set((self.cache)() + 1);
    }

    pub fn bump_selected_data(self) {
        self.bump_links();
        self.bump_diagnostics();
        self.bump_cache();
    }

    pub fn bump_after_auth_change(self) {
        self.bump_auth();
        self.bump_users();
        self.bump_selected_data();
    }

    pub fn bump_after_user_change(self) {
        self.bump_users();
        self.bump_selected_data();
    }

    pub fn bump_after_cache_refresh(self) {
        self.bump_cache();
        self.bump_diagnostics();
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct PendingState {
    pub login: Signal<bool>,
    pub logout: Signal<bool>,
    pub account_update: Signal<bool>,
    pub create_user: Signal<bool>,
    pub reorder_users: Signal<bool>,
    pub save_links: Signal<bool>,
    pub delete_user: Signal<bool>,
    pub refresh_cache: Signal<bool>,
    pub clear_cache: Signal<bool>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct LinkDraftState {
    pub saved_by_user: Signal<HashMap<String, String>>,
    pub draft_by_user: Signal<HashMap<String, String>>,
    pub active_username: Signal<Option<String>>,
}

#[derive(Clone)]
pub struct ConsoleResources {
    pub auth_resource: Resource<Result<Option<CurrentUserResponse>, String>>,
    pub users_resource: Resource<Result<Vec<UserSummary>, String>>,
    pub links_resource: Resource<Result<Option<UserLinksResponse>, String>>,
    pub diagnostics_resource: Resource<Result<Option<UserDiagnosticsResponse>, String>>,
    pub cache_resource: Resource<Result<Option<UserCacheStatusResponse>, String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CacheDisplay {
    pub state: String,
    pub line_count: u32,
    pub body_bytes: u64,
    pub generated_at: String,
    pub expires_at: String,
}

#[derive(Clone, Debug)]
pub struct ResourceSnapshot<T> {
    pub value: Option<T>,
    pub error: Option<String>,
}

impl<T> Default for ResourceSnapshot<T> {
    fn default() -> Self {
        Self {
            value: None,
            error: None,
        }
    }
}

impl CacheDisplay {
    pub fn from_status(status: Option<&UserCacheStatusResponse>) -> Self {
        Self {
            state: status
                .map(|status| status.state.clone())
                .unwrap_or_else(|| "empty".to_string()),
            line_count: status.map(|status| status.line_count).unwrap_or_default(),
            body_bytes: status.map(|status| status.body_bytes).unwrap_or_default(),
            generated_at: format_timestamp(
                status.and_then(|status| status.generated_at),
                "尚未生成",
            ),
            expires_at: format_timestamp(status.and_then(|status| status.expires_at), "不适用"),
        }
    }

    pub fn state_class(&self) -> &'static str {
        match self.state.as_str() {
            "fresh" => "tag--success",
            "expired" | "stale" => "tag--danger",
            _ => "tag--cool",
        }
    }

    pub fn state_label(&self) -> &'static str {
        match self.state.as_str() {
            "fresh" => "新鲜",
            "expired" => "已过期",
            "stale" => "陈旧",
            "empty" => "为空",
            _ => "未知",
        }
    }
}

pub fn use_feedback_signals() -> FeedbackSignals {
    FeedbackSignals {
        status_message: use_signal(|| None::<String>),
        error_message: use_signal(|| None::<String>),
    }
}

pub fn use_refresh_state() -> RefreshState {
    RefreshState {
        auth: use_signal(|| 0u32),
        users: use_signal(|| 0u32),
        links: use_signal(|| 0u32),
        diagnostics: use_signal(|| 0u32),
        cache: use_signal(|| 0u32),
    }
}

pub fn use_pending_state() -> PendingState {
    PendingState {
        login: use_signal(|| false),
        logout: use_signal(|| false),
        account_update: use_signal(|| false),
        create_user: use_signal(|| false),
        reorder_users: use_signal(|| false),
        save_links: use_signal(|| false),
        delete_user: use_signal(|| false),
        refresh_cache: use_signal(|| false),
        clear_cache: use_signal(|| false),
    }
}

pub fn use_link_draft_state() -> LinkDraftState {
    LinkDraftState {
        saved_by_user: use_signal(HashMap::new),
        draft_by_user: use_signal(HashMap::new),
        active_username: use_signal(|| None::<String>),
    }
}

pub fn use_console_resources(
    selected_username: Option<String>,
    refresh: RefreshState,
) -> ConsoleResources {
    let selected_username_for_links = selected_username.clone();
    let selected_username_for_diagnostics = selected_username.clone();
    let selected_username_for_cache = selected_username;

    let auth_resource = use_resource(move || async move {
        let _ = (refresh.auth)();
        services::load_current_user().await
    });
    let users_resource = use_resource(move || async move {
        let _ = (refresh.users)();
        services::load_users().await
    });
    let links_resource = use_resource(use_reactive!(|(selected_username_for_links,)| async move {
        let _ = (refresh.links)();
        services::load_links(selected_username_for_links.clone()).await
    }));
    let diagnostics_resource = use_resource(use_reactive!(|(
        selected_username_for_diagnostics,
    )| async move {
        let _ = (refresh.diagnostics)();
        services::load_diagnostics(selected_username_for_diagnostics.clone()).await
    }));
    let cache_resource = use_resource(use_reactive!(|(selected_username_for_cache,)| async move {
        let _ = (refresh.cache)();
        services::load_cache_status(selected_username_for_cache.clone()).await
    }));

    ConsoleResources {
        auth_resource,
        users_resource,
        links_resource,
        diagnostics_resource,
        cache_resource,
    }
}

pub fn resource_snapshot<T: Clone>(resource: &Resource<Result<T, String>>) -> ResourceSnapshot<T> {
    match &*resource.read_unchecked() {
        Some(Ok(value)) => ResourceSnapshot {
            value: Some(value.clone()),
            error: None,
        },
        Some(Err(error)) => ResourceSnapshot {
            value: None,
            error: Some(error.clone()),
        },
        None => ResourceSnapshot::default(),
    }
}

pub fn optional_resource_snapshot<T: Clone>(
    resource: &Resource<Result<Option<T>, String>>,
) -> ResourceSnapshot<T> {
    match &*resource.read_unchecked() {
        Some(Ok(Some(value))) => ResourceSnapshot {
            value: Some(value.clone()),
            error: None,
        },
        Some(Ok(None)) => ResourceSnapshot::default(),
        Some(Err(error)) => ResourceSnapshot {
            value: None,
            error: Some(error.clone()),
        },
        None => ResourceSnapshot::default(),
    }
}

pub fn sync_links_text(
    selected_username: Option<String>,
    mut links_text: Signal<String>,
    links_resource: Resource<Result<Option<UserLinksResponse>, String>>,
    drafts: LinkDraftState,
) {
    {
        let selected_username = selected_username.clone();
        let mut selection_drafts = drafts;
        use_effect(use_reactive!(|(selected_username,)| {
            if (selection_drafts.active_username)() == selected_username {
                return;
            }

            selection_drafts
                .active_username
                .set(selected_username.clone());

            let next_text = selected_username
                .as_deref()
                .map(|username| {
                    display_links_for_user(
                        username,
                        &(selection_drafts.saved_by_user)(),
                        &(selection_drafts.draft_by_user)(),
                    )
                })
                .unwrap_or_default();

            if links_text() != next_text {
                links_text.set(next_text);
            }
        }));
    }

    let mut resource_drafts = drafts;
    use_effect(move || {
        let payload = links_resource
            .read_unchecked()
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .and_then(|payload| payload.as_ref())
            .cloned();

        if let Some(UserLinksResponse { username, links }) = payload {
            let saved_text = links.join("\n");

            let mut saved_by_user = (resource_drafts.saved_by_user)();
            if saved_by_user.get(&username) != Some(&saved_text) {
                saved_by_user.insert(username.clone(), saved_text.clone());
                resource_drafts.saved_by_user.set(saved_by_user.clone());
            }

            let mut draft_by_user = (resource_drafts.draft_by_user)();
            if draft_by_user.get(&username) == Some(&saved_text) {
                draft_by_user.remove(&username);
                resource_drafts.draft_by_user.set(draft_by_user.clone());
            }

            if (resource_drafts.active_username)().as_deref() == Some(username.as_str()) {
                let next_text = display_links_for_user(&username, &saved_by_user, &draft_by_user);
                if links_text() != next_text {
                    links_text.set(next_text);
                }
            }
        }
    });
}

pub fn remember_links_input(username: &str, next_text: &str, mut drafts: LinkDraftState) {
    let saved_by_user = (drafts.saved_by_user)();
    let mut draft_by_user = (drafts.draft_by_user)();
    update_draft_map(&mut draft_by_user, &saved_by_user, username, next_text);
    drafts.draft_by_user.set(draft_by_user);
}

pub fn mark_links_saved(username: &str, saved_text: &str, mut drafts: LinkDraftState) {
    let mut saved_by_user = (drafts.saved_by_user)();
    saved_by_user.insert(username.to_string(), saved_text.to_string());
    drafts.saved_by_user.set(saved_by_user);

    let mut draft_by_user = (drafts.draft_by_user)();
    draft_by_user.remove(username);
    drafts.draft_by_user.set(draft_by_user);
}

pub fn clear_links_state_for_user(username: &str, mut drafts: LinkDraftState) {
    let mut saved_by_user = (drafts.saved_by_user)();
    saved_by_user.remove(username);
    drafts.saved_by_user.set(saved_by_user);

    let mut draft_by_user = (drafts.draft_by_user)();
    draft_by_user.remove(username);
    drafts.draft_by_user.set(draft_by_user);
}

pub fn has_unsaved_links(
    selected_username: Option<&str>,
    current_links_text: &str,
    drafts: LinkDraftState,
) -> bool {
    let Some(username) = selected_username else {
        return false;
    };

    let saved_by_user = (drafts.saved_by_user)();
    current_links_text
        != saved_by_user
            .get(username)
            .map(String::as_str)
            .unwrap_or_default()
}

pub fn format_timestamp(value: Option<i64>, empty: &str) -> String {
    let Ok(display_format) = parse("[year]-[month]-[day] [hour]:[minute] UTC") else {
        return value.map_or_else(|| empty.to_string(), |timestamp| timestamp.to_string());
    };

    match value {
        Some(value) => match OffsetDateTime::from_unix_timestamp(value) {
            Ok(timestamp) => timestamp
                .format(&display_format)
                .unwrap_or_else(|_| value.to_string()),
            Err(_) => value.to_string(),
        },
        None => empty.to_string(),
    }
}

fn display_links_for_user(
    username: &str,
    saved_by_user: &HashMap<String, String>,
    draft_by_user: &HashMap<String, String>,
) -> String {
    draft_by_user
        .get(username)
        .cloned()
        .or_else(|| saved_by_user.get(username).cloned())
        .unwrap_or_default()
}

fn update_draft_map(
    draft_by_user: &mut HashMap<String, String>,
    saved_by_user: &HashMap<String, String>,
    username: &str,
    next_text: &str,
) {
    let saved_text = saved_by_user
        .get(username)
        .map(String::as_str)
        .unwrap_or_default();

    if next_text == saved_text {
        draft_by_user.remove(username);
    } else {
        draft_by_user.insert(username.to_string(), next_text.to_string());
    }
}

