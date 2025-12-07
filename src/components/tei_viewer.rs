// src/components/tei_viewer.rs
use crate::tei_data::*;
use crate::utils::resource_url;
use gloo_net::http::Request;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Event, HtmlImageElement, MouseEvent, PointerEvent, WheelEvent};
use yew::{prelude::*, AttrValue};

#[derive(Properties, PartialEq)]
pub struct TeiViewerProps {
    pub project: String,
    pub page: u32,
}

pub enum TeiViewerMsg {
    LoadDiplomatic(String),
    LoadTranslation(String),
    LoadCommentary(String),
    DiplomaticLoaded(Result<TeiDocument, String>),
    TranslationLoaded(Result<TeiDocument, String>),
    CommentaryLoaded(Result<String, String>),
    HoverLine(String),
    ClickLine(String),
    ClearHover,
    ToggleView(ViewType),
    ToggleCommentary,
    UpdateImageScale(f64),
    StartDrag(MouseEvent),
    DragImage(MouseEvent),
    EndDrag,
    ToggleMetadata,
    ToggleMetadataDip,
    ToggleMetadataTrad,
    ToggleLegend,
    ImageLoaded(Event),
    ImageLoadedWithDimensions(u32, u32),
    StartSplitterDrag(MouseEvent),
    SplitterDrag(MouseEvent),
    EndSplitterDrag,

    PointerDown(i32, i32, i32),
    PointerMove(i32, i32, i32),
    PointerUp(i32, i32, i32),
    PointerLeave(i32, i32, i32),
}

#[derive(Clone, PartialEq)]
pub enum ViewType {
    Diplomatic,
    Translation,
    Both,
    Commentary,
}

pub struct TeiViewer {
    diplomatic: Option<TeiDocument>,
    translation: Option<TeiDocument>,
    commentary: Option<String>,
    hovered_zone: Option<String>,
    locked_zone: Option<String>,
    active_view: ViewType,
    show_image: bool,
    loading: bool,
    error: Option<String>,
    // commentary popup
    show_commentary: bool,
    commentary_first_load: bool,
    // zoom and pan
    image_scale: f32,
    image_offset_x: f32,
    image_offset_y: f32,
    // dragging state
    dragging: bool,
    last_mouse_x: i32,
    last_mouse_y: i32,
    pointers: Vec<(i32, (i32, i32))>,
    last_pointer_distance: f64,
    // metadata popup
    show_metadata_popup: bool,
    metadata_selected: Option<ViewType>,
    current_page: u32,
    current_project: String,
    // legend
    show_legend: bool,
    // image intrinsic dimensions (natural)
    image_nat_w: u32,
    image_nat_h: u32,
    // splitter state
    image_panel_width: f64,
    splitter_dragging: bool,
    splitter_start_x: f64,
    splitter_start_width: f64,
}

impl Component for TeiViewer {
    type Message = TeiViewerMsg;
    type Properties = TeiViewerProps;

    fn create(ctx: &Context<Self>) -> Self {
        let project = ctx.props().project.clone();
        let page = ctx.props().page;

        // Kick off loads
        let dip_path = resource_url(&format!("public/projects/{}/p{}_dip.xml", project, page));
        ctx.link()
            .send_message(TeiViewerMsg::LoadDiplomatic(dip_path));
        let trad_path = resource_url(&format!("public/projects/{}/p{}_trad.xml", project, page));
        ctx.link()
            .send_message(TeiViewerMsg::LoadTranslation(trad_path));
        let commentary_path = resource_url(&format!("public/projects/{}/commentary.html", project));
        ctx.link()
            .send_message(TeiViewerMsg::LoadCommentary(commentary_path));

        Self {
            diplomatic: None,
            translation: None,
            commentary: None,
            hovered_zone: None,
            locked_zone: None,
            active_view: ViewType::Both,
            show_image: true,
            loading: true,
            error: None,
            show_commentary: false, // Will be set to true when commentary loads successfully
            commentary_first_load: true,
            image_scale: 1.0, // Start at normal size
            image_offset_x: 0.0,
            image_offset_y: 0.0,
            dragging: false,
            last_mouse_x: 0,
            last_mouse_y: 0,
            pointers: Vec::new(),
            last_pointer_distance: 0.0,
            show_metadata_popup: false,
            metadata_selected: None,
            current_page: page,
            current_project: project,
            show_legend: false,
            image_nat_w: 0,
            image_nat_h: 0,
            image_panel_width: 45.0,
            splitter_dragging: false,
            splitter_start_x: 0.0,
            splitter_start_width: 45.0,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old: &Self::Properties) -> bool {
        let new_page = ctx.props().page;
        let new_project = ctx.props().project.clone();

        // Check if either page or project changed
        if new_page != self.current_page || new_project != self.current_project {
            self.current_page = new_page;
            self.current_project = new_project.clone();
            self.diplomatic = None;
            self.translation = None;
            self.commentary = None;
            self.loading = true;
            self.error = None;
            self.hovered_zone = None;
            self.locked_zone = None;
            self.image_scale = 0.3;
            self.image_offset_x = 0.0;
            self.image_offset_y = 0.0;
            self.image_nat_w = 0;
            self.image_nat_h = 0;
            // reload
            let cache_bust = js_sys::Date::now() as u64;
            let dip_path = format!(
                "public/projects/{}/p{}_dip.xml?v={}",
                new_project, new_page, cache_bust
            );
            ctx.link()
                .send_message(TeiViewerMsg::LoadDiplomatic(dip_path));
            let trad_path = format!(
                "public/projects/{}/p{}_trad.xml?v={}",
                new_project, new_page, cache_bust
            );
            ctx.link()
                .send_message(TeiViewerMsg::LoadTranslation(trad_path));
            let commentary_path = format!(
                "public/projects/{}/commentary.html?v={}",
                new_project, cache_bust
            );
            ctx.link()
                .send_message(TeiViewerMsg::LoadCommentary(commentary_path));
            true
        } else {
            false
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TeiViewerMsg::ImageLoadedWithDimensions(width, height) => {
                self.image_nat_w = width;
                self.image_nat_h = height;
                true
            }
            TeiViewerMsg::LoadDiplomatic(path) => {
                let link = ctx.link().clone();
                spawn_local(async move {
                    let result = match Request::get(&path).send().await {
                        Ok(resp) => match resp.text().await {
                            Ok(xml) => crate::tei_parser::parse_tei_xml(&xml),
                            Err(e) => Err(format!("Failed to read response text: {:?}", e)),
                        },
                        Err(e) => Err(format!("Failed to load diplomatic: {:?}", e)),
                    };
                    link.send_message(TeiViewerMsg::DiplomaticLoaded(result));
                });
                false
            }
            TeiViewerMsg::LoadTranslation(path) => {
                let link = ctx.link().clone();
                spawn_local(async move {
                    let result = match Request::get(&path).send().await {
                        Ok(resp) => match resp.text().await {
                            Ok(xml) => crate::tei_parser::parse_tei_xml(&xml),
                            Err(e) => Err(format!("Failed to read response text: {:?}", e)),
                        },
                        Err(e) => Err(format!("Failed to load translation: {:?}", e)),
                    };
                    link.send_message(TeiViewerMsg::TranslationLoaded(result));
                });
                false
            }
            TeiViewerMsg::LoadCommentary(path) => {
                let link = ctx.link().clone();
                spawn_local(async move {
                    let result = match Request::get(&path).send().await {
                        Ok(resp) => match resp.text().await {
                            Ok(html) => Ok(html),
                            Err(e) => Err(format!("Failed to read commentary text: {:?}", e)),
                        },
                        Err(e) => Err(format!("Failed to load commentary: {:?}", e)),
                    };
                    link.send_message(TeiViewerMsg::CommentaryLoaded(result));
                });
                false
            }
            TeiViewerMsg::CommentaryLoaded(res) => {
                match res {
                    Ok(html) => {
                        self.commentary = Some(html);
                        // Auto-show only on first load if commentary exists
                        if self.commentary_first_load {
                            self.show_commentary = true;
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to load commentary: {:?}", e);
                        // Set fallback message instead of None
                        self.commentary =
                            Some("<p class=\"sin-comentario\">Sin comentario</p>".to_string());
                        // Auto-show fallback message on first load
                        if self.commentary_first_load {
                            self.show_commentary = true;
                        }
                    }
                }
                true
            }
            TeiViewerMsg::DiplomaticLoaded(res) => {
                match res {
                    Ok(doc) => {
                        self.diplomatic = Some(doc);
                        if self.translation.is_some() {
                            self.loading = false;
                        }
                        if self.show_metadata_popup {
                            self.metadata_selected = Some(ViewType::Diplomatic);
                        }
                    }
                    Err(e) => {
                        // If fetching/parsing fails (for example the XML file is missing or a network error),
                        // treat it as an empty document so the viewer can still display the image and UI.
                        log::warn!("Failed to load diplomatic: {:?}", e);
                        self.diplomatic = Some(TeiDocument::new());
                        // If we already have the translation loaded (even if empty), stop the loading spinner.
                        if self.translation.is_some() {
                            self.loading = false;
                        }
                        // Preserve existing behavior for metadata popup selection.
                        if self.show_metadata_popup {
                            self.metadata_selected = Some(ViewType::Diplomatic);
                        }
                    }
                }
                true
            }
            TeiViewerMsg::TranslationLoaded(res) => {
                match res {
                    Ok(doc) => {
                        self.translation = Some(doc);
                        if self.diplomatic.is_some() {
                            self.loading = false;
                        }
                        if self.show_metadata_popup {
                            if self.diplomatic.is_some() {
                                self.metadata_selected = Some(ViewType::Diplomatic);
                            } else {
                                self.metadata_selected = Some(ViewType::Translation);
                            }
                        }
                    }
                    Err(e) => {
                        // If translation fetch/parsing fails, treat as empty translation so images still show.
                        log::warn!("Failed to load translation: {:?}", e);
                        self.translation = Some(TeiDocument::new());
                        // If we already have the diplomatic loaded (even if empty), stop the loading spinner.
                        if self.diplomatic.is_some() {
                            self.loading = false;
                        }
                        // Preserve existing behavior for metadata popup selection.
                        if self.show_metadata_popup {
                            if self.diplomatic.is_some() {
                                self.metadata_selected = Some(ViewType::Diplomatic);
                            } else {
                                self.metadata_selected = Some(ViewType::Translation);
                            }
                        }
                    }
                }
                true
            }
            TeiViewerMsg::HoverLine(zone) => {
                if self.locked_zone.is_none() {
                    self.hovered_zone = Some(zone);
                    true
                } else {
                    false
                }
            }
            TeiViewerMsg::ClickLine(zone) => {
                if self.locked_zone.as_ref() == Some(&zone) {
                    self.locked_zone = None;
                } else {
                    self.locked_zone = Some(zone);
                }
                true
            }
            TeiViewerMsg::ClearHover => {
                if self.locked_zone.is_none() {
                    self.hovered_zone = None;
                    true
                } else {
                    false
                }
            }
            TeiViewerMsg::ToggleView(view) => {
                self.active_view = view;
                true
            }
            TeiViewerMsg::ToggleCommentary => {
                self.show_commentary = !self.show_commentary;
                // After first manual toggle, don't auto-show anymore
                if self.commentary_first_load {
                    self.commentary_first_load = false;
                }
                true
            }
            TeiViewerMsg::UpdateImageScale(factor) => {
                self.image_scale = (self.image_scale * (factor as f32)).clamp(0.2, 8.0);
                true
            }
            TeiViewerMsg::StartDrag(event) => {
                self.dragging = true;
                self.last_mouse_x = event.client_x();
                self.last_mouse_y = event.client_y();
                false
            }
            TeiViewerMsg::DragImage(event) => {
                if self.dragging {
                    let x = event.client_x();
                    let y = event.client_y();
                    let dx = x - self.last_mouse_x;
                    let dy = y - self.last_mouse_y;
                    self.image_offset_x += dx as f32;
                    self.image_offset_y += dy as f32;
                    self.last_mouse_x = x;
                    self.last_mouse_y = y;
                    true
                } else {
                    false
                }
            }
            TeiViewerMsg::EndDrag => {
                self.dragging = false;
                true
            }
            TeiViewerMsg::PointerDown(id, x, y) => {
                self.pointers.push((id, (x, y)));
                if self.pointers.len() == 1 {
                    // Single pointer - initialize drag position
                    self.last_mouse_x = x;
                    self.last_mouse_y = y;
                } else if self.pointers.len() == 2 {
                    // Two pointers - initialize pinch zoom
                    let p1 = self.pointers[0].1;
                    let p2 = self.pointers[1].1;
                    self.last_pointer_distance =
                        f64::sqrt(((p1.0 - p2.0).pow(2) + (p1.1 - p2.1).pow(2)) as f64);
                }
                self.dragging = true;
                false
            }
            TeiViewerMsg::PointerMove(id, x, y) => {
                if let Some(pointer) = self.pointers.iter_mut().find(|(p_id, _)| *p_id == id) {
                    pointer.1 = (x, y);
                }

                if self.pointers.len() == 2 {
                    // Two-finger pinch zoom
                    let p1 = self.pointers[0].1;
                    let p2 = self.pointers[1].1;
                    let new_dist = f64::sqrt(((p1.0 - p2.0).pow(2) + (p1.1 - p2.1).pow(2)) as f64);

                    // Calculate zoom center (midpoint between two pointers)
                    let center_x = (p1.0 + p2.0) as f32 / 2.0;
                    let center_y = (p1.1 + p2.1) as f32 / 2.0;

                    if self.last_pointer_distance > 0.0 {
                        let scale_factor = (new_dist / self.last_pointer_distance) as f32;
                        let old_scale = self.image_scale;
                        self.image_scale = (self.image_scale * scale_factor).clamp(0.1, 8.0);

                        // Adjust offset so zoom occurs around the gesture center
                        let scale_change = self.image_scale / old_scale;
                        self.image_offset_x =
                            center_x + (self.image_offset_x - center_x) * scale_change;
                        self.image_offset_y =
                            center_y + (self.image_offset_y - center_y) * scale_change;
                    }

                    self.last_pointer_distance = new_dist;
                } else if self.pointers.len() == 1 {
                    // Single-finger pan
                    let dx = x - self.last_mouse_x;
                    let dy = y - self.last_mouse_y;
                    self.image_offset_x += dx as f32;
                    self.image_offset_y += dy as f32;
                    self.last_mouse_x = x;
                    self.last_mouse_y = y;
                }

                true
            }
            TeiViewerMsg::PointerUp(id, _, _) => {
                self.pointers.retain(|(p_id, _)| *p_id != id);

                // Reset distance when transitioning from 2 to 1 pointer
                if self.pointers.len() == 1 {
                    let p = self.pointers[0].1;
                    self.last_mouse_x = p.0;
                    self.last_mouse_y = p.1;
                    self.last_pointer_distance = 0.0;
                } else if self.pointers.is_empty() {
                    self.dragging = false;
                    self.last_pointer_distance = 0.0;
                }

                true
            }
            TeiViewerMsg::PointerLeave(id, _, _) => {
                self.pointers.retain(|(p_id, _)| *p_id != id);

                // Reset distance when transitioning from 2 to 1 pointer
                if self.pointers.len() == 1 {
                    let p = self.pointers[0].1;
                    self.last_mouse_x = p.0;
                    self.last_mouse_y = p.1;
                    self.last_pointer_distance = 0.0;
                } else if self.pointers.is_empty() {
                    self.dragging = false;
                    self.last_pointer_distance = 0.0;
                }

                true
            }
            TeiViewerMsg::ToggleMetadata => {
                self.show_metadata_popup = !self.show_metadata_popup;
                if self.show_metadata_popup {
                    let preferred = match self.active_view {
                        ViewType::Diplomatic => Some(ViewType::Diplomatic),
                        ViewType::Translation => Some(ViewType::Translation),
                        ViewType::Both => {
                            if self.diplomatic.is_some() {
                                Some(ViewType::Diplomatic)
                            } else if self.translation.is_some() {
                                Some(ViewType::Translation)
                            } else {
                                None
                            }
                        }
                        ViewType::Commentary => Some(ViewType::Diplomatic), // Default to diplomatic for commentary
                    };
                    self.metadata_selected = preferred;
                } else {
                    self.metadata_selected = None;
                }
                true
            }
            TeiViewerMsg::ToggleMetadataDip => {
                if self.diplomatic.is_some() {
                    self.metadata_selected = Some(ViewType::Diplomatic);
                }
                true
            }
            TeiViewerMsg::ToggleMetadataTrad => {
                if self.translation.is_some() {
                    self.metadata_selected = Some(ViewType::Translation);
                }
                true
            }
            TeiViewerMsg::ToggleLegend => {
                self.show_legend = !self.show_legend;
                true
            }
            TeiViewerMsg::ImageLoaded(_event) => {
                // Image dimensions will be handled via other means
                true
            }
            TeiViewerMsg::StartSplitterDrag(event) => {
                self.splitter_dragging = true;
                self.splitter_start_x = event.client_x() as f64;
                self.splitter_start_width = self.image_panel_width;
                event.prevent_default();

                // Add global mouse listeners for proper drag behavior
                if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                    let link = ctx.link().clone();
                    let move_callback =
                        wasm_bindgen::closure::Closure::wrap(Box::new(move |e: MouseEvent| {
                            link.send_message(TeiViewerMsg::SplitterDrag(e));
                        })
                            as Box<dyn FnMut(_)>);

                    let link2 = ctx.link().clone();
                    let up_callback =
                        wasm_bindgen::closure::Closure::wrap(Box::new(move |_: MouseEvent| {
                            link2.send_message(TeiViewerMsg::EndSplitterDrag);
                        })
                            as Box<dyn FnMut(_)>);

                    // Store callbacks for cleanup
                    if let Some(body) = document.body() {
                        let _ = body.set_attribute("data-splitter-active", "true");
                    }

                    let _ = document.add_event_listener_with_callback(
                        "mousemove",
                        move_callback.as_ref().unchecked_ref(),
                    );
                    let _ = document.add_event_listener_with_callback(
                        "mouseup",
                        up_callback.as_ref().unchecked_ref(),
                    );

                    move_callback.forget();
                    up_callback.forget();
                }

                true
            }
            TeiViewerMsg::SplitterDrag(event) => {
                if self.splitter_dragging {
                    let current_x = event.client_x() as f64;
                    let dx = current_x - self.splitter_start_x;

                    // Get actual container width from the DOM
                    let container_width =
                        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                            if let Some(container) =
                                document.query_selector(".viewer-content").ok().flatten()
                            {
                                if let Ok(element) = container.dyn_into::<web_sys::HtmlElement>() {
                                    element.client_width() as f64
                                } else {
                                    1000.0
                                }
                            } else {
                                1000.0
                            }
                        } else {
                            1000.0
                        };

                    let dx_percent = (dx / container_width) * 100.0;
                    let new_width = self.splitter_start_width + dx_percent;
                    self.image_panel_width = new_width.max(20.0).min(80.0);
                    true
                } else {
                    false
                }
            }
            TeiViewerMsg::EndSplitterDrag => {
                self.splitter_dragging = false;

                // Clean up global listeners
                if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                    if let Some(body) = document.body() {
                        let _ = body.remove_attribute("data-splitter-active");
                    }
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.loading {
            return html! {
                <div class="loading"><p>{"Cargando documentos TEI..."}</p></div>
            };
        }
        if let Some(err) = &self.error {
            return html! {
                <div class="error"><p>{format!("Error: {}", err)}</p></div>
            };
        }

        // Set CSS custom property for dynamic column sizing
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(body) = document.body() {
                    let _ = body.style().set_property(
                        "--image-panel-width",
                        &format!("{}%", self.image_panel_width),
                    );
                }
            }
        }

        html! {
            <div class="tei-viewer-container">
                { self.render_controls(ctx) }
                { self.render_legend(ctx) }
                <div class="viewer-content">
                    { self.render_image_panel(ctx) }
                    { self.render_splitter(ctx) }
                    { self.render_text_panels(ctx) }
                    { self.render_metadata_popup(ctx) }
                    { self.render_commentary_popup(ctx) }
                </div>
            </div>
        }
    }
}

impl TeiViewer {
    fn render_controls(&self, ctx: &Context<Self>) -> Html {
        let toggle_dip = ctx
            .link()
            .callback(|_| TeiViewerMsg::ToggleView(ViewType::Diplomatic));
        let toggle_trad = ctx
            .link()
            .callback(|_| TeiViewerMsg::ToggleView(ViewType::Translation));
        let toggle_both = ctx
            .link()
            .callback(|_| TeiViewerMsg::ToggleView(ViewType::Both));
        let toggle_commentary = ctx.link().callback(|_| TeiViewerMsg::ToggleCommentary);
        let zoom_in = ctx.link().callback(|_| TeiViewerMsg::UpdateImageScale(1.2));
        let zoom_out = ctx.link().callback(|_| TeiViewerMsg::UpdateImageScale(0.8));
        let toggle_meta = ctx.link().callback(|_| TeiViewerMsg::ToggleMetadata);
        let toggle_legend = ctx.link().callback(|_| TeiViewerMsg::ToggleLegend);

        html! {
            <div class="controls-panel">
                <div class="view-toggles">
                    <button class={if self.active_view == ViewType::Diplomatic { "active" } else { "" }} onclick={toggle_dip}>{"Edición diplomática"}</button>
                    <button class={if self.active_view == ViewType::Translation { "active" } else { "" }} onclick={toggle_trad}>{"Traducción"}</button>
                    <button class={if self.active_view == ViewType::Both { "active" } else { "" }} onclick={toggle_both}>{"Ambas"}</button>
                    <button class={if self.show_commentary { "active" } else { "" }} onclick={toggle_commentary}>{"Comentario"}</button>
                </div>
                <div class="image-controls">
                    <button onclick={zoom_in}>{"🔍 +"}</button>
                    <button onclick={zoom_out}>{"🔍 -"}</button>
                    <span class="zoom-level">{format!("{}%", (self.image_scale * 100.0) as i32)}</span>
                    <button onclick={toggle_meta} title="Toggle Metadata">{ if self.show_metadata_popup { "Ocultar metadata" } else { "Mostrar metadata" } }</button>
                    <button onclick={toggle_legend} title="Toggle Color Legend">{ if self.show_legend { "🎨 Ocultar leyenda" } else { "🎨 Mostrar leyenda" } }</button>
                </div>
            </div>
        }
    }

    fn render_image_panel(&self, ctx: &Context<Self>) -> Html {
        if !self.show_image {
            return html! {};
        }
        let doc = self.diplomatic.as_ref().or(self.translation.as_ref());
        if let Some(doc) = doc {
            // resolve image URL (robust): derive filename and prefer serving from project's images/ directory.
            // If the TEI already contains a public path, use it as-is (but ensure it is an absolute path).
            // If the facsimile image_url is empty, fall back to a page-based filename (e.g. "p1.jpg")
            // derived from the current page prop.
            let image_filename = if doc.facsimile.image_url.trim().is_empty() {
                // use page-based fallback like "p1.jpg"
                format!("p{}.jpg", ctx.props().page)
            } else {
                doc.facsimile
                    .image_url
                    .rsplit('/')
                    .next()
                    .unwrap_or(doc.facsimile.image_url.as_str())
                    .to_string()
            };

            // Use natural image dimensions for display, fall back to declared if not loaded
            let declared_w = doc.facsimile.width;
            let declared_h = doc.facsimile.height;
            let use_w = if self.image_nat_w > 0 {
                self.image_nat_w
            } else {
                declared_w
            };
            let use_h = if self.image_nat_h > 0 {
                self.image_nat_h
            } else {
                declared_h
            };

            // Build an absolute URL (leading slash) for browser requests.
            // Cases handled:
            // - If TEI provides a full http(s) URL, use it as-is.
            // - If TEI provides a path starting with '/', use it as-is (already absolute).
            // - If TEI provides a path starting with 'public/', prefix with '/' to make '/public/...'.
            // - Otherwise, construct '/public/projects/{project}/images/{image_filename}'.
            let image_url = {
                let raw = doc.facsimile.image_url.trim();
                if raw.is_empty() {
                    // TEI didn't specify; use page-based fallback under project images
                    resource_url(&format!(
                        "public/projects/{}/images/{}",
                        ctx.props().project,
                        image_filename
                    ))
                } else if raw.starts_with("http://") || raw.starts_with("https://") {
                    // external absolute URL, use directly
                    raw.to_string()
                } else if raw.starts_with('/') {
                    // already absolute path, use directly
                    raw.to_string()
                } else if raw.starts_with("public/") {
                    // make absolute by adding leading slash
                    format!("/{}", raw)
                } else {
                    // treat as filename or relative path -> place under project images and make absolute
                    resource_url(&format!(
                        "public/projects/{}/images/{}",
                        ctx.props().project,
                        image_filename
                    ))
                }
            };

            let onwheel = ctx.link().callback(|e: WheelEvent| {
                e.prevent_default();
                let delta = -e.delta_y() as f32;
                let factor = if delta > 0.0 { 1.1 } else { 0.9 };
                TeiViewerMsg::UpdateImageScale(factor)
            });

            let onmousedown = {
                let link = ctx.link().clone();
                Callback::from(move |e: MouseEvent| {
                    e.prevent_default();
                    link.send_message(TeiViewerMsg::StartDrag(e));
                })
            };
            let onmousemove = {
                let link = ctx.link().clone();
                Callback::from(move |e: MouseEvent| {
                    link.send_message(TeiViewerMsg::DragImage(e));
                })
            };
            let onmouseup = ctx.link().callback(|_| TeiViewerMsg::EndDrag);
            let onmouseleave = ctx.link().callback(|_| TeiViewerMsg::EndDrag);

            let onpointerdown = {
                let link = ctx.link().clone();
                Callback::from(move |e: PointerEvent| {
                    e.prevent_default();
                    if let Some(target) = e.target() {
                        if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                            let _ = element.set_pointer_capture(e.pointer_id());
                        }
                    }
                    link.send_message(TeiViewerMsg::PointerDown(
                        e.pointer_id(),
                        e.client_x(),
                        e.client_y(),
                    ))
                })
            };
            let onpointermove = ctx.link().callback(|e: PointerEvent| {
                e.prevent_default();
                TeiViewerMsg::PointerMove(e.pointer_id(), e.client_x(), e.client_y())
            });
            let onpointerup = {
                let link = ctx.link().clone();
                Callback::from(move |e: PointerEvent| {
                    e.prevent_default();
                    if let Some(target) = e.target() {
                        if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                            let _ = element.release_pointer_capture(e.pointer_id());
                        }
                    }
                    link.send_message(TeiViewerMsg::PointerUp(
                        e.pointer_id(),
                        e.client_x(),
                        e.client_y(),
                    ))
                })
            };
            let onpointerleave = ctx.link().callback(|e: PointerEvent| {
                e.prevent_default();
                TeiViewerMsg::PointerLeave(e.pointer_id(), e.client_x(), e.client_y())
            });

            // onload captures intrinsic natural size
            let onload = {
                let link = ctx.link().clone();
                Callback::from(move |e: Event| {
                    if let Some(t) = e.target() {
                        if let Ok(img) = t.dyn_into::<HtmlImageElement>() {
                            let nat_w = img.natural_width() as u32;
                            let nat_h = img.natural_height() as u32;

                            // Send message with natural dimensions
                            link.send_message(TeiViewerMsg::ImageLoadedWithDimensions(
                                nat_w, nat_h,
                            ));
                        }
                    }
                })
            };

            // Active zone (hover or locked)
            let active_zone = self.locked_zone.as_ref().or(self.hovered_zone.as_ref());

            // We will render the image and the svg overlay inside the same container.
            // The container receives the pan/zoom transform so both image and svg align perfectly.
            // The SVG's viewBox will be set to natural image size (if available) and polygons converted
            // from TEI facsimile coords into the natural image coordinate space.

            // Create transform style: translate then scale, origin top-left
            let transform_style = format!(
                "transform-origin: 0 0; transform: translate({}px, {}px) scale({}); position: relative; display: inline-block;",
                self.image_offset_x, self.image_offset_y, self.image_scale
            );

            html! {
                <div class="image-panel">
                    <div
                        class="image-container"
                        {onwheel}
                        {onmousedown}
                        {onmousemove}
                        {onmouseup}
                        {onmouseleave}
                        {onpointerdown}
                        {onpointermove}
                        {onpointerup}
                        {onpointerleave}
                        style="position: relative; overflow: hidden; touch-action: none;"
                    >
                        <div class="image-and-overlay" style={transform_style}>
                            <img
                                src={image_url.clone()}
                                onload={onload}
                                style={format!("display:block; width: {}px; height: {}px; max-width: none; max-height: none;", use_w, use_h)}
                            />
                            { self.render_zone_overlays(&doc.facsimile, active_zone, use_w, use_h, declared_w, declared_h) }
                        </div>
                    </div>
                </div>
            }
        } else {
            html! {
                <div class="image-panel"><p>{"No image available"}</p></div>
            }
        }
    }

    /// Render overlays using shared transformed container strategy (SVG inside same container as <img>)
    fn render_zone_overlays(
        &self,
        facsimile: &Facsimile,
        active_zone: Option<&String>,
        display_w: u32,
        display_h: u32,
        declared_w: u32,
        declared_h: u32,
    ) -> Html {
        // Scale zone coordinates from declared space to natural image space

        if display_w == 0 || display_h == 0 {
            return html! {};
        }

        if let Some(zone_id) = active_zone {
            if let Some(zone) = facsimile.zones.get(zone_id) {
                if zone.points.is_empty() {
                    return html! {};
                }

                // Compute scale factors from declared coordinates to natural/display coordinates
                let src_w = if declared_w > 0 {
                    declared_w
                } else {
                    facsimile.width
                };
                let src_h = if declared_h > 0 {
                    declared_h
                } else {
                    facsimile.height
                };

                let factor_x = if src_w > 0 {
                    (display_w as f32) / (src_w as f32)
                } else {
                    1.0
                };
                let factor_y = if src_h > 0 {
                    (display_h as f32) / (src_h as f32)
                } else {
                    1.0
                };

                // Scale coordinates from declared space to natural space
                let points_str = zone
                    .points
                    .iter()
                    .map(|(x, y)| {
                        let px = (*x as f32) * factor_x;
                        let py = (*y as f32) * factor_y;
                        format!("{:.2},{:.2}", px, py)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                // No scaling - both image and SVG use same dimensions, coordinates map 1:1
                return html! {
                    <svg
                        class="overlay-svg"
                        style={format!("position: absolute; top: 0; left: 0; width: {}px; height: {}px; pointer-events: none;", display_w, display_h)}
                        width={display_w.to_string()}
                        height={display_h.to_string()}
                        viewBox={format!("0 0 {} {}", display_w, display_h)}
                        preserveAspectRatio="none"
                        xmlns="http://www.w3.org/2000/svg"
                    >
                        <polygon
                            points={points_str}
                            fill="rgba(255, 255, 0, 0.35)"
                            stroke="yellow"
                            stroke-width="2"
                        />
                    </svg>
                };
            }
        }

        html! {}
    }

    fn render_splitter(&self, ctx: &Context<Self>) -> Html {
        let onmousedown = ctx
            .link()
            .callback(|e: MouseEvent| TeiViewerMsg::StartSplitterDrag(e));

        html! {
            <div
                class="splitter"
                onmousedown={onmousedown}
                title="Drag to resize panels"
            >
                <div class="splitter-handle"></div>
            </div>
        }
    }

    fn render_text_panels(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="text-panels">
                { if self.active_view == ViewType::Diplomatic || self.active_view == ViewType::Both {
                    self.render_diplomatic_panel(ctx)
                } else {
                    html!{}
                } }
                { if self.active_view == ViewType::Translation || self.active_view == ViewType::Both {
                    self.render_translation_panel(ctx)
                } else {
                    html!{}
                } }
            </div>
        }
    }

    fn render_diplomatic_panel(&self, ctx: &Context<Self>) -> Html {
        if let Some(doc) = &self.diplomatic {
            html! {
                <div class="text-panel diplomatic-panel">
                    <h3>{"Edición diplomática"}</h3>
                    <div class="text-content">
                        { for doc.lines.iter().enumerate().map(|(idx, line)| self.render_line(ctx, line, idx)) }
                        { self.render_footnotes(&doc.footnotes) }
                    </div>
                </div>
            }
        } else {
            html! {
                <div class="text-panel diplomatic-panel">
                    <h3>{"Edición diplomática"}</h3>
                    <p>{"Cargando..."}</p>
                </div>
            }
        }
    }

    fn render_translation_panel(&self, ctx: &Context<Self>) -> Html {
        if let Some(doc) = &self.translation {
            html! {
                <div class="text-panel translation-panel">
                    <h3>{"Traducción"}</h3>
                    <div class="text-content">
                        { for doc.lines.iter().enumerate().map(|(idx, line)| self.render_line(ctx, line, idx)) }
                        { self.render_footnotes(&doc.footnotes) }
                    </div>
                </div>
            }
        } else {
            html! {
                <div class="text-panel translation-panel">
                    <h3>{"Traducción"}</h3>
                    <p>{"Cargando..."}</p>
                </div>
            }
        }
    }

    fn render_line(&self, ctx: &Context<Self>, line: &Line, idx: usize) -> Html {
        let zone_id = line.facs.clone();
        let is_active = self.locked_zone.as_ref() == Some(&zone_id)
            || self.hovered_zone.as_ref() == Some(&zone_id);
        let onmouseenter = {
            let zid = zone_id.clone();
            ctx.link()
                .callback(move |_| TeiViewerMsg::HoverLine(zid.clone()))
        };
        let onmouseleave = ctx.link().callback(|_| TeiViewerMsg::ClearHover);
        let onclick = {
            let zid = zone_id.clone();
            ctx.link()
                .callback(move |_| TeiViewerMsg::ClickLine(zid.clone()))
        };
        let class = if is_active { "line active" } else { "line" };

        html! {
            <div class={class} {onmouseenter} {onmouseleave} {onclick}>
                <span class="line-number">{ idx + 1 }</span>
                <span class="line-content">{ for line.content.iter().map(|n| self.render_text_node(n)) }</span>
            </div>
        }
    }

    fn render_text_node(&self, node: &TextNode) -> Html {
        match node {
            TextNode::Text { content } => html! { <>{content}</> },
            TextNode::Abbr { abbr, expan } => html! {
                <abbr title={format!("[Abreviatura] {}", expan)} class="abbreviation" data-tooltip-type="abbr">{ abbr }</abbr>
            },
            TextNode::Choice { sic, corr } => html! {
                <span class="correction" title={format!("[Corrección] Lectura: {}", corr)}>{ sic }</span>
            },
            TextNode::Regularised { orig, reg } => html! {
                <span class="regularised" title={format!("[Regularización] Original: {}", orig)}>{ reg }</span>
            },
            TextNode::Num { value, tipo, text } => html! {
                <span class="number" title={format!("[Número] Valor: {} | Tipo: {}", value, tipo)}>{ text }</span>
            },
            TextNode::PersName {
                content,
                tipo,
                firstname,
                continued,
                ref_uri,
            } => {
                // Build a descriptive title from available attributes
                let mut title_parts: Vec<String> = Vec::new();
                if !tipo.is_empty() {
                    title_parts.push(format!("[Persona] Tipo: {}", tipo));
                } else {
                    title_parts.push("[Persona]".to_string());
                }
                if let Some(fnme) = firstname {
                    title_parts.push(format!("Nombre: {}", fnme));
                }
                if continued.unwrap_or(false) {
                    title_parts.push("Continúa".to_string());
                }
                if let Some(r) = ref_uri {
                    title_parts.push(format!("Ref: {}", r));
                }

                // Check for nested abbreviations and add their info to the combined title
                for node in content {
                    if let TextNode::Abbr { abbr, expan } = node {
                        title_parts.push(format!("[Abreviatura] {}: {}", abbr, expan));
                    }
                }

                let title = title_parts.join(" | ");

                html! {
                    <span class="person-name" title={title} data-tooltip-type="person">
                        { for content.iter().map(|n| self.render_text_node_no_abbr_tooltip(n)) }
                    </span>
                }
            }
            TextNode::PlaceName { name, attrs } => {
                // Show only the visible place name inline. Ancillary attributes
                // (e.g., country, region) are exposed via the element's title so
                // they appear when hovering. This keeps the inline flow intact.
                let mut title_parts: Vec<String> = Vec::new();
                for (k, v) in attrs.iter() {
                    // Normalize key names for display (optional)
                    title_parts.push(format!("{}: {}", k, v));
                }
                let title = if title_parts.is_empty() {
                    format!("[Lugar]: {}", name)
                } else {
                    format!("{} — {}", title_parts.join("; "), name)
                };
                html! {
                    <span class="place-name" title={title.clone()}>{ name }</span>
                }
            }
            TextNode::Ref {
                ref_type,
                target,
                content,
            } => html! {
                <span class="ref" title={format!("[Referencia] Tipo: {} | Destino: {}", ref_type, target)}>{ content }</span>
            },
            TextNode::Unclear { reason, content } => html! {
                <span class="unclear" title={format!("[Incierto] Razón: {}", reason)}>{ content }</span>
            },
            TextNode::RsType { rs_type, content } => html! {
                <span class={format!("rs-type rs-{}", rs_type)} title={format!("[Cadena de Referencia] Tipo: {}", rs_type)}>{ content }</span>
            },
            TextNode::NoteRef { note_id, n } => html! {
                <sup class="footnote-ref" title="[Nota al pie]">
                    <a id={format!("ref_{}", note_id)} href={format!("#{}", note_id)}>{ n }</a>
                </sup>
            },
            TextNode::InlineNote { content, n } => html! {
                <sup class="footnote-ref" title={format!("[Nota al pie] {}", content)}>{ n }</sup>
            },
            TextNode::Hi { rend, content } => {
                // Handle multiple rend values (e.g., "bold italic")
                // Render nested nodes instead of a single string content.
                // We rely on text nodes to carry their own leading/trailing space,
                // so simply rendering nested nodes in order preserves spacing.
                let classes = rend
                    .split_whitespace()
                    .map(|r| format!("hi-{}", r))
                    .collect::<Vec<_>>()
                    .join(" ");

                // Only show titles for non-basic formatting to avoid clustering
                // Basic formatting (bold, italic, underline) is visually obvious
                let basic_formatting = ["bold", "italic", "underline", "superscript", "subscript"];
                let show_title = !rend
                    .split_whitespace()
                    .all(|r| basic_formatting.contains(&r));

                if show_title {
                    html! {
                        <span class={classes} title={format!("[Resaltado] Estilo: {}", rend)}>
                            { for content.iter().map(|n| self.render_text_node(n)) }
                        </span>
                    }
                } else {
                    html! {
                        <span class={classes}>
                            { for content.iter().map(|n| self.render_text_node(n)) }
                        </span>
                    }
                }
            }
        }
    }

    fn render_text_node_no_abbr_tooltip(&self, node: &TextNode) -> Html {
        match node {
            TextNode::Text { content } => html! { <>{content}</> },
            TextNode::Abbr { abbr, expan: _ } => html! {
                <abbr class="abbreviation">{ abbr }</abbr>
            },
            TextNode::Choice { sic, corr } => html! {
                <span class="correction" title={format!("[Corrección] Lectura: {}", corr)}>{ sic }</span>
            },
            TextNode::Regularised { orig, reg } => html! {
                <span class="regularised" title={format!("[Regularización] Original: {}", orig)}>{ reg }</span>
            },
            TextNode::Num { value, tipo, text } => html! {
                <span class="number" title={format!("[Número] Valor: {} | Tipo: {}", value, tipo)}>{ text }</span>
            },
            TextNode::PersName {
                content,
                tipo,
                firstname,
                continued,
                ref_uri,
            } => {
                // Nested person names should use regular rendering
                self.render_text_node(&TextNode::PersName {
                    content: content.clone(),
                    tipo: tipo.clone(),
                    firstname: firstname.clone(),
                    continued: *continued,
                    ref_uri: ref_uri.clone(),
                })
            }
            TextNode::PlaceName { name, attrs } => {
                let mut title_parts: Vec<String> = Vec::new();
                for (k, v) in attrs.iter() {
                    title_parts.push(format!("{}: {}", k, v));
                }
                let title = if title_parts.is_empty() {
                    format!("[Lugar]: {}", name)
                } else {
                    format!("{} — {}", title_parts.join("; "), name)
                };
                html! {
                    <span class="place-name" title={title}>{ name }</span>
                }
            }
            TextNode::Ref {
                ref_type,
                target,
                content,
            } => html! {
                <span class="ref" title={format!("[Referencia] Tipo: {} | Destino: {}", ref_type, target)}>{ content }</span>
            },
            TextNode::Unclear { reason, content } => html! {
                <span class="unclear" title={format!("[Incierto] Razón: {}", reason)}>{ content }</span>
            },
            TextNode::RsType { rs_type, content } => html! {
                <span class={format!("rs-type rs-{}", rs_type)} title={format!("[Cadena de Referencia] Tipo: {}", rs_type)}>{ content }</span>
            },
            TextNode::NoteRef { note_id, n } => html! {
                <sup class="footnote-ref" title="[Nota al pie]">
                    <a id={format!("ref_{}", note_id)} href={format!("#{}", note_id)}>{ n }</a>
                </sup>
            },
            TextNode::InlineNote { content, n } => html! {
                <sup class="footnote-ref" title={format!("[Nota al pie] {}", content)}>{ n }</sup>
            },
            TextNode::Hi { rend, content } => {
                let classes = rend
                    .split_whitespace()
                    .map(|r| format!("hi-{}", r))
                    .collect::<Vec<_>>()
                    .join(" ");

                let basic_formatting = ["bold", "italic", "underline", "superscript", "subscript"];
                let show_title = !rend
                    .split_whitespace()
                    .all(|r| basic_formatting.contains(&r));

                if show_title {
                    html! {
                        <span class={classes} title={format!("[Resaltado] Estilo: {}", rend)}>
                            { for content.iter().map(|n| self.render_text_node_no_abbr_tooltip(n)) }
                        </span>
                    }
                } else {
                    html! {
                        <span class={classes}>
                            { for content.iter().map(|n| self.render_text_node_no_abbr_tooltip(n)) }
                        </span>
                    }
                }
            }
        }
    }

    fn render_legend(&self, ctx: &Context<Self>) -> Html {
        if !self.show_legend {
            return html! {};
        }

        let on_close = ctx.link().callback(|_| TeiViewerMsg::ToggleLegend);

        html! {
            <div class="legend-panel">
                <div class="legend-header">
                    <h3>{"Leyenda de Colores"}</h3>
                    <button class="close-btn" onclick={on_close}>{"×"}</button>
                </div>
                <div class="legend-items">
                    <div class="legend-item">
                        <span class="legend-swatch abbreviation">{"Ab"}</span>
                        <span class="legend-label">{"Abreviatura"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch correction">{"Co"}</span>
                        <span class="legend-label">{"Corrección"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch regularised">{"Rg"}</span>
                        <span class="legend-label">{"Regularización"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch number">{"12"}</span>
                        <span class="legend-label">{"Número"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch person-name">{"Pe"}</span>
                        <span class="legend-label">{"Persona"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch place-name">{"Lu"}</span>
                        <span class="legend-label">{"Lugar"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch ref">{"Rf"}</span>
                        <span class="legend-label">{"Referencia"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch unclear">{"??"}</span>
                        <span class="legend-label">{"Texto incierto"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch rs-divine">{"Dv"}</span>
                        <span class="legend-label">{"Entidad divina"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch rs-astral">{"As"}</span>
                        <span class="legend-label">{"Entidad astral"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch footnote-ref">{"1"}</span>
                        <span class="legend-label">{"Nota al pie"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch hi-bold">{"N"}</span>
                        <span class="legend-label">{"Negrita"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch hi-italic">{"C"}</span>
                        <span class="legend-label">{"Cursiva"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch hi-superscript">{"x²"}</span>
                        <span class="legend-label">{"Superíndice"}</span>
                    </div>
                    <div class="legend-item">
                        <span class="legend-swatch hi-subscript">{"H₂O"}</span>
                        <span class="legend-label">{"Subíndice"}</span>
                    </div>
                </div>
            </div>
        }
    }

    fn render_footnotes(&self, footnotes: &[Footnote]) -> Html {
        if footnotes.is_empty() {
            return html! {};
        }

        html! {
            <div class="footnotes-section">
                <hr class="footnotes-divider" />
                <h4>{"Notas"}</h4>
                <ol class="footnotes-list">
                    { for footnotes.iter().map(|note| {
                        let note_num = note.n.clone();
                        let note_id = note.id.clone();
                        html! {
                            <li id={note_id.clone()} class="footnote-item">
                                <a href={format!("#ref_{}", note_id)} class="footnote-number">{ &note_num }</a>
                                <span class="footnote-content">{ &note.content }</span>
                            </li>
                        }
                    }) }
                </ol>
            </div>
        }
    }

    fn render_metadata_popup(&self, ctx: &Context<Self>) -> Html {
        if !self.show_metadata_popup {
            return html! {};
        }
        let dip = self.diplomatic.as_ref();
        let trad = self.translation.as_ref();
        let on_close = ctx.link().callback(|_| TeiViewerMsg::ToggleMetadata);
        let on_toggle_dip = ctx.link().callback(|_| TeiViewerMsg::ToggleMetadataDip);
        let on_toggle_trad = ctx.link().callback(|_| TeiViewerMsg::ToggleMetadataTrad);

        html! {
            <div class="metadata-popup-overlay">
                <div class="metadata-popup">
                    <div class="metadata-popup-header">
                        <h2>{"Metadatos"}</h2>
                        <button class="close-btn" onclick={on_close}>{"×"}</button>
                    </div>
                    <div class="metadata-popup-selectors">
                        <label>
                            <input type="radio" name="metadata-select"
                                checked={matches!(self.metadata_selected, Some(ViewType::Diplomatic))}
                                onclick={on_toggle_dip} />
                            {"Diplomática"}
                        </label>
                        <label>
                            <input type="radio" name="metadata-select"
                                checked={matches!(self.metadata_selected, Some(ViewType::Translation))}
                                onclick={on_toggle_trad} />
                            {"Traducción"}
                        </label>
                    </div>
                    <div class="metadata-popup-content">
                        { if matches!(self.metadata_selected, Some(ViewType::Diplomatic)) && dip.is_some() {
                            self.render_metadata_panel_for(dip, "Edición Diplomática")
                        } else if matches!(self.metadata_selected, Some(ViewType::Translation)) && trad.is_some() {
                            self.render_metadata_panel_for(trad, "Traducción")
                        } else {
                            html!{ <p>{"No hay metadatos disponibles para la edición seleccionada."}</p> }
                        } }
                    </div>
                </div>
            </div>
        }
    }

    fn render_metadata_panel_for(&self, doc_opt: Option<&TeiDocument>, label: &str) -> Html {
        if let Some(doc) = doc_opt {
            html! {
                <>
                    <h3>{ label }</h3>
                    <dl>
                        <dt>{"Título:"}</dt><dd>{ &doc.metadata.title }</dd>
                        <dt>{"Autor:"}</dt><dd>{ &doc.metadata.author }</dd>
                        <dt>{"Editor:"}</dt><dd>{ &doc.metadata.editor }</dd>
                        <dt>{"Tipo de Edición:"}</dt><dd>{ &doc.metadata.edition_type }</dd>
                        <dt>{"Idioma:"}</dt><dd>{ &doc.metadata.language }</dd>
                        { if let Some(c) = &doc.metadata.country { html!{<><dt>{"País:"}</dt><dd>{c}</dd></>} } else { html!{} } }
                        { if let Some(s) = &doc.metadata.settlement { html!{<><dt>{"Ciudad:"}</dt><dd>{s}</dd></>} } else { html!{} } }
                        { if let Some(i) = &doc.metadata.institution { html!{<><dt>{"Institución:"}</dt><dd>{i}</dd></>} } else { html!{} } }
                        { if let Some(col) = &doc.metadata.collection { html!{<><dt>{"Colección:"}</dt><dd>{col}</dd></>} } else { html!{} } }
                        { if let Some(sig) = &doc.metadata.siglum { html!{<><dt>{"Sigla:"}</dt><dd>{sig}</dd></>} } else { html!{} } }
                    </dl>
                    <h4>{"Información de Imagen"}</h4>
                    <dl>
                        <dt>{"ID de Superficie:"}</dt><dd>{ &doc.facsimile.surface_id }</dd>
                        <dt>{"Archivo de Imagen:"}</dt><dd>{ &doc.facsimile.image_url }</dd>
                        <dt>{"Dimensiones Declaradas:"}</dt><dd>{ format!("{} × {} píxeles", doc.facsimile.width, doc.facsimile.height) }</dd>
                        <dt>{"Dimensiones Intrínsecas (cargadas):"}</dt><dd>{ format!("{} × {} píxeles", self.image_nat_w, self.image_nat_h) }</dd>
                        <dt>{"Zonas:"}</dt><dd>{ format!("{} zonas", doc.facsimile.zones.len()) }</dd>
                        <dt>{"Líneas:"}</dt><dd>{ format!("{} líneas", doc.lines.len()) }</dd>
                    </dl>
                </>
            }
        } else {
            html! {}
        }
    }

    fn render_commentary_popup(&self, ctx: &Context<Self>) -> Html {
        if !self.show_commentary {
            return html! {};
        }

        let on_close = ctx.link().callback(|_| TeiViewerMsg::ToggleCommentary);
        let fallback_message = "<p class=\"sin-comentario\">Sin comentario</p>".to_string();
        let commentary_html = self.commentary.as_ref().unwrap_or(&fallback_message);

        html! {
            <div class="commentary-popup-overlay">
                <div class="commentary-popup">
                    <div class="commentary-popup-header">
                        <h2>{"Comentario"}</h2>
                        <button class="close-btn" onclick={on_close}>{"×"}</button>
                    </div>
                    <div class="commentary-popup-content">
                        <div class="commentary-html-content">
                            { Html::from_html_unchecked(AttrValue::from(commentary_html.clone())) }
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
