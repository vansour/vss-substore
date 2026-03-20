mod actions;
mod auth;
mod cache;
mod diagnostics;
mod editor;
mod overview;
mod services;
mod state;
mod users;
mod view;

use dioxus::prelude::*;

use crate::components::shell::AppShell;
use auth::{AccountPanel, ControlPlanePanel, LoginPanel};
use cache::CachePanel;
use diagnostics::DiagnosticsPanel;
use editor::EditorPanel;
use overview::UserOverview;
use state::{
    has_unsaved_links, optional_resource_snapshot, resource_snapshot, sync_links_text,
    use_console_resources, use_feedback_signals, use_link_draft_state, use_pending_state,
    use_refresh_state,
};
use users::UsersPanel;
use view::{ConsolePageMeta, UserSummary};

#[component]
pub fn AdminConsole(mode: &'static str, route_user: Option<String>) -> Element {
    let login_username = use_signal(String::new);
    let login_password = use_signal(String::new);
    let create_username = use_signal(String::new);
    let links_text = use_signal(String::new);
    let account_username = use_signal(String::new);
    let account_current_password = use_signal(String::new);
    let account_new_password = use_signal(String::new);

    let feedback = use_feedback_signals();
    let link_drafts = use_link_draft_state();
    let pending = use_pending_state();
    let refresh = use_refresh_state();
    let resources = use_console_resources(route_user.clone(), refresh);
    sync_links_text(
        route_user.clone(),
        links_text,
        resources.links_resource,
        link_drafts,
    );

    let current_user = optional_resource_snapshot(&resources.auth_resource);
    let users = resource_snapshot(&resources.users_resource);
    let cache_status = optional_resource_snapshot(&resources.cache_resource);
    let diagnostics = optional_resource_snapshot(&resources.diagnostics_resource);

    let page = ConsolePageMeta::build(mode, route_user, current_user.value.clone());
    let user_count = users.value.as_ref().map(Vec::len).unwrap_or_default();
    let current_links_text = links_text();
    let has_unsaved_changes = has_unsaved_links(
        page.selected_username.as_deref(),
        &current_links_text,
        link_drafts,
    );
    let user_summary = UserSummary::build(
        page.selected_username.clone(),
        user_count,
        &current_links_text,
        cache_status.value.as_ref(),
        diagnostics.value.as_ref(),
    );
    let has_selected_user = user_summary.is_some();

    rsx! {
        AppShell {
            title: page.shell_title.clone(),
            summary: page.shell_summary.clone(),
            compact: !page.is_authenticated,
            active_mode: if page.is_authenticated { Some(page.active_mode) } else { None },
            selected_user: page.selected_username.clone(),
            if let Some(message) = (feedback.status_message)() {
                article {
                    class: "notice notice--success",
                    role: "status",
                    "aria-live": "polite",
                    "aria-atomic": "true",
                    div {
                        strong { "操作完成" }
                        p { "{message}" }
                    }
                }
            }
            if let Some(message) = (feedback.error_message)() {
                article {
                    class: "notice notice--error",
                    role: "alert",
                    "aria-live": "assertive",
                    "aria-atomic": "true",
                    div {
                        strong { "操作失败" }
                        p { "{message}" }
                    }
                }
            }
            if let Some(username) = current_user.value.clone().map(|user| user.username) {
                if page.active_mode == "account" {
                    div { class: "account-shell",
                        ControlPlanePanel {
                            username,
                            selected_username: page.selected_username.clone(),
                            show_selection: false,
                            links_text,
                            pending,
                            feedback,
                            refresh,
                        }
                        AccountPanel {
                            account_username,
                            account_current_password,
                            account_new_password,
                            current_username: page.current_username.clone(),
                            pending,
                            feedback,
                            refresh,
                        }
                    }
                } else {
                    div { class: "console-layout",
                        if has_selected_user {
                            aside { class: "console-sidebar",
                                ControlPlanePanel {
                                    username,
                                    selected_username: page.selected_username.clone(),
                                    show_selection: true,
                                    links_text,
                                    pending,
                                    feedback,
                                    refresh,
                                }
                                UsersPanel {
                                    create_username,
                                    users: users.value.clone(),
                                    selected_username: page.selected_username.clone(),
                                    pending,
                                    feedback,
                                    refresh,
                                }
                            }
                            section { class: "console-main",
                                if let Some(user_summary) = user_summary.clone() {
                                    UserOverview { summary: user_summary.clone() }
                                    div { class: "workspace-canvas",
                                        div { class: "workspace-primary",
                                            EditorPanel {
                                                username: user_summary.selected_username.clone(),
                                                selected_route: user_summary.selected_route.clone(),
                                                links_text,
                                                selected_link_count: user_summary.selected_link_count,
                                                drafts: link_drafts,
                                                has_unsaved_changes,
                                                pending,
                                                feedback,
                                                refresh,
                                            }
                                        }
                                        aside { class: "console-support",
                                            CachePanel {
                                                username: user_summary.selected_username.clone(),
                                                cache: user_summary.cache_display.clone(),
                                                cache_error: cache_status.error.clone(),
                                                pending,
                                                feedback,
                                                refresh,
                                            }
                                            DiagnosticsPanel {
                                                diagnostics: diagnostics.value.clone(),
                                                diagnostics_error: diagnostics.error.clone(),
                                                success_count: user_summary.success_count,
                                                error_count: user_summary.error_count,
                                                blocked_count: user_summary.blocked_count,
                                                pending_count: user_summary.pending_count,
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            aside { class: "console-sidebar",
                                ControlPlanePanel {
                                    username,
                                    selected_username: page.selected_username.clone(),
                                    show_selection: true,
                                    links_text,
                                    pending,
                                    feedback,
                                    refresh,
                                }
                                UsersPanel {
                                    create_username,
                                    users: users.value.clone(),
                                    selected_username: page.selected_username.clone(),
                                    pending,
                                    feedback,
                                    refresh,
                                }
                            }
                            section { class: "console-main",
                                article { class: "panel panel--editor panel--empty",
                                    div { class: "section-head",
                                        div {
                                            p { class: "eyebrow", "订阅组" }
                                            h2 { "请先选择一个订阅组" }
                                            p { class: "muted", "从左侧列表选择已有订阅组，或新建一个订阅组后开始配置。" }
                                        }
                                        span { class: "tag", "共 {user_count} 个订阅组" }
                                    }
                                    div { class: "empty-state empty-user__copy",
                                        strong { "订阅组待选择" }
                                        p { "选中订阅组后，这里会展示对应的源链接、缓存状态和抓取诊断信息。" }
                                    }
                                    if users.value.is_some() {
                                        p { class: "muted empty-user__note",
                                            "左侧订阅组列表是唯一入口：可直接进入已有订阅组，或在新建订阅组表单中创建后进入。"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                LoginPanel {
                    login_username,
                    login_password,
                    pending,
                    feedback,
                    refresh,
                }
            }
        }
    }
}
