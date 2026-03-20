use dioxus::prelude::*;

use crate::app::Route;

use super::{
    actions, services,
    state::{FeedbackSignals, LinkDraftState, PendingState, RefreshState, remember_links_input},
};

#[component]
pub fn EditorPanel(
    username: String,
    selected_route: String,
    mut links_text: Signal<String>,
    selected_link_count: usize,
    drafts: LinkDraftState,
    has_unsaved_changes: bool,
    pending: PendingState,
    feedback: FeedbackSignals,
    refresh: RefreshState,
) -> Element {
    let navigator = use_navigator();
    let mut links_error = use_signal(|| None::<String>);
    let mut confirm_delete = use_signal(|| false);
    let username_for_input = username.clone();
    let username_for_save = username.clone();
    let username_for_delete = username.clone();
    let save_pending = (pending.save_links)();
    let delete_pending = (pending.delete_user)();
    let editor_busy = save_pending || delete_pending;
    let current_links_text = links_text();
    let draft_stats = services::analyze_links(&current_links_text, 4);
    let preview_overflow = draft_stats
        .normalized_count
        .saturating_sub(draft_stats.normalized_preview.len());
    let local_format_issue = draft_stats.first_invalid.as_ref().map(|invalid| {
        format!(
            "发现 {} 条格式不正确的链接，保存前需要修正。首条问题：{invalid}",
            draft_stats.invalid_count
        )
    });
    let editor_status_title = if draft_stats.invalid_count > 0 {
        "发现格式问题"
    } else if has_unsaved_changes {
        "草稿待保存"
    } else {
        "当前内容已同步"
    };
    let editor_status_detail = if draft_stats.invalid_count > 0 {
        "先修正格式不正确的链接，再执行保存。".to_string()
    } else if has_unsaved_changes {
        draft_status_note(&draft_stats)
    } else {
        "当前编辑器内容已经与已保存版本一致。".to_string()
    };
    let save_button_label = if save_pending {
        "保存中…"
    } else if has_unsaved_changes
        && (draft_stats.blank_count > 0 || draft_stats.duplicate_count > 0)
    {
        "保存并归一化"
    } else {
        "保存源链接"
    };
    let delete_label = if delete_pending {
        "删除中…"
    } else if confirm_delete() {
        "请在下方确认"
    } else {
        "删除订阅组"
    };
    let editor_class = if links_error().is_some() {
        "source-editor source-editor--error"
    } else {
        "source-editor"
    };

    {
        let username = username.clone();
        use_effect(move || {
            let _ = &username;
            links_error.set(None);
            confirm_delete.set(false);
        });
    }

    rsx! {
        article { id: "editor-panel", class: "panel panel--editor editor-panel",
            div { class: "editor-panel__header",
                div { class: "editor-panel__lead",
                    p { class: "eyebrow", "主编辑区" }
                    h2 { "编辑订阅组源链接" }
                    p { class: "panel-copy", "订阅组 {username} · 公开订阅地址 {selected_route}" }
                }
                div { class: "badge-row",
                    span { class: "tag tag--accent", "{selected_link_count} 条" }
                    if has_unsaved_changes {
                        span { class: "tag tag--danger", "未保存草稿" }
                    }
                }
            }
            div { class: "metric-grid editor-metric-grid",
                article { class: "metric-card metric-card--editor",
                    span { class: "stat-kicker", "输入行" }
                    strong { class: "metric-value", "{draft_stats.raw_line_count}" }
                }
                article { class: "metric-card metric-card--editor",
                    span { class: "stat-kicker", "非空" }
                    strong { class: "metric-value", "{draft_stats.non_empty_count}" }
                }
                article { class: "metric-card metric-card--editor",
                    span { class: "stat-kicker", "保存后" }
                    strong { class: "metric-value", "{draft_stats.normalized_count}" }
                }
                article { class: "metric-card metric-card--editor",
                    span { class: "stat-kicker", "格式异常" }
                    strong { class: "metric-value", "{draft_stats.invalid_count}" }
                }
            }
            textarea {
                class: "{editor_class}",
                disabled: editor_busy,
                value: "{current_links_text}",
                oninput: move |event| {
                    let value = event.value();
                    links_error.set(None);
                    links_text.set(value.clone());
                    remember_links_input(&username_for_input, &value, drafts);
                },
                rows: "16",
                placeholder: "https://example.com/feed\nhttps://news.example.org/article",
                aria_invalid: if links_error().is_some() { "true" } else { "false" }
            }
            if let Some(message) = links_error() {
                p { class: "field-error", "{message}" }
            } else if let Some(message) = local_format_issue {
                p { class: "field-error", "{message}" }
            } else if has_unsaved_changes {
                p { class: "field-hint", "当前有未保存草稿，切换订阅组后会在当前浏览器会话中保留。" }
            } else {
                p { class: "field-hint", "仅支持 HTTP/HTTPS，私网地址和不可解析目标会被拒绝。" }
            }
            div { class: "editor-preview",
                div { class: "editor-preview__head",
                    div {
                        p { class: "eyebrow", "保存后预览" }
                        h3 { "即将保留的链接顺序" }
                    }
                    span { class: "tag", "{draft_stats.normalized_count} 条" }
                }
                if draft_stats.normalized_preview.is_empty() {
                    p { class: "muted",
                        if draft_stats.invalid_count > 0 {
                            "当前草稿里还没有可保存的有效链接。"
                        } else {
                            "当前草稿为空，保存后会清空这个订阅组的源链接。"
                        }
                    }
                } else {
                    div { class: "editor-preview__list",
                        for (index, link) in draft_stats.normalized_preview.iter().enumerate() {
                            div { class: "editor-preview__item",
                                span { class: "editor-preview__order", "{index + 1}" }
                                code { "{link}" }
                            }
                        }
                    }
                    if preview_overflow > 0 {
                        p { class: "muted", "还有 {preview_overflow} 条链接未展开，保存后会继续按当前顺序保留。" }
                    }
                }
            }
            div { class: "editor-panel__actions",
                div { class: "editor-action-bar",
                    div { class: "editor-action-bar__status",
                        p { class: "eyebrow", "保存动作" }
                        strong { "{editor_status_title}" }
                        p { class: "muted", "{editor_status_detail}" }
                    }
                    div { class: "button-row editor-panel__primary-actions",
                        button {
                            class: "button button--primary",
                            onclick: move |_| {
                                actions::save_links(
                                    username_for_save.clone(),
                                    links_text(),
                                    links_text,
                                    links_error,
                                    drafts,
                                    pending.save_links,
                                    feedback,
                                    refresh,
                                );
                            },
                            disabled: editor_busy || !has_unsaved_changes || draft_stats.invalid_count > 0,
                            aria_busy: if save_pending { "true" } else { "false" },
                            "{save_button_label}"
                        }
                        a {
                            class: "button button--ghost",
                            href: "{selected_route}",
                            target: "_blank",
                            rel: "noreferrer",
                            "公开预览"
                        }
                        button {
                            class: "button button--ghost",
                            onclick: move |_| refresh.bump_diagnostics(),
                            "刷新诊断"
                        }
                    }
                }
                button {
                    class: "button button--danger",
                    disabled: editor_busy,
                    aria_busy: if delete_pending { "true" } else { "false" },
                    onclick: move |_| confirm_delete.set(true),
                    "{delete_label}"
                }
                if confirm_delete() {
                    article {
                        class: "confirm-strip confirm-strip--danger",
                        role: "group",
                        "aria-label": "删除订阅组确认",
                        div { class: "confirm-strip__copy",
                            p { class: "eyebrow", "危险操作" }
                            strong { "确认删除 {username}" }
                            p { class: "muted", "这会同时移除该订阅组的链接配置、缓存快照和抓取诊断，且当前页面会返回订阅组列表。" }
                        }
                        div { class: "button-row confirm-strip__actions",
                            button {
                                class: "button button--ghost",
                                disabled: editor_busy,
                                onclick: move |_| confirm_delete.set(false),
                                "取消"
                            }
                            button {
                                class: "button button--danger",
                                disabled: editor_busy,
                                aria_busy: if delete_pending { "true" } else { "false" },
                                onclick: move |_| {
                                    confirm_delete.set(false);
                                    actions::delete_user_and_leave(
                                        username_for_delete.clone(),
                                        links_text,
                                        drafts,
                                        pending.delete_user,
                                        feedback,
                                        refresh,
                                        move || {
                                            navigator.replace(Route::Dashboard {});
                                        },
                                    );
                                },
                                if delete_pending { "删除中…" } else { "确认删除订阅组" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn draft_status_note(stats: &services::DraftLinkStats) -> String {
    if stats.normalized_count == 0 {
        return "保存后会清空这个订阅组的源链接。".to_string();
    }

    let mut notes = Vec::new();
    if stats.duplicate_count > 0 {
        notes.push(format!("合并 {} 条重复链接", stats.duplicate_count));
    }
    if stats.blank_count > 0 {
        notes.push(format!("忽略 {} 行空白", stats.blank_count));
    }

    if notes.is_empty() {
        format!(
            "当前草稿将按填写顺序保存，共 {} 条。",
            stats.normalized_count
        )
    } else {
        format!(
            "保存后保留 {} 条链接，并会自动{}。",
            stats.normalized_count,
            notes.join("，")
        )
    }
}
