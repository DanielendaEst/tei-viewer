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
    ChangeProject(String),
}

#[derive(Clone, Debug)]
pub struct Project {
    id: String,
    name: String,
}

pub struct App {
    current_project: String,
    current_page: String,
    available_projects: Vec<Project>,
    available_pages: Vec<String>,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        // Available projects - hardcoded for now, could be loaded from /projects directory
        let available_projects = vec![
            Project {
                id: "PGM-XIII".to_string(),
                name: "Papyri Graecae Magicae XIII".to_string(),
            },
            Project {
                id: "Tractatus-Fascinatione".to_string(),
                name: "Tractatus de Fascinatione".to_string(),
            },
        ];

        let current_project = available_projects
            .first()
            .map(|p| p.id.clone())
            .unwrap_or_else(|| "PGM-XIII".to_string());

        let available_pages = get_unique_pages();
        let first_page = available_pages
            .first()
            .cloned()
            .unwrap_or_else(|| "p1".to_string());

        Self {
            current_project,
            current_page: first_page,
            available_projects,
            available_pages,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::ChangePage(page) => {
                self.current_page = page;
                true
            }
            AppMsg::ChangeProject(project) => {
                self.current_project = project;
                // Reset to first page when changing projects
                self.current_page = self
                    .available_pages
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "p1".to_string());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_page_change = ctx.link().callback(AppMsg::ChangePage);
        let on_project_change = ctx.link().callback(AppMsg::ChangeProject);

        // Find current project name
        let current_project_name = self
            .available_projects
            .iter()
            .find(|p| p.id == self.current_project)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| self.current_project.clone());

        html! {
            <div class="app-container">
                <header class="app-header">
                    <h1>{"Visualizador TEI-XML"}</h1>
                    <p class="subtitle">{format!("Visualizador interactivo - {}", current_project_name)}</p>
                </header>

                <main class="app-main">
                    <div class="selectors-container">
                        <div class="project-selector">
                            <label for="project-select">{"Proyecto: "}</label>
                            <select
                                id="project-select"
                                onchange={
                                    let on_change = on_project_change.clone();
                                    Callback::from(move |e: Event| {
                                        let target = e.target_dyn_into::<web_sys::HtmlSelectElement>();
                                        if let Some(select) = target {
                                            on_change.emit(select.value());
                                        }
                                    })
                                }
                            >
                                {for self.available_projects.iter().map(|project| {
                                    html! {
                                        <option
                                            value={project.id.clone()}
                                            selected={&self.current_project == &project.id}
                                        >
                                            {project.name.clone()}
                                        </option>
                                    }
                                })}
                            </select>
                        </div>

                        <div class="page-selector">
                            <label for="page-select">{"Página: "}</label>
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
                                            {format!("Página {}", page)}
                                        </option>
                                    }
                                })}
                            </select>
                        </div>
                    </div>

                    <TeiViewer
                        project={self.current_project.clone()}
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
