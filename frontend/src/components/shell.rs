use dioxus::prelude::*;

use crate::app::Route;

#[component]
pub fn AppShell(
    title: String,
    summary: String,
    compact: bool,
    active_mode: Option<&'static str>,
    selected_user: Option<String>,
    children: Element,
) -> Element {
    let page_class = if compact {
        "shell-page shell-page--compact"
    } else {
        "shell-page shell-page--app"
    };
    let shell_class = if compact {
        "shell shell--compact"
    } else {
        "shell shell--app"
    };
    let content_class = if compact {
        "content content--compact"
    } else {
        "content"
    };
    let dashboard_link_class = route_link_class(active_mode, "dashboard");
    let user_link_class = route_link_class(active_mode, "user");
    let account_link_class = route_link_class(active_mode, "account");
    let spotlight_label = shell_spotlight_label(active_mode, selected_user.as_deref());
    let spotlight_title = selected_user.clone().unwrap_or_else(|| title.clone());
    let spotlight_summary = shell_spotlight_summary(active_mode, selected_user.as_deref(), &summary);
    let spotlight_route = selected_user.as_ref().map(|username| format!("/{username}"));
    let mode_label = shell_mode_label(active_mode);
    let mode_summary = shell_mode_summary(active_mode, selected_user.is_some());
    let selected_user_for_nav = selected_user.map(|username| {
        let label = topbar_workspace_label(&username);
        (username, label)
    });

    rsx! {
        div { class: "{page_class}",
            div { class: "ambient ambient--warm" }
            div { class: "ambient ambient--cool" }
            div { class: "{shell_class}",
                if !compact {
                    header { class: "panel panel--topbar shell-header",
                        div { class: "shell-header__grid",
                            div { class: "shell-header__hero",
                                div { class: "shell-header__brand",
                                    p { class: "eyebrow", "{submora_core::APP_NAME}" }
                                }
                                div { class: "shell-header__copy",
                                    h1 { class: "shell-header__title", "{title}" }
                                    p { class: "muted shell-header__summary", "{summary}" }
                                }
                                div { class: "shell-header__deck",
                                    article { class: "shell-deck-card",
                                        p { class: "eyebrow", "控制模式" }
                                        strong { "{mode_label}" }
                                        p { class: "muted", "{mode_summary}" }
                                    }
                                    article { class: "shell-deck-card",
                                        p { class: "eyebrow", "{spotlight_label}" }
                                        strong { "{spotlight_title}" }
                                        if let Some(route) = spotlight_route.clone() {
                                            code { "{route}" }
                                        } else {
                                            p { class: "muted", "当前页面没有对应的公开路由。" }
                                        }
                                    }
                                    article { class: "shell-deck-card shell-deck-card--signals",
                                        p { class: "eyebrow", "运行栈" }
                                        div { class: "badge-row shell-header__signals",
                                            span { class: "tag", "统一运行时" }
                                            span { class: "tag tag--cool", "Axum + Dioxus" }
                                        }
                                    }
                                }
                            }
                            div { class: "shell-header__aside",
                                nav { class: "route-nav route-nav--topbar",
                                    Link { class: "{dashboard_link_class}", to: Route::Dashboard {}, "订阅组" }
                                    if let Some((selected_username, selected_label)) = selected_user_for_nav.clone() {
                                        Link {
                                            class: "{user_link_class} route-link--workspace",
                                            to: Route::UserDetail { username: selected_username.clone() },
                                            title: "{selected_username}",
                                            "{selected_label}"
                                        }
                                    }
                                    Link { class: "{account_link_class}", to: Route::Account {}, "账户" }
                                }
                                article { class: "shell-spotlight",
                                    p { class: "eyebrow", "{spotlight_label}" }
                                    strong { "{spotlight_title}" }
                                    p { class: "muted", "{spotlight_summary}" }
                                    div { class: "shell-spotlight__meta",
                                        span { class: "tag", "{mode_label}" }
                                        if let Some(route) = spotlight_route {
                                            code { "{route}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                main { class: "{content_class}", {children} }
            }
        }
    }
}

fn route_link_class(active_mode: Option<&'static str>, link_mode: &'static str) -> &'static str {
    if active_mode == Some(link_mode) {
        "route-link route-link--active"
    } else {
        "route-link"
    }
}

fn topbar_workspace_label(username: &str) -> String {
    const MAX_LABEL_LEN: usize = 22;

    if username.len() <= MAX_LABEL_LEN {
        username.to_string()
    } else {
        format!("{}...", &username[..MAX_LABEL_LEN - 3])
    }
}

fn shell_spotlight_label(
    active_mode: Option<&'static str>,
    selected_user: Option<&str>,
) -> &'static str {
    if selected_user.is_some() {
        "当前订阅组"
    } else {
        match active_mode {
            Some("account") => "账户中心",
            _ => "当前页面",
        }
    }
}

fn shell_spotlight_summary(
    active_mode: Option<&'static str>,
    selected_user: Option<&str>,
    default_summary: &str,
) -> String {
    if let Some(username) = selected_user {
        format!("公开订阅地址 /{username}，这里集中处理源链接编辑、缓存快照和抓取诊断。")
    } else {
        match active_mode {
            Some("account") => "维护管理员用户名和密码，更新后当前会话会立即退出。".to_string(),
            _ => default_summary.to_string(),
        }
    }
}

fn shell_mode_label(active_mode: Option<&'static str>) -> &'static str {
    match active_mode {
        Some("account") => "账户维护",
        Some("user") => "订阅组详情",
        _ => "订阅组控制台",
    }
}

fn shell_mode_summary(active_mode: Option<&'static str>, has_selected_user: bool) -> &'static str {
    match active_mode {
        Some("account") => "当前页面聚焦管理员凭据和会话安全设置。",
        Some("user") if has_selected_user => "当前视图聚焦单个订阅组，方便连续编辑和排查。",
        _ => "先从左侧订阅组列表选中对象，再进入具体编辑和诊断流程。",
    }
}
