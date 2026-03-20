use std::future::Future;

use dioxus::prelude::*;
use submora_shared::users::{UserLinksResponse, UserSummary};

use crate::messages::extract_field_validation_error;

use super::{
    services,
    state::{
        FeedbackSignals, LinkDraftState, RefreshState, clear_links_state_for_user, mark_links_saved,
    },
};

fn spawn_pending<Fut>(mut pending: Signal<bool>, future: Fut)
where
    Fut: Future<Output = ()> + 'static,
{
    pending.set(true);
    spawn(async move {
        future.await;
        pending.set(false);
    });
}

pub fn logout_session(
    mut links_text: Signal<String>,
    pending: Signal<bool>,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) {
    if pending() {
        return;
    }

    feedback.clear();
    spawn_pending(pending, async move {
        match services::logout().await {
            Ok(message) => {
                feedback.set_status(message);
                links_text.set(String::new());
                refresh.bump_after_auth_change();
            }
            Err(error) => feedback.set_error(error),
        }
    });
}

pub fn submit_login(
    username: String,
    password: String,
    mut login_password: Signal<String>,
    pending: Signal<bool>,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) {
    if pending() {
        return;
    }

    feedback.clear();
    spawn_pending(pending, async move {
        match services::login(username, password).await {
            Ok(_) => {
                login_password.set(String::new());
                refresh.bump_after_auth_change();
            }
            Err(error) => feedback.set_error(error),
        }
    });
}

pub fn submit_account_update(
    current_username: String,
    next_username: String,
    current_password: String,
    new_password: String,
    mut account_username: Signal<String>,
    mut account_current_password: Signal<String>,
    mut account_new_password: Signal<String>,
    pending: Signal<bool>,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) {
    if pending() {
        return;
    }

    feedback.clear();
    spawn_pending(pending, async move {
        match services::update_account(
            current_username,
            next_username,
            current_password,
            new_password,
        )
        .await
        {
            Ok(message) => {
                feedback.set_status(message);
                account_username.set(String::new());
                account_current_password.set(String::new());
                account_new_password.set(String::new());
                refresh.bump_after_auth_change();
            }
            Err(error) => feedback.set_error(error),
        }
    });
}

pub fn create_user_and_open<F>(
    username: String,
    mut create_username: Signal<String>,
    pending: Signal<bool>,
    feedback: FeedbackSignals,
    refresh: RefreshState,
    open_user: F,
) where
    F: FnOnce(String) + 'static,
{
    if pending() {
        return;
    }

    feedback.clear();
    spawn_pending(pending, async move {
        match services::create_user(username).await {
            Ok(user) => {
                let username = user.username;
                feedback.set_status(format!("已创建订阅组 {username}"));
                create_username.set(String::new());
                refresh.bump_after_user_change();
                open_user(username);
            }
            Err(error) => feedback.set_error(error),
        }
    });
}

pub fn submit_user_order(
    order: Vec<String>,
    pending: Signal<bool>,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) {
    if pending() {
        return;
    }

    feedback.clear();
    spawn_pending(pending, async move {
        match services::set_order(order).await {
            Ok(_) => {
                feedback.set_status("已更新订阅组顺序");
                refresh.bump_users();
            }
            Err(error) => feedback.set_error(error),
        }
    });
}

pub fn reordered_usernames(
    users: &[UserSummary],
    username: &str,
    offset: isize,
) -> Option<Vec<String>> {
    let position = users.iter().position(|item| item.username == username)?;
    let target = position as isize + offset;

    if target < 0 || target >= users.len() as isize {
        return None;
    }

    let mut order = users
        .iter()
        .map(|item| item.username.clone())
        .collect::<Vec<_>>();
    order.swap(position, target as usize);
    Some(order)
}

pub fn move_username_to_edge(
    users: &[UserSummary],
    username: &str,
    to_start: bool,
) -> Option<Vec<String>> {
    let position = users.iter().position(|item| item.username == username)?;
    let target = if to_start {
        0
    } else {
        users.len().checked_sub(1)?
    };

    if position == target {
        return None;
    }

    let mut order = users
        .iter()
        .map(|item| item.username.clone())
        .collect::<Vec<_>>();
    let moved = order.remove(position);
    order.insert(target, moved);
    Some(order)
}

pub fn save_links(
    username: String,
    next_links: String,
    links_text: Signal<String>,
    mut links_error: Signal<Option<String>>,
    drafts: LinkDraftState,
    pending: Signal<bool>,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) {
    if pending() {
        return;
    }

    feedback.clear();
    links_error.set(None);
    let draft_stats = services::analyze_links(&next_links, 0);
    spawn_pending(pending, async move {
        match services::save_links(username, next_links).await {
            Ok(response) => apply_saved_links(
                response,
                draft_stats,
                links_text,
                links_error,
                drafts,
                feedback,
                refresh,
            ),
            Err(error) => {
                if let Some(message) = extract_field_validation_error(&error, "links", "链接") {
                    links_error.set(Some(message));
                } else {
                    feedback.set_error(error);
                }
            }
        }
    });
}

pub fn delete_user_and_leave<F>(
    username: String,
    mut links_text: Signal<String>,
    drafts: LinkDraftState,
    pending: Signal<bool>,
    feedback: FeedbackSignals,
    refresh: RefreshState,
    on_deleted: F,
) where
    F: FnOnce() + 'static,
{
    if pending() {
        return;
    }

    feedback.clear();
    spawn_pending(pending, async move {
        let deleted_username = username.clone();
        match services::delete_user(username).await {
            Ok(message) => {
                feedback.set_status(message);
                links_text.set(String::new());
                clear_links_state_for_user(&deleted_username, drafts);
                refresh.bump_after_user_change();
                on_deleted();
            }
            Err(error) => feedback.set_error(error),
        }
    });
}

pub fn refresh_cache(
    username: String,
    pending: Signal<bool>,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) {
    if pending() {
        return;
    }

    feedback.clear();
    spawn_pending(pending, async move {
        match services::refresh_cache(username).await {
            Ok(status) => {
                feedback.set_status(format!(
                    "已刷新 {} 的缓存，共 {} 行",
                    status.username, status.line_count
                ));
                refresh.bump_after_cache_refresh();
            }
            Err(error) => feedback.set_error(error),
        }
    });
}

pub fn clear_cache(
    username: String,
    pending: Signal<bool>,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) {
    if pending() {
        return;
    }

    feedback.clear();
    spawn_pending(pending, async move {
        match services::clear_cache(username).await {
            Ok(message) => {
                feedback.set_status(message);
                refresh.bump_cache();
            }
            Err(error) => feedback.set_error(error),
        }
    });
}

fn apply_saved_links(
    response: UserLinksResponse,
    draft_stats: services::DraftLinkStats,
    mut links_text: Signal<String>,
    mut links_error: Signal<Option<String>>,
    drafts: LinkDraftState,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) {
    let normalized_links = response.links.join("\n");
    mark_links_saved(&response.username, &normalized_links, drafts);
    links_error.set(None);
    links_text.set(normalized_links);
    feedback.set_status(saved_links_message(
        &response.username,
        response.links.len(),
        &draft_stats,
    ));
    refresh.bump_selected_data();
}

fn saved_links_message(
    username: &str,
    saved_count: usize,
    draft_stats: &services::DraftLinkStats,
) -> String {
    if saved_count == 0 {
        return format!("已清空 {username} 的源链接");
    }

    let mut normalized_changes = Vec::new();
    if draft_stats.duplicate_count > 0 {
        normalized_changes.push(format!("合并 {} 条重复", draft_stats.duplicate_count));
    }
    if draft_stats.blank_count > 0 {
        normalized_changes.push(format!("忽略 {} 行空白", draft_stats.blank_count));
    }

    if normalized_changes.is_empty() {
        format!("已保存 {username} 的源链接，共 {saved_count} 条")
    } else {
        format!(
            "已保存 {username} 的源链接，保留 {saved_count} 条，{}",
            normalized_changes.join("，")
        )
    }
}

