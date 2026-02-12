// Silo AI - GPUI 原生客户端

mod ui;

use gpui::{App, Application, Bounds, Context, CursorStyle, Entity, SharedString, Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size};
use silo_lib::{execute_agent_task, get_backend_type, get_vault_stats, AppState};
use std::sync::Arc;
use ui::{key_bindings, TextInput};

struct Message {
    id: String,
    role: Role,
    content: SharedString,
}

enum Role {
    User,
    Assistant,
}

struct Artifact {
    content: SharedString,
    mime_type: SharedString,
}

struct SiloApp {
    state: Option<Arc<AppState>>,
    messages: Vec<Message>,
    text_input: Entity<TextInput>,
    backend_type: SharedString,
    document_count: u64,
    artifacts: Vec<Artifact>,
    isLoading: bool,
    error: Option<SharedString>,
}

fn example_prompt_div(
    cx: &mut Context<SiloApp>,
    id: impl Into<gpui::ElementId>,
    text: impl Into<SharedString>,
    bg: gpui::Rgba,
    border: gpui::Rgba,
    text_color: gpui::Rgba,
    hover_border: gpui::Rgba,
) -> impl IntoElement {
    let text_shared = text.into();
    let text_for_listener = text_shared.clone();
    div()
        .id(id)
        .rounded_lg()
        .p_3()
        .border_1()
        .border_color(border)
        .bg(bg)
        .text_sm()
        .text_color(text_color)
        .cursor_pointer()
        .hover(move |s| s.bg(border).border_color(hover_border))
        .on_click(cx.listener(move |this, _, window, cx| {
            let input = text_for_listener.to_string();
            if input.trim().is_empty() || this.isLoading {
                return;
            }
            this.messages.push(Message {
                id: format!("{}", chrono::Utc::now().timestamp_millis()),
                role: Role::User,
                content: input.clone().into(),
            });
            this.isLoading = true;
            cx.notify();

            let state = this.state.clone();
            window.spawn(cx, async move |cx| {
                let result = cx
                    .background_executor()
                    .spawn(async move {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async move {
                            if let Some(ref s) = state {
                                execute_agent_task(s.as_ref(), input, None).await
                            } else {
                                Err("未初始化".into())
                            }
                        })
                    })
                    .await;

                let _ = cx.update_root(|root, _, cx| {
                    if let Ok(view) = root.downcast::<SiloApp>() {
                        view.update(cx, |app, cx| {
                            app.isLoading = false;
                            match result {
                                Ok(response) => {
                                    let reasoning = response
                                        .get("reasoning")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("任务完成")
                                        .to_string();
                                    app.messages.push(Message {
                                        id: format!("{}", chrono::Utc::now().timestamp_millis()),
                                        role: Role::Assistant,
                                        content: reasoning.into(),
                                    });
                                    app.artifacts = response
                                        .get("artifacts")
                                        .and_then(|v| v.as_array())
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|v| {
                                                    v.as_object().map(|o| Artifact {
                                                        content: o
                                                            .get("content")
                                                            .and_then(|c| c.as_str().map(String::from))
                                                            .unwrap_or_default()
                                                            .into(),
                                                        mime_type: o
                                                            .get("mime_type")
                                                            .and_then(|m| m.as_str().map(String::from))
                                                            .unwrap_or_default()
                                                            .into(),
                                                    })
                                                })
                                                .collect()
                                        })
                                        .unwrap_or_default();
                                }
                                Err(e) => {
                                    app.messages.push(Message {
                                        id: format!("{}", chrono::Utc::now().timestamp_millis()),
                                        role: Role::Assistant,
                                        content: format!(
                                            "❌ 错误: {}\n\n提示：当前运行在模拟模式下。要使用真实 AI 模型，请配置模型文件。",
                                            e
                                        )
                                        .into(),
                                    });
                                }
                            }
                            cx.notify();
                        });
                    }
                });
            })
            .detach();
        }))
        .child(text_shared)
}

impl SiloApp {
    /// 使用预初始化状态创建（主入口）
    fn new_with_state(
        _window: &mut Window,
        cx: &mut Context<Self>,
        state: Option<Arc<AppState>>,
        backend_type: String,
        document_count: u64,
        init_error: Option<String>,
    ) -> Self {
        let text_input = cx.new(|cx| TextInput::new(cx, "输入指令..."));
        Self {
            state,
            messages: vec![],
            text_input,
            backend_type: backend_type.into(),
            document_count,
            artifacts: vec![],
            isLoading: false,
            error: init_error.map(Into::into),
        }
    }

    #[allow(dead_code)]
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new_with_state(window, cx, None, "检测中...".to_string(), 0, None)
    }
}

impl Render for SiloApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let amber = rgb(0xfbbf24);
        let charcoal = rgb(0x1e1e1e);
        let gray_200 = rgb(0xe5e7eb);
        let gray_300 = rgb(0xd1d5db);
        let gray_400 = rgb(0x9ca3af);
        let gray_500 = rgb(0x6b7280);
        let gray_700 = rgb(0x374151);
        let gray_800 = rgb(0x1f2937);
        let gray_900 = rgb(0x111827);

        let empty_msg = self.messages.is_empty();
        let loading = self.isLoading;
        let has_error = self.error.is_some();
        let empty_artifacts = self.artifacts.is_empty();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(charcoal)
            .text_color(gray_200)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(gray_700)
                    .p_4()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_4()
                            .child(div().text_xl().text_color(amber).child("SILO"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(gray_400)
                                    .child(format!("Backend: {}", self.backend_type)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(gray_500)
                                    .child(format!("Vault: {} docs", self.document_count)),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(gray_500)
                            .child("Your Data's Fortress"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .border_r_1()
                            .border_color(gray_700)
                            .overflow_hidden()
                            .p_4()
                            .gap_4()
                            .child(
                                div().when(empty_msg, |d| {
                                    d.flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .gap_6()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .items_center()
                                                .gap_1()
                                                .child(
                                                    div()
                                                        .text_lg()
                                                        .text_color(gray_200)
                                                        .child("欢迎使用 Silo AI"),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .text_color(gray_500)
                                                        .child("隐私优先的本地 Agent 操作系统 · Your Data's Fortress"),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(gray_400)
                                                .child("试试这些示例指令（点击填充到输入框）："),
                                        )
                                        .child(example_prompt_div(
                                            cx,
                                            "example-1",
                                            "列出当前目录下的文件",
                                            gray_800,
                                            gray_700,
                                            gray_300,
                                            amber,
                                        ))
                                        .child(example_prompt_div(
                                            cx,
                                            "example-2",
                                            "扫描 ~/Downloads 找到上个月的发票 PDF",
                                            gray_800,
                                            gray_700,
                                            gray_300,
                                            amber,
                                        ))
                                        .child(example_prompt_div(
                                            cx,
                                            "example-3",
                                            "用一句话介绍你自己",
                                            gray_800,
                                            gray_700,
                                            gray_300,
                                            amber,
                                        ))
                                }),
                            )
                            .children(self.messages.iter().map(|msg| {
                                let is_user = matches!(msg.role, Role::User);
                                div()
                                    .flex()
                                    .when(is_user, |d| d.justify_end())
                                    .when(!is_user, |d| d.justify_start())
                                    .child(
                                        div()
                                            .rounded_lg()
                                            .p_3()
                                            .text_sm()
                                            .when(is_user, |d| {
                                                d.bg(gray_800)
                                                    .border_1()
                                                    .border_color(amber)
                                                    .text_color(amber)
                                            })
                                            .when(!is_user, |d| {
                                                d.bg(gray_800)
                                                    .border_1()
                                                    .border_color(gray_700)
                                                    .text_color(gray_200)
                                            })
                                            .child(msg.content.clone()),
                                    )
                            }))
                            .child(
                                div().when(loading, |d| {
                                    d.flex()
                                        .justify_start()
                                        .child(
                                            div()
                                                .bg(gray_800)
                                                .rounded_lg()
                                                .p_3()
                                                .border_1()
                                                .border_color(gray_700)
                                                .child("..."),
                                        )
                                }),
                            )
                            .child(
                                div().when(has_error, |d| {
                                    d.text_color(gpui::red())
                                        .child(self.error.clone().unwrap_or_default())
                                }),
                            ),
                    )
                    .child(
                        div()
                            .w(px(384.))
                            .flex()
                            .flex_col()
                            .border_l_1()
                            .border_color(gray_700)
                            .bg(gray_900)
                            .child(
                                div()
                                    .border_b_1()
                                    .border_color(gray_700)
                                    .p_4()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(amber)
                                            .child("LIVE ARTIFACTS"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .overflow_hidden()
                                    .p_4()
                                    .gap_4()
                                    .when(empty_artifacts, |d: gpui::Div| {
                                        d.flex()
                                            .items_center()
                                            .justify_center()
                                            .text_color(gray_500)
                                            .child("执行任务后，生成的代码、文档将在此预览")
                                    })
                                    .when(!empty_artifacts, |d: gpui::Div| {
                                        d.children(
                                            self.artifacts
                                                .iter()
                                                .map(|a| {
                                                    div()
                                                        .bg(gray_800)
                                                        .rounded_lg()
                                                        .p_3()
                                                        .border_1()
                                                        .border_color(gray_700)
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(gray_400)
                                                                .child(a.mime_type.clone()),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(gray_200)
                                                                .child(a.content.clone()),
                                                        )
                                                }),
                                        )
                                    }),
                            ),
                    ),
            )
            .child(
                div()
                    .border_t_1()
                    .border_color(gray_700)
                    .p_4()
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .flex_1()
                                    .bg(gray_900)
                                    .border_1()
                                    .border_color(gray_700)
                                    .rounded_sm()
                                    .px_4()
                                    .py_2()
                                    .text_sm()
                                    .text_color(gray_200)
                                    .id(gpui::ElementId::Name("input-area".into()))
                                    .cursor(CursorStyle::IBeam)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        let handle = this.text_input.read(cx).focus_handle.clone();
                                        window.focus(&handle);
                                    }))
                                    .child(self.text_input.clone()),
                            )
                            .child(
                                div()
                                    .id("execute")
                                    .px_6()
                                    .py_2()
                                    .bg(amber)
                                    .text_color(charcoal)
                                    .rounded_sm()
                                    .child("执行")
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        let input = this
                                            .text_input
                                            .update(cx, |ti, cx| {
                                                let s = ti.content().to_string();
                                                ti.clear(cx);
                                                s
                                            });
                                        if input.trim().is_empty() || this.isLoading {
                                            return;
                                        }
                                        this.messages.push(Message {
                                            id: format!(
                                                "{}",
                                                chrono::Utc::now().timestamp_millis()
                                            ),
                                            role: Role::User,
                                            content: input.clone().into(),
                                        });
                                        this.isLoading = true;
                                        cx.notify();

            let state = this.state.clone();
            window.spawn(cx, async move |cx| {
                let result = cx
                    .background_executor()
                    .spawn(async move {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async move {
                            if let Some(ref s) = state {
                                execute_agent_task(s.as_ref(), input, None).await
                            } else {
                                Err("未初始化".into())
                            }
                        })
                    })
                    .await;

                let _ = cx.update_root(|root, _, cx| {
                    if let Ok(view) = root.downcast::<SiloApp>() {
                        view.update(cx, |app, cx| {
                            app.isLoading = false;
                            match result {
                                                            Ok(response) => {
                                                                let reasoning = response
                                                                    .get("reasoning")
                                                                    .and_then(|v| v.as_str())
                                                                    .unwrap_or("任务完成")
                                                                    .to_string();
                                                                app.messages.push(Message {
                                                                    id: format!(
                                                                        "{}",
                                                                        chrono::Utc::now()
                                                                            .timestamp_millis()
                                                                    ),
                                                                    role: Role::Assistant,
                                                                    content: reasoning.into(),
                                                                });
                                                                app.artifacts = response
                                                                    .get("artifacts")
                                                                    .and_then(|v| v.as_array())
                                                                    .map(|arr| {
                                                                        arr.iter()
                                                                            .filter_map(|v| {
                                                                                v.as_object().map(
                                                                                    |o| {
                                                                                        Artifact {
                                                                                            content: o
                                                                                                .get("content")
                                                                                                .and_then(
                                                                                                    |c| {
                                                                                                        c.as_str()
                                                                                                            .map(String::from)
                                                                                                    },
                                                                                                )
                                                                                                .unwrap_or_default()
                                                                                                .into(),
                                                                                            mime_type: o
                                                                                                .get("mime_type")
                                                                                                .and_then(
                                                                                                    |m| {
                                                                                                        m.as_str()
                                                                                                            .map(String::from)
                                                                                                    },
                                                                                                )
                                                                                                .unwrap_or_default()
                                                                                                .into(),
                                                                                        }
                                                                                    },
                                                                                )
                                                                            })
                                                                            .collect()
                                                                    })
                                                                    .unwrap_or_default();
                                                            }
                                                            Err(e) => {
                                                                app.messages.push(Message {
                                                                    id: format!(
                                                                        "{}",
                                                                        chrono::Utc::now()
                                                                            .timestamp_millis()
                                                                    ),
                                                                    role: Role::Assistant,
                                                                    content: format!(
                                                                        "❌ 错误: {}\n\n提示：当前运行在模拟模式下。要使用真实 AI 模型，请配置模型文件。",
                                                                        e
                                                                    )
                                                                    .into(),
                                                                });
                                                            }
                                                        }
                                                        cx.notify();
                                                    });
                                                }
                                            });
                                    })
                                    .detach();
                                        })),
                            ),
                    ),
            )
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // 在打开窗口前同步完成初始化，避免 GPUI 异步 spawn 与 tokio 的兼容问题
    let rt = tokio::runtime::Runtime::new().unwrap();
    let init_result = rt.block_on(AppState::new());
    let (state, backend_type, document_count, init_error) = match init_result {
        Ok(s) => {
            let state = Arc::new(s);
            let backend = rt.block_on(get_backend_type(&state)).unwrap_or_else(|e| e);
            let stats = rt.block_on(get_vault_stats(&state)).unwrap_or_default();
            let doc_count = stats
                .get("document_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            (Some(state), backend, doc_count, None)
        }
        Err(e) => (
            None,
            "初始化失败".to_string(),
            0,
            Some(format!("初始化失败: {}", e)),
        ),
    };

    Application::new().run(move |cx: &mut App| {
        cx.bind_keys(key_bindings());
        let bounds = Bounds::centered(None, size(px(1400.), px(900.)), cx);
        let state_clone = state.clone();
        let backend_clone = backend_type.clone();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, cx| {
                cx.new(|cx| {
                    SiloApp::new_with_state(
                        window,
                        cx,
                        state_clone.clone(),
                        backend_clone.clone(),
                        document_count,
                        init_error.clone(),
                    )
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
