use dioxus::prelude::*;

use crate::app::Route;

use super::{
    actions,
    state::{FeedbackSignals, PendingState, RefreshState},
};

#[component]
pub fn ControlPlanePanel(
    username: String,
    selected_username: Option<String>,
    show_selection: bool,
    mut links_text: Signal<String>,
    pending: PendingState,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) -> Element {
    let selected = selected_username.clone();
    let public_route = selected_username.map(|value| format!("/{value}"));
    let logout_pending = (pending.logout)();
    let selection_state = if selected.is_some() {
        "已锁定订阅组"
    } else {
        "等待选择"
    };
    let selection_summary = if let Some(selected_username) = selected.clone() {
        format!("当前选中 {selected_username}，公开订阅地址 /{selected_username}")
    } else {
        "从左侧列表选择订阅组后，再继续编辑和诊断。".to_string()
    };

    rsx! {
        article { class: "panel panel--accent session-panel",
            div { class: "section-head session-panel__header",
                div {
                    p { class: "eyebrow", "管理员" }
                    h2 { "{username}" }
                    p { class: "muted", "使用当前账号管理订阅组与源链接。" }
                }
            }
            if show_selection {
                div { class: "session-panel__selection",
                    p { class: "eyebrow", "当前订阅组" }
                    if let Some(selected_username_value) = selected.clone() {
                        strong { "{selected_username_value}" }
                        p { class: "muted", "公开订阅地址 /{selected_username_value}" }
                    } else {
                        strong { "尚未选择订阅组" }
                        p { class: "muted", "从左侧列表选择，或先新建一个订阅组。" }
                    }
                }
            }
            div { class: "session-panel__facts",
                article { class: "session-fact" ,
                    span { class: "stat-kicker", "登录身份" }
                    strong { "管理员" }
                    p { class: "muted", "当前会话正在使用账号 {username}。" }
                }
                article { class: "session-fact",
                    span { class: "stat-kicker", "控制焦点" }
                    strong { "{selection_state}" }
                    p { class: "muted", "{selection_summary}" }
                }
            }
            div { class: "button-row session-panel__nav",
                Link { class: "button button--ghost", to: Route::Account {}, "账户设置" }
                if let Some(public_route) = public_route {
                    a {
                        class: "button button--ghost",
                        href: "{public_route}",
                        target: "_blank",
                        rel: "noreferrer",
                        "公开预览"
                    }
                }
                button {
                    class: "button button--danger",
                    disabled: logout_pending,
                    aria_busy: if logout_pending { "true" } else { "false" },
                    onclick: move |_| actions::logout_session(links_text, pending.logout, feedback, refresh),
                    if logout_pending { "退出中…" } else { "退出登录" }
                }
            }
        }
    }
}

#[component]
pub fn LoginPanel(
    mut login_username: Signal<String>,
    mut login_password: Signal<String>,
    pending: PendingState,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) -> Element {
    let login_pending = (pending.login)();

    rsx! {
        article { class: "panel panel--hero auth-panel auth-panel--compact",
            div { class: "auth-panel__intro",
                p { class: "eyebrow", "{submora_core::APP_NAME}" }
                h1 { class: "auth-panel__title", "登录管理台" }
                p { class: "muted", "使用管理员账号管理订阅组、源链接与聚合状态。" }
                div { class: "badge-row auth-panel__signals",
                    span { class: "tag tag--accent", "Cookie 会话" }
                    span { class: "tag tag--cool", "CSRF 写保护" }
                }
            }
            form {
                class: "form-stack auth-form",
                onsubmit: move |event| {
                    event.prevent_default();
                    actions::submit_login(
                        login_username(),
                        login_password(),
                        login_password,
                        pending.login,
                        feedback,
                        refresh,
                    );
                },
                label { class: "field",
                    span { "用户名" }
                    input {
                        autocomplete: "username",
                        disabled: login_pending,
                        value: "{login_username()}",
                        oninput: move |event| login_username.set(event.value()),
                        placeholder: "admin"
                    }
                }
                label { class: "field",
                    span { "密码" }
                    input {
                        autocomplete: "current-password",
                        r#type: "password",
                        disabled: login_pending,
                        value: "{login_password()}",
                        oninput: move |event| login_password.set(event.value()),
                        placeholder: "••••••••"
                    }
                }
                button {
                    class: "button button--primary button--wide",
                    r#type: "submit",
                    disabled: login_pending,
                    aria_busy: if login_pending { "true" } else { "false" },
                    if login_pending { "登录中…" } else { "登录" }
                }
            }
        }
    }
}

#[component]
pub fn AccountPanel(
    mut account_username: Signal<String>,
    mut account_current_password: Signal<String>,
    mut account_new_password: Signal<String>,
    current_username: String,
    pending: PendingState,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) -> Element {
    let current_username_for_submit = current_username.clone();
    let current_username_placeholder = current_username.clone();
    let account_update_pending = (pending.account_update)();

    rsx! {
        article { class: "panel account-panel",
            div { class: "section-head",
                div {
                    p { class: "eyebrow", "账户设置" }
                    h2 { "管理员账户" }
                    p { class: "muted", "修改后当前会话会立即退出，请使用新凭据重新登录。" }
                }
                span { class: "tag", "{current_username}" }
            }
            form {
                class: "form-stack",
                onsubmit: move |event| {
                    event.prevent_default();
                    actions::submit_account_update(
                        current_username_for_submit.clone(),
                        account_username(),
                        account_current_password(),
                        account_new_password(),
                        account_username,
                        account_current_password,
                        account_new_password,
                        pending.account_update,
                        feedback,
                        refresh,
                    );
                },
                div { class: "field-grid",
                    label { class: "field",
                        span { "新用户名" }
                        input {
                            disabled: account_update_pending,
                            value: "{account_username()}",
                            oninput: move |event| account_username.set(event.value()),
                            placeholder: current_username_placeholder.clone()
                        }
                    }
                    label { class: "field",
                        span { "当前密码" }
                        input {
                            r#type: "password",
                            disabled: account_update_pending,
                            value: "{account_current_password()}",
                            oninput: move |event| account_current_password.set(event.value()),
                            placeholder: "必填"
                        }
                    }
                }
                label { class: "field",
                    span { "新密码" }
                    input {
                        r#type: "password",
                        disabled: account_update_pending,
                        value: "{account_new_password()}",
                        oninput: move |event| account_new_password.set(event.value()),
                        placeholder: "字母 + 数字 + 符号"
                    }
                }
                button {
                    class: "button button--primary",
                    r#type: "submit",
                    disabled: account_update_pending,
                    aria_busy: if account_update_pending { "true" } else { "false" },
                    if account_update_pending { "更新中…" } else { "更新账户" }
                }
            }
        }
    }
}
