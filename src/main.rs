// src/main.rs
mod components;
mod generated_pages;
mod tei_data;
mod tei_parser;

use components::tei_viewer::TeiViewer;
use generated_pages::get_unique_pages;
use yew::prelude::*;

pub enum AppMsg {
    ChangePage(String),
}

pub struct App {
    current_page: String,
    available_pages: Vec<String>,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        // Dynamically loaded from scanning projects directory
        let available_pages = get_unique_pages();
        let first_page = available_pages
            .first()
            .cloned()
            .unwrap_or_else(|| "p1".to_string());

        Self {
            current_page: first_page,
            available_pages,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::ChangePage(page) => {
                self.current_page = page;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_page_change = ctx.link().callback(AppMsg::ChangePage);

        html! {
            <div class="app-container">
                <header class="app-header">
                    <h1>{"Visualizador TEI-XML"}</h1>
                    <p class="subtitle">{"Visualizador interactivo para el PGM XIII"}</p>
                </header>

                <main class="app-main">
                    <div class="page-selector">
                        <label for="page-select">{"Select Page: "}</label>
                        <select
                            id="page-select"
                            onchange={
                                let on_change = on_page_change.clone();
                                Callback::from(move |e: Event| {
                                    let target = e.target_dyn_into::<web_sys::HtmlSelectElement>();
                                    if let Some(select) = target {
                                        on_change.emit(select.value());
                                    }
                                })
                            }
                        >
                            {for self.available_pages.iter().map(|page| {
                                html! {
                                    <option
                                        value={page.clone()}
                                        selected={&self.current_page == page}
                                    >
                                        {format!("Page {}", page)}
                                    </option>
                                }
                            })}
                        </select>
                    </div>

                    <TeiViewer
                        project="PGM-XIII"
                        page={self.current_page.clone()}
                    />
                </main>

                <footer class="app-footer">
                    <p>{"TEI-XML Viewer © 2024"}</p>
                </footer>
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
