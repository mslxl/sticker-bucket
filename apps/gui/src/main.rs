mod input;
mod settings;

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use gpui::{
    AnyElement, App, Application, Bounds, BoxShadow, Context, Div, ElementId, Entity, FontWeight,
    ObjectFit, PathPromptOptions, SharedString, Stateful, Window, WindowBackgroundAppearance,
    WindowBounds, WindowControlArea, WindowOptions, div, hsla, img, linear_color_stop,
    linear_gradient, point, prelude::*, px, rgb, rgba, size,
};
use input::TextInput;
use memelith_clip::{BuiltinModel, ClipModel, ExecutionPolicy};
use memelith_core::{
    APPLICATION_NAME, Meme, MemeContent, MemeDatabase, NewMeme, NewMemeContent, NewMemePack, NewTag,
};
use thiserror::Error;
use uuid::Uuid;

const INBOX_NAME: &str = "Inbox";

// Apple 系统色板（浅色外观）
const ACCENT: u32 = 0x007aff; // systemBlue
const ACCENT_HOVER: u32 = 0x0070e8;
const ACCENT_PRESS: u32 = 0x0063cc;
const SUCCESS: u32 = 0x34c759; // systemGreen
const DANGER: u32 = 0xff3b30; // systemRed
const INK: u32 = 0x1d1d1f; // label
const LABEL_2: u32 = 0x3c3c4399; // secondaryLabel
const LABEL_3: u32 = 0x3c3c4366; // tertiaryLabel
const SEP: u32 = 0x3c3c431f; // 分组内的发丝分隔线

// 液态玻璃材质：根层完全透出窗口模糊底；侧边栏是低不透明度的活性玻璃，
// 内容面板近白高亮，卡片几乎不透明——亮度层级避免白色互叠成灰
const SIDEBAR_GLASS: u32 = 0xffffff8c;
const CONTENT_BG: u32 = 0xfbfafde8;
const GLASS_CARD: u32 = 0xfffffff2; // 分组 / 卡片
const GLASS_STRONG: u32 = 0xffffffff; // 选中 / hover 提亮
const EDGE_DARK: u32 = 0x00000010; // 卡片与控件的定义边

fn card_shadow() -> Vec<BoxShadow> {
    vec![
        BoxShadow {
            color: hsla(0., 0., 0., 0.06),
            offset: point(px(0.), px(2.)),
            blur_radius: px(8.),
            spread_radius: px(-1.),
        },
        BoxShadow {
            color: hsla(0., 0., 0., 0.04),
            offset: point(px(0.), px(0.5)),
            blur_radius: px(1.),
            spread_radius: px(0.),
        },
    ]
}

fn control_shadow() -> Vec<BoxShadow> {
    vec![BoxShadow {
        color: hsla(0., 0., 0., 0.10),
        offset: point(px(0.), px(1.)),
        blur_radius: px(2.5),
        spread_radius: px(0.),
    }]
}

fn accent_fill() -> gpui::Background {
    linear_gradient(
        180.,
        linear_color_stop(rgb(0x2b8fff), 0.),
        linear_color_stop(rgb(ACCENT), 1.),
    )
}

/// 玻璃卡片：亮白浮层 + 极浅定义边 + 轻投影
fn glass_card() -> Div {
    div()
        .rounded(px(14.))
        .bg(rgba(GLASS_CARD))
        .border_1()
        .border_color(rgba(EDGE_DARK))
        .shadow(card_shadow())
}

/// 分组列表容器，对应 macOS 设置里的 inset group
fn glass_group() -> Div {
    glass_card().rounded(px(12.)).overflow_hidden()
}

fn group_row() -> Div {
    div()
        .min_h(px(44.))
        .px_4()
        .py_3()
        .flex()
        .items_center()
        .gap_4()
}

fn hairline() -> Div {
    div().h(px(1.)).ml_4().bg(rgba(SEP))
}

fn section_header(label: impl Into<SharedString>) -> Div {
    div()
        .px_2()
        .pb_2()
        .text_size(px(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgba(LABEL_2))
        .child(label.into())
}

/// 主操作按钮：macOS 强调色胶囊（小尺寸、平光、细阴影）
fn primary_pill(id: impl Into<ElementId>) -> Stateful<Div> {
    div()
        .id(id)
        .h(px(28.))
        .px_4()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(accent_fill())
        .text_color(rgb(0xffffff))
        .text_size(px(13.))
        .font_weight(FontWeight::SEMIBOLD)
        .cursor_pointer()
        .shadow(control_shadow())
        .hover(|style| style.bg(rgb(ACCENT_HOVER)))
        .active(|style| style.bg(rgb(ACCENT_PRESS)))
}

/// 次级按钮：macOS bordered 白胶囊
fn glass_pill(id: impl Into<ElementId>) -> Stateful<Div> {
    div()
        .id(id)
        .h(px(28.))
        .px_4()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(rgba(0xffffffd9))
        .border_1()
        .border_color(rgba(EDGE_DARK))
        .text_color(rgb(INK))
        .text_size(px(13.))
        .font_weight(FontWeight::MEDIUM)
        .cursor_pointer()
        .shadow(control_shadow())
        .hover(|style| style.bg(rgb(0xffffff)))
        .active(|style| style.bg(rgba(0xeceaf0f5)))
}

/// 无边框圆形图标按钮（如移除），hover 才浮现底色
fn icon_button(id: impl Into<ElementId>) -> Stateful<Div> {
    div()
        .id(id)
        .size(px(22.))
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .text_color(rgba(LABEL_2))
        .text_xs()
        .cursor_pointer()
        .hover(|style| style.bg(rgba(0x3c3c4314)).text_color(rgb(INK)))
        .active(|style| style.bg(rgba(0x3c3c4326)))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Page {
    Add,
    All,
    Settings,
}

impl Page {
    const fn label(self) -> &'static str {
        match self {
            Self::Add => "添加",
            Self::All => "全部",
            Self::Settings => "设置",
        }
    }

    const fn icon(self) -> &'static str {
        match self {
            Self::Add => "＋",
            Self::All => "▦",
            Self::Settings => "⚙\u{fe0e}",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Add => 0,
            Self::All => 1,
            Self::Settings => 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DraftContent {
    Image(PathBuf),
    Text(String),
}

#[derive(Clone, Debug)]
enum Notice {
    Success(String),
    Error(String),
}

#[derive(Debug, Error)]
enum UiError {
    #[error("无法加载 embedding 模型：{0}")]
    Clip(#[from] memelith_clip::Error),

    #[error("无法访问 Meme 数据库：{0}")]
    Database(#[from] memelith_core::Error),
}

struct OpenedLibrary {
    database: MemeDatabase,
    inbox_id: Uuid,
    memes: Vec<Meme>,
    pack_names: HashMap<Uuid, String>,
}

struct MemelithView {
    database: Option<MemeDatabase>,
    storage_root: Option<PathBuf>,
    inbox_id: Option<Uuid>,
    page: Page,
    memes: Vec<Meme>,
    pack_names: HashMap<Uuid, String>,
    draft_contents: Vec<DraftContent>,
    name_input: Entity<TextInput>,
    description_input: Entity<TextInput>,
    tags_input: Entity<TextInput>,
    text_content_input: Entity<TextInput>,
    notice: Option<Notice>,
    opening_storage: bool,
}

impl MemelithView {
    fn new(
        saved_storage: Result<Option<PathBuf>, settings::SettingsError>,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut view = Self {
            database: None,
            storage_root: None,
            inbox_id: None,
            page: Page::All,
            memes: Vec::new(),
            pack_names: HashMap::new(),
            draft_contents: Vec::new(),
            name_input: cx.new(|cx| TextInput::new("可选，例如：震惊", cx)),
            description_input: cx.new(|cx| TextInput::new("可选，补充使用场景", cx)),
            tags_input: cx.new(|cx| TextInput::new("用逗号分隔，例如：猫猫, 反应", cx)),
            text_content_input: cx.new(|cx| TextInput::new("输入一段 Meme 文字", cx)),
            notice: None,
            opening_storage: false,
        };

        match saved_storage {
            Ok(Some(path)) => view.activate_storage(path, false, cx),
            Ok(None) => {}
            Err(error) => {
                view.notice = Some(Notice::Error(format!("无法读取上次的存储位置：{error}")));
            }
        }
        view
    }

    fn activate_storage(&mut self, path: PathBuf, persist: bool, cx: &mut Context<Self>) {
        self.opening_storage = true;
        self.notice = None;
        cx.notify();

        match open_library(&path) {
            Ok(opened) => {
                let canonical_root = opened.database.storage_root().to_path_buf();
                self.database = Some(opened.database);
                self.storage_root = Some(canonical_root.clone());
                self.inbox_id = Some(opened.inbox_id);
                self.memes = opened.memes;
                self.pack_names = opened.pack_names;
                self.page = Page::All;
                if persist {
                    self.notice = match settings::save_storage_root(&canonical_root) {
                        Ok(()) => Some(Notice::Success("存储位置已更新".to_owned())),
                        Err(error) => Some(Notice::Error(format!(
                            "存储已打开，但无法记住该位置：{error}"
                        ))),
                    };
                }
            }
            Err(error) => {
                self.notice = Some(Notice::Error(error.to_string()));
            }
        }
        self.opening_storage = false;
        cx.notify();
    }

    fn choose_storage(&mut self, _: &gpui::ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("选择存储位置".into()),
        });
        cx.spawn(async move |this, cx| match receiver.await {
            Ok(Ok(Some(paths))) => {
                let path = paths.into_iter().next();
                if let Some(path) = path {
                    let _ = this.update(cx, |view, cx| view.activate_storage(path, true, cx));
                }
            }
            Ok(Ok(None)) => {}
            Ok(Err(error)) => {
                let _ = this.update(cx, |view, cx| {
                    view.notice = Some(Notice::Error(format!("无法打开目录选择器：{error}")));
                    cx.notify();
                });
            }
            Err(error) => {
                let _ = this.update(cx, |view, cx| {
                    view.notice = Some(Notice::Error(format!("目录选择已中断：{error}")));
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn choose_images(&mut self, _: &gpui::ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("添加图片".into()),
        });
        cx.spawn(async move |this, cx| match receiver.await {
            Ok(Ok(Some(paths))) => {
                let _ = this.update(cx, |view, cx| {
                    view.draft_contents
                        .extend(paths.into_iter().map(DraftContent::Image));
                    view.notice = None;
                    cx.notify();
                });
            }
            Ok(Ok(None)) => {}
            Ok(Err(error)) => {
                let _ = this.update(cx, |view, cx| {
                    view.notice = Some(Notice::Error(format!("无法打开图片选择器：{error}")));
                    cx.notify();
                });
            }
            Err(error) => {
                let _ = this.update(cx, |view, cx| {
                    view.notice = Some(Notice::Error(format!("图片选择已中断：{error}")));
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn add_text_content(&mut self, _: &gpui::ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        let text = self.text_content_input.read(cx).text();
        if text.trim().is_empty() {
            self.notice = Some(Notice::Error("请先输入文字内容".to_owned()));
            cx.notify();
            return;
        }
        self.draft_contents
            .push(DraftContent::Text(text.trim().to_owned()));
        self.text_content_input
            .update(cx, |input, cx| input.reset(cx));
        self.notice = None;
        cx.notify();
    }

    fn remove_draft_content(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.draft_contents.len() {
            self.notice = Some(Notice::Error("要移除的内容已经不存在".to_owned()));
        } else {
            self.draft_contents.remove(index);
            self.notice = None;
        }
        cx.notify();
    }

    fn save_meme(&mut self, _: &gpui::ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.draft_contents.is_empty() {
            self.notice = Some(Notice::Error("请至少添加一张图片或一段文字".to_owned()));
            cx.notify();
            return;
        }
        let Some(inbox_id) = self.inbox_id else {
            self.notice = Some(Notice::Error("Inbox 尚未准备好".to_owned()));
            cx.notify();
            return;
        };
        let name = optional_input_text(&self.name_input.read(cx).text());
        let description = optional_input_text(&self.description_input.read(cx).text());
        let tags = parse_tags(&self.tags_input.read(cx).text());
        let contents = self
            .draft_contents
            .iter()
            .map(|content| match content {
                DraftContent::Image(path) => NewMemeContent::Image {
                    source_path: path.clone(),
                },
                DraftContent::Text(text) => NewMemeContent::Text { text: text.clone() },
            })
            .collect();

        let Some(database) = self.database.as_mut() else {
            self.notice = Some(Notice::Error("数据库尚未打开".to_owned()));
            cx.notify();
            return;
        };
        let meme = match database.create_meme(
            inbox_id,
            NewMeme {
                name,
                description,
                contents,
            },
        ) {
            Ok(meme) => meme,
            Err(error) => {
                self.notice = Some(Notice::Error(format!("保存失败：{error}")));
                cx.notify();
                return;
            }
        };

        let mut tag_error = None;
        for tag_name in tags {
            let tag = match database.list_tags().and_then(|known| {
                if let Some(tag) = known
                    .into_iter()
                    .find(|tag| tag.name.eq_ignore_ascii_case(&tag_name))
                {
                    Ok(tag)
                } else {
                    database.create_tag(NewTag { name: tag_name })
                }
            }) {
                Ok(tag) => tag,
                Err(error) => {
                    tag_error = Some(error.to_string());
                    break;
                }
            };
            if let Err(error) = database.attach_tag_to_meme(meme.id, tag.id) {
                tag_error = Some(error.to_string());
                break;
            }
        }

        self.reset_add_form(cx);
        if let Err(error) = self.refresh_library() {
            self.notice = Some(Notice::Error(format!(
                "Meme 已保存，但刷新列表失败：{error}"
            )));
        } else if let Some(error) = tag_error {
            self.notice = Some(Notice::Error(format!(
                "Meme 已保存，但部分 Tag 未关联：{error}"
            )));
            self.page = Page::All;
        } else {
            self.notice = Some(Notice::Success("Meme 已保存到 Inbox".to_owned()));
            self.page = Page::All;
        }
        cx.notify();
    }

    fn reset_add_form(&mut self, cx: &mut Context<Self>) {
        self.draft_contents.clear();
        for input in [
            &self.name_input,
            &self.description_input,
            &self.tags_input,
            &self.text_content_input,
        ] {
            input.update(cx, |input, cx| input.reset(cx));
        }
    }

    fn refresh_library(&mut self) -> Result<(), memelith_core::Error> {
        let database = self.database.as_ref().ok_or_else(|| {
            memelith_core::Error::InvalidDatabase("database is not open".to_owned())
        })?;
        let packs = database.list_meme_packs()?;
        let memes = database.list_all_memes()?;
        self.pack_names = packs.into_iter().map(|pack| (pack.id, pack.name)).collect();
        self.memes = memes;
        Ok(())
    }

    fn navigate(&mut self, page: Page, cx: &mut Context<Self>) {
        if page == Page::All
            && let Err(error) = self.refresh_library()
        {
            self.notice = Some(Notice::Error(format!("无法刷新 Meme：{error}")));
        }
        self.page = page;
        cx.notify();
    }

    fn render_onboarding(&self, cx: &mut Context<Self>) -> AnyElement {
        let button_label = if self.opening_storage {
            "正在打开…"
        } else {
            "选择存储位置"
        };
        div()
            .size_full()
            .bg(rgba(CONTENT_BG))
            .text_color(rgb(INK))
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(52.))
                    .flex_none()
                    .window_control_area(WindowControlArea::Drag),
            )
            .child(
                div().flex_1().flex().items_center().justify_center().child(
                    glass_card()
                        .rounded(px(20.))
                        .w(px(540.))
                        .p_10()
                        .flex()
                        .flex_col()
                        .items_start()
                        .gap_5()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(ACCENT))
                                .child(APPLICATION_NAME.to_uppercase()),
                        )
                        .child(
                            div()
                                .text_size(px(30.))
                                .font_weight(FontWeight::BOLD)
                                .child("先为 Meme 找一个家"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .line_height(px(22.))
                                .text_color(rgba(LABEL_2))
                                .child(
                                    "数据库、图片与文字都会保存在你选择的文件夹中，之后也可以在设置里更换。",
                                ),
                        )
                        .when_some(self.notice.as_ref(), |element, notice| {
                            element.child(render_notice(notice))
                        })
                        .child(
                            primary_pill("choose-storage-onboarding")
                                .on_click(cx.listener(Self::choose_storage))
                                .child(button_label),
                        ),
                ),
            )
            .into_any_element()
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let storage_label: SharedString = self
            .storage_root
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().into_owned().into())
            .unwrap_or_else(|| "未选择".into());
        div()
            .w(px(224.))
            .h_full()
            .flex_none()
            .bg(rgba(SIDEBAR_GLASS))
            .border_r_1()
            .border_color(rgba(0x0000000d))
            .flex()
            .flex_col()
            .child(
                // 预留 macOS 红绿灯区域，同时作为窗口拖拽区
                div()
                    .h(px(56.))
                    .flex_none()
                    .window_control_area(WindowControlArea::Drag),
            )
            .child(
                div()
                    .px_5()
                    .pb_3()
                    .text_size(px(15.))
                    .font_weight(FontWeight::BOLD)
                    .window_control_area(WindowControlArea::Drag)
                    .child(APPLICATION_NAME),
            )
            .child(div().px_4().flex().flex_col().gap_1().children(
                [Page::Add, Page::All, Page::Settings].map(|page| {
                    let selected = self.page == page;
                    div()
                        .id(("nav", page.index()))
                        .h(px(32.))
                        .px_3()
                        .rounded(px(9.))
                        .flex()
                        .items_center()
                        .gap_3()
                        .cursor_pointer()
                        .text_size(px(13.))
                        .text_color(rgb(INK))
                        .when(selected, |style| {
                            style
                                .bg(rgba(GLASS_STRONG))
                                .shadow(control_shadow())
                                .font_weight(FontWeight::SEMIBOLD)
                        })
                        .when(!selected, |style| {
                            style.hover(|style| style.bg(rgba(0xffffff73)))
                        })
                        .active(|style| style.bg(rgba(0xffffffff)))
                        .on_click(cx.listener(move |view, _, _, cx| view.navigate(page, cx)))
                        .child(
                            div()
                                .w(px(20.))
                                .text_center()
                                .text_color(rgb(ACCENT))
                                .child(page.icon()),
                        )
                        .child(page.label())
                }),
            ))
            .child(div().flex_1())
            .child(
                div()
                    .px_5()
                    .pb_4()
                    .text_xs()
                    .text_color(rgba(LABEL_3))
                    .truncate()
                    .child(storage_label),
            )
            .into_any_element()
    }

    fn render_header(&self) -> AnyElement {
        let (title, subtitle): (SharedString, SharedString) = match self.page {
            Page::Add => (
                "添加 Meme".into(),
                "保存后会进入默认的 Inbox".to_owned().into(),
            ),
            Page::All => (
                "全部 Meme".into(),
                format!("共 {} 个，不按 MemePack 分组", self.memes.len()).into(),
            ),
            Page::Settings => ("设置".into(), "管理本地存储位置".to_owned().into()),
        };
        div()
            .flex_none()
            .px_8()
            .pt_6()
            .pb_4()
            .flex()
            .flex_col()
            .gap_1()
            .window_control_area(WindowControlArea::Drag)
            .child(
                div()
                    .text_size(px(26.))
                    .font_weight(FontWeight::BOLD)
                    .child(title),
            )
            .child(div().text_sm().text_color(rgba(LABEL_2)).child(subtitle))
            .into_any_element()
    }

    fn render_add_page(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut content_group = glass_group().flex().flex_col().child(
            group_row()
                .child(
                    div()
                        .flex_1()
                        .text_sm()
                        .text_color(rgba(LABEL_2))
                        .child("至少添加一张图片或一段文字"),
                )
                .child(
                    glass_pill("choose-images")
                        .on_click(cx.listener(Self::choose_images))
                        .child("选择图片"),
                ),
        );
        content_group = content_group.child(hairline()).child(
            group_row()
                .child(div().flex_1().child(self.text_content_input.clone()))
                .child(
                    glass_pill("add-text-content")
                        .on_click(cx.listener(Self::add_text_content))
                        .child("添加文字"),
                ),
        );
        for (index, content) in self.draft_contents.iter().enumerate() {
            content_group = content_group
                .child(hairline())
                .child(self.render_draft_content(index, content, cx));
        }

        div()
            .size_full()
            .id("add-page-scroll")
            .overflow_y_scroll()
            .px_8()
            .pb_8()
            .child(
                div()
                    .w_full()
                    .max_w(px(720.))
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(section_header("内容"))
                            .child(content_group),
                    )
                    .child(
                        div().flex().flex_col().child(section_header("信息")).child(
                            glass_group()
                                .flex()
                                .flex_col()
                                .child(field_row("名称", self.name_input.clone()))
                                .child(hairline())
                                .child(field_row("简介", self.description_input.clone()))
                                .child(hairline())
                                .child(field_row("Tag", self.tags_input.clone())),
                        ),
                    )
                    .child(
                        div().flex().justify_end().child(
                            primary_pill("save-meme")
                                .on_click(cx.listener(Self::save_meme))
                                .child("保存到 Inbox"),
                        ),
                    ),
            )
            .into_any_element()
    }

    fn render_draft_content(
        &self,
        index: usize,
        content: &DraftContent,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let preview = match content {
            DraftContent::Image(path) => div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .size(px(44.))
                        .flex_none()
                        .rounded(px(10.))
                        .overflow_hidden()
                        .bg(rgba(0x3c3c4314))
                        .child(img(path.clone()).size_full().object_fit(ObjectFit::Cover)),
                )
                .child(
                    div().flex_1().text_sm().truncate().child(
                        path.file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                            .unwrap_or_else(|| path.display().to_string()),
                    ),
                )
                .into_any_element(),
            DraftContent::Text(text) => div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .size(px(44.))
                        .flex_none()
                        .rounded(px(10.))
                        .bg(rgba(0x5856d61f))
                        .text_color(rgb(ACCENT))
                        .flex()
                        .items_center()
                        .justify_center()
                        .font_weight(FontWeight::BOLD)
                        .child("Aa"),
                )
                .child(div().flex_1().text_sm().truncate().child(text.clone()))
                .into_any_element(),
        };
        group_row()
            .id(("draft-content", index))
            .child(div().flex_1().min_w_0().child(preview))
            .child(
                icon_button(("remove-draft", index))
                    .on_click(
                        cx.listener(move |view, _, _, cx| view.remove_draft_content(index, cx)),
                    )
                    .child("✕"),
            )
            .into_any_element()
    }

    fn render_all_page(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .size_full()
            .id("all-page-scroll")
            .overflow_y_scroll()
            .px_8()
            .pb_8()
            .when(self.memes.is_empty(), |element| {
                element.child(
                    div()
                        .w_full()
                        .h_full()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(17.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgba(LABEL_2))
                                .child("没有 Meme"),
                        )
                        .child(
                            div()
                                .text_size(px(13.))
                                .text_color(rgba(LABEL_3))
                                .child("从“添加”导入图片或文字，开始整理你的收藏。"),
                        )
                        .child(
                            div().pt_2().child(
                                glass_pill("empty-add")
                                    .on_click(cx.listener(|view, _, _, cx| {
                                        view.navigate(Page::Add, cx);
                                    }))
                                    .child("去添加"),
                            ),
                        ),
                )
            })
            .child(
                div().flex().flex_wrap().gap_5().children(
                    self.memes
                        .iter()
                        .enumerate()
                        .map(|(index, meme)| self.render_meme(index, meme)),
                ),
            )
            .into_any_element()
    }

    fn render_meme(&self, index: usize, meme: &Meme) -> AnyElement {
        let preview = meme
            .contents
            .iter()
            .find_map(|content| match content {
                MemeContent::Image(image) => self.database.as_ref().and_then(|database| {
                    database
                        .resolve_media_path(&image.relative_path)
                        .ok()
                        .map(|path| {
                            div()
                                .h(px(160.))
                                .w_full()
                                .overflow_hidden()
                                .bg(rgba(0x3c3c4314))
                                .child(img(path).size_full().object_fit(ObjectFit::Cover))
                                .into_any_element()
                        })
                }),
                MemeContent::Text(_) => None,
            })
            .or_else(|| {
                meme.contents.iter().find_map(|content| match content {
                    MemeContent::Text(text) => Some(
                        div()
                            .h(px(160.))
                            .w_full()
                            .p_5()
                            .bg(rgba(0x5856d61a))
                            .text_color(rgb(0x4543b8))
                            .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                            .flex()
                            .items_center()
                            .child(text.text.clone())
                            .into_any_element(),
                    ),
                    MemeContent::Image(_) => None,
                })
            })
            .unwrap_or_else(|| {
                div()
                    .h(px(160.))
                    .w_full()
                    .bg(rgba(0x3c3c4314))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgba(LABEL_3))
                    .child("无法预览")
                    .into_any_element()
            });
        let title = meme
            .name
            .clone()
            .unwrap_or_else(|| "未命名 Meme".to_owned());
        let pack_name = self
            .pack_names
            .get(&meme.meme_pack_id)
            .cloned()
            .unwrap_or_else(|| "未知 MemePack".to_owned());
        glass_card()
            .id(("meme", index))
            .w(px(240.))
            .overflow_hidden()
            .cursor_pointer()
            .hover(|style| style.bg(rgba(GLASS_STRONG)))
            .child(preview)
            .child(
                div()
                    .p_4()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .truncate()
                            .child(title),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgba(LABEL_2))
                            .child(format!("{pack_name} · {} 个内容", meme.contents.len())),
                    )
                    .when_some(meme.description.as_ref(), |element, description| {
                        element.child(
                            div()
                                .text_xs()
                                .text_color(rgba(LABEL_2))
                                .truncate()
                                .child(description.clone()),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_settings_page(&self, cx: &mut Context<Self>) -> AnyElement {
        let storage = self
            .storage_root
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "未选择".to_owned());
        div()
            .size_full()
            .id("settings-page-scroll")
            .overflow_y_scroll()
            .px_8()
            .pb_8()
            .child(
                div()
                    .w_full()
                    .max_w(px(720.))
                    .flex()
                    .flex_col()
                    .child(section_header("存储"))
                    .child(
                        glass_group().child(
                            group_row()
                                .child(
                                    div()
                                        .min_w_0()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .child("存储位置"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgba(LABEL_2))
                                                .truncate()
                                                .child(storage),
                                        )
                                        .child(div().text_xs().text_color(rgba(LABEL_3)).child(
                                            "更换位置会打开另一个资料库，不会自动迁移当前数据。",
                                        )),
                                )
                                .child(
                                    glass_pill("change-storage")
                                        .on_click(cx.listener(Self::choose_storage))
                                        .child("更换位置"),
                                ),
                        ),
                    ),
            )
            .into_any_element()
    }
}

impl Render for MemelithView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.database.is_none() {
            return self.render_onboarding(cx);
        }

        let page = match self.page {
            Page::Add => self.render_add_page(cx),
            Page::All => self.render_all_page(cx),
            Page::Settings => self.render_settings_page(cx),
        };
        div()
            .size_full()
            .text_color(rgb(INK))
            .flex()
            .child(self.render_sidebar(cx))
            .child(
                div()
                    .min_w_0()
                    .h_full()
                    .flex_1()
                    .bg(rgba(CONTENT_BG))
                    .flex()
                    .flex_col()
                    .child(self.render_header())
                    .when_some(self.notice.as_ref(), |element, notice| {
                        element.child(div().px_8().pb_3().child(render_notice(notice)))
                    })
                    .child(div().min_h_0().flex_1().child(page)),
            )
            .into_any_element()
    }
}

fn open_library(storage_root: &Path) -> Result<OpenedLibrary, UiError> {
    let model = ClipModel::load_builtin(
        BuiltinModel::ChineseClipVitBasePatch16,
        ExecutionPolicy::Auto,
    )?;
    let mut database = MemeDatabase::open(storage_root, model)?;
    let packs = database.list_meme_packs()?;
    let inbox_id = if let Some(inbox) = packs.iter().find(|pack| pack.name == INBOX_NAME) {
        inbox.id
    } else {
        database
            .create_meme_pack(NewMemePack {
                name: INBOX_NAME.to_owned(),
                description: Some("默认收件箱".to_owned()),
                author: None,
                source: None,
            })?
            .id
    };
    let packs = database.list_meme_packs()?;
    let memes = database.list_all_memes()?;
    Ok(OpenedLibrary {
        database,
        inbox_id,
        memes,
        pack_names: packs.into_iter().map(|pack| (pack.id, pack.name)).collect(),
    })
}

fn field_row(label: impl Into<SharedString>, input: Entity<TextInput>) -> AnyElement {
    group_row()
        .child(
            div()
                .w(px(64.))
                .flex_none()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .child(label.into()),
        )
        .child(div().flex_1().min_w_0().child(input))
        .into_any_element()
}

fn render_notice(notice: &Notice) -> AnyElement {
    let (dot, message) = match notice {
        Notice::Success(message) => (SUCCESS, message),
        Notice::Error(message) => (DANGER, message),
    };
    div()
        .w_full()
        .px_4()
        .py_3()
        .rounded(px(12.))
        .bg(rgba(GLASS_CARD))
        .border_1()
        .border_color(rgba(EDGE_DARK))
        .shadow(card_shadow())
        .flex()
        .items_center()
        .gap_3()
        .child(div().size(px(8.)).flex_none().rounded_full().bg(rgb(dot)))
        .child(div().text_sm().text_color(rgb(INK)).child(message.clone()))
        .into_any_element()
}

fn optional_input_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn parse_tags(value: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    value
        .split([',', '，'])
        .filter_map(optional_input_text)
        .filter(|tag| seen.insert(tag.to_ascii_lowercase()))
        .collect()
}

fn main() {
    Application::new().run(|cx: &mut App| {
        input::init(cx);
        let saved_storage = settings::load_storage_root();
        let bounds = Bounds::centered(None, size(px(1120.), px(760.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(820.), px(560.))),
                app_id: Some("org.memelith.app".to_owned()),
                window_background: WindowBackgroundAppearance::Blurred,
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some(APPLICATION_NAME.into()),
                    appears_transparent: true,
                    traffic_light_position: Some(point(px(24.), px(24.))),
                }),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| MemelithView::new(saved_storage, cx)),
        )
        .expect("failed to open the Memelith window");
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::{optional_input_text, parse_tags};

    #[test]
    fn parses_trimmed_case_insensitive_unique_tags() {
        assert_eq!(
            parse_tags(" Cat, reaction，cat, 中文， "),
            vec!["Cat", "reaction", "中文"]
        );
    }

    #[test]
    fn preserves_absence_for_blank_optional_fields() {
        assert_eq!(optional_input_text("  "), None);
        assert_eq!(optional_input_text("  title  "), Some("title".to_owned()));
    }
}
