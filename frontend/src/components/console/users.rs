use dioxus::prelude::*;
use submora_shared::users::UserSummary;

use crate::app::Route;

use super::{
    actions,
    state::{FeedbackSignals, PendingState, RefreshState},
};

#[component]
pub fn UsersPanel(
    mut create_username: Signal<String>,
    users: Option<Vec<UserSummary>>,
    selected_username: Option<String>,
    pending: PendingState,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) -> Element {
    let navigator = use_navigator();
    let mut search_query = use_signal(String::new);
    let user_list = users.clone().unwrap_or_default();
    let user_count = user_list.len();
    let selected = selected_username.clone();
    let create_pending = (pending.create_user)();
    let search_value = search_query();
    let normalized_query = search_value.trim().to_ascii_lowercase();
    let sorting_enabled = normalized_query.is_empty();
    let visible_users = if sorting_enabled {
        user_list.clone()
    } else {
        user_list
            .iter()
            .filter(|user| {
                user.username
                    .to_ascii_lowercase()
                    .contains(&normalized_query)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let visible_count = visible_users.len();

    rsx! {
        article { class: "panel users-panel",
            div { class: "section-head",
                div {
                    p { class: "eyebrow", "订阅组" }
                    h2 { "订阅组列表" }
                    p { class: "muted", "每个订阅组都会对应一个公开订阅地址。" }
                }
                span { class: "tag", "{user_count} 个订阅组" }
            }
            div { class: "users-panel__composer",
                form {
                    class: "inline-form",
                    onsubmit: move |event| {
                        event.prevent_default();
                        actions::create_user_and_open(
                            create_username(),
                            create_username,
                            pending.create_user,
                            feedback,
                            refresh,
                            move |username| {
                                navigator.push(Route::UserDetail { username });
                            },
                        );
                    },
                    label { class: "field field--inline",
                        span { "新建订阅组" }
                        input {
                            disabled: create_pending,
                            value: "{create_username()}",
                            oninput: move |event| create_username.set(event.value()),
                            placeholder: "alpha-feed"
                        }
                    }
                    button {
                        class: "button button--primary",
                        r#type: "submit",
                        disabled: create_pending,
                        aria_busy: if create_pending { "true" } else { "false" },
                        if create_pending { "创建中…" } else { "新建" }
                    }
                }
            }
            if users.is_some() && !user_list.is_empty() {
                div { class: "users-panel__toolbar",
                    label { class: "field field--inline users-panel__search",
                        span { "筛选订阅组" }
                        input {
                            value: "{search_value}",
                            oninput: move |event| search_query.set(event.value()),
                            placeholder: "按用户名搜索"
                        }
                    }
                    div { class: "button-row users-panel__toolbar-actions",
                        if !sorting_enabled {
                            button {
                                class: "button button--ghost button--compact",
                                r#type: "button",
                                onclick: move |_| search_query.set(String::new()),
                                "清除筛选"
                            }
                        }
                        span { class: "tag", "显示 {visible_count} / {user_count}" }
                    }
                    if !sorting_enabled {
                        p { class: "muted users-panel__search-note",
                            "筛选模式下仅保留管理和预览动作；清除筛选后可继续调整完整顺序。"
                        }
                    }
                }
            }
            if users.is_some() {
                if user_list.is_empty() {
                    div { class: "empty-state",
                        strong { "还没有订阅组" }
                        p { "先新建一个订阅组，再开始维护它的源链接和公开订阅输出。" }
                    }
                } else if visible_users.is_empty() {
                    div { class: "empty-state",
                        strong { "没有匹配的订阅组" }
                        p { "试试其他关键词，或者清除当前筛选后查看完整订阅组列表。" }
                    }
                } else {
                    div { class: "user-list",
                        for (index, user) in visible_users.clone().into_iter().enumerate() {
                            UserRow {
                                key: "{user.username}",
                                index,
                                user,
                                users: user_list.clone(),
                                selected: selected.clone(),
                                sorting_enabled,
                                pending,
                                feedback,
                                refresh,
                            }
                        }
                    }
                }
            } else {
                p { class: "muted", "正在加载订阅组..." }
            }
        }
    }
}

#[component]
fn UserRow(
    index: usize,
    user: UserSummary,
    users: Vec<UserSummary>,
    selected: Option<String>,
    sorting_enabled: bool,
    pending: PendingState,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) -> Element {
    let navigator = use_navigator();
    let is_selected = selected.as_deref() == Some(user.username.as_str());
    let username = user.username.clone();
    let username_for_select = username.clone();
    let username_for_up = username.clone();
    let username_for_down = username.clone();
    let username_for_top = username.clone();
    let username_for_bottom = username.clone();
    let public_route = format!("/{username}");
    let order_source_for_up = users.clone();
    let order_source_for_down = users.clone();
    let order_source_for_top = users.clone();
    let order_source_for_bottom = users.clone();
    let can_move_up = index > 0;
    let can_move_down = index + 1 < users.len();
    let can_move_to_top = can_move_up;
    let can_move_to_bottom = can_move_down;
    let card_class = if is_selected {
        "user-card user-card--selected"
    } else {
        "user-card"
    };
    let edit_button_class = if is_selected {
        "button button--primary button--compact"
    } else {
        "button button--ghost button--compact"
    };
    let order_label = format!("#{}", index + 1);
    let reorder_pending = (pending.reorder_users)();

    rsx! {
        article { class: "{card_class}",
            div { class: "user-card__meta",
                div { class: "user-card-head",
                    div {
                        strong { "{user.username}" }
                        p { class: "muted", "公开订阅地址 " code { "/{user.username}" } }
                    }
                    div { class: "badge-row",
                        if is_selected {
                            span { class: "tag tag--accent", "当前订阅组" }
                        }
                        span { class: "tag", "{order_label}" }
                    }
                }
            }
            div { class: "user-card__actions",
                div { class: "button-row user-card__primary-actions",
                    button {
                        class: "{edit_button_class}",
                        r#type: "button",
                        onclick: move |_| {
                            navigator.push(Route::UserDetail {
                                username: username_for_select.clone(),
                            });
                        },
                        "管理"
                    }
                    a {
                        class: "button button--ghost button--compact",
                        href: "{public_route}",
                        target: "_blank",
                        rel: "noreferrer",
                        "预览"
                    }
                }
                if sorting_enabled {
                    div { class: "button-row user-card__reorder",
                        button {
                            class: "button button--ghost button--compact",
                            r#type: "button",
                            disabled: reorder_pending || !can_move_to_top,
                            aria_busy: if reorder_pending { "true" } else { "false" },
                            onclick: move |_| {
                                if let Some(order) =
                                    actions::move_username_to_edge(&order_source_for_top, &username_for_top, true)
                                {
                                    actions::submit_user_order(
                                        order,
                                        pending.reorder_users,
                                        feedback,
                                        refresh,
                                    );
                                }
                            },
                            if reorder_pending { "排序中…" } else { "置顶" }
                        }
                        button {
                            class: "button button--ghost button--compact",
                            r#type: "button",
                            disabled: reorder_pending || !can_move_up,
                            aria_busy: if reorder_pending { "true" } else { "false" },
                            onclick: move |_| {
                                if let Some(order) =
                                    actions::reordered_usernames(&order_source_for_up, &username_for_up, -1)
                                {
                                    actions::submit_user_order(
                                        order,
                                        pending.reorder_users,
                                        feedback,
                                        refresh,
                                    );
                                }
                            },
                            if reorder_pending { "排序中…" } else { "上移" }
                        }
                        button {
                            class: "button button--ghost button--compact",
                            r#type: "button",
                            disabled: reorder_pending || !can_move_down,
                            aria_busy: if reorder_pending { "true" } else { "false" },
                            onclick: move |_| {
                                if let Some(order) =
                                    actions::reordered_usernames(&order_source_for_down, &username_for_down, 1)
                                {
                                    actions::submit_user_order(
                                        order,
                                        pending.reorder_users,
                                        feedback,
                                        refresh,
                                    );
                                }
                            },
                            if reorder_pending { "排序中…" } else { "下移" }
                        }
                        button {
                            class: "button button--ghost button--compact",
                            r#type: "button",
                            disabled: reorder_pending || !can_move_to_bottom,
                            aria_busy: if reorder_pending { "true" } else { "false" },
                            onclick: move |_| {
                                if let Some(order) =
                                    actions::move_username_to_edge(&order_source_for_bottom, &username_for_bottom, false)
                                {
                                    actions::submit_user_order(
                                        order,
                                        pending.reorder_users,
                                        feedback,
                                        refresh,
                                    );
                                }
                            },
                            if reorder_pending { "排序中…" } else { "置底" }
                        }
                    }
                }
            }
        }
    }
}
