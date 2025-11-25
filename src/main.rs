// src/main.rs
mod components;
mod project_config;
mod tei_data;
mod tei_parser;

use components::tei_viewer::TeiViewer;
use gloo_net::http::Request;
use project_config::ProjectConfig;
use yew::prelude::*;

pub enum AppMsg {
    ChangePage(u32),
    ChangeProject(String),
    ManifestsLoaded(Vec<ProjectConfig>),
    ManifestLoadFailed(String),
}

pub struct App {
    current_project: String,
    current_page: u32,
    available_projects: Vec<ProjectConfig>,
    loading: bool,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // Start loading manifests
        ctx.link().send_future(async {
            match load_all_manifests().await {
                Ok(configs) => AppMsg::ManifestsLoaded(configs),
                Err(e) => AppMsg::ManifestLoadFailed(e),
            }
        });

        Self {
            current_project: String::new(),
            current_page: 1,
            available_projects: Vec::new(),
            loading: true,
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
                self.current_page = 1;
                true
            }
            AppMsg::ManifestsLoaded(configs) => {
                self.available_projects = configs;
                self.loading = false;

                // Set the first project as current if available
                if let Some(first) = self.available_projects.first() {
                    self.current_project = first.id.clone();
                }
                true
            }
            AppMsg::ManifestLoadFailed(error) => {
                log::error!("Failed to load manifests: {}", error);
                self.loading = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.loading {
            return html! {
                <div class="app-container">
                    <header class="app-header">
                        <h1>{"Visualizador TEI-XML"}</h1>
                    </header>
                    <main class="app-main">
                        <div class="loading">{"Cargando proyectos..."}</div>
                    </main>
                </div>
            };
        }

        if self.available_projects.is_empty() {
            return html! {
                <div class="app-container">
                    <header class="app-header">
                        <h1>{"Visualizador TEI-XML"}</h1>
                    </header>
                    <main class="app-main">
                        <div class="error">{"No se encontraron proyectos. Por favor, asegúrese de que los archivos manifest.json estén presentes en la carpeta public/projects/"}</div>
                    </main>
                </div>
            };
        }

        let on_page_change = ctx.link().callback(AppMsg::ChangePage);
        let on_project_change = ctx.link().callback(AppMsg::ChangeProject);

        // Find current project config
        let current_project_config = self
            .available_projects
            .iter()
            .find(|p| p.id == self.current_project)
            .cloned();

        let current_project_name = current_project_config
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| self.current_project.clone());

        // Get available pages for current project
        let available_pages = current_project_config
            .as_ref()
            .map(|p| p.pages.clone())
            .unwrap_or_default();

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
                                            if let Ok(page_num) = select.value().parse::<u32>() {
                                                on_change.emit(page_num);
                                            }
                                        }
                                    })
                                }
                            >
                                {for available_pages.iter().map(|page_info| {
                                    html! {
                                        <option
                                            value={page_info.number.to_string()}
                                            selected={self.current_page == page_info.number}
                                        >
                                            {format!("{}", page_info.label)}
                                        </option>
                                    }
                                })}
                            </select>
                        </div>
                    </div>

                    <TeiViewer
                        project={self.current_project.clone()}
                        page={self.current_page}
                    />
                </main>

                <footer class="app-footer">
                    <p>{"TEI-XML Viewer © 2024"}</p>
                </footer>
            </div>
        }
    }
}

async fn load_all_manifests() -> Result<Vec<ProjectConfig>, String> {
    // List of known project directories to check
    // In a real implementation, you might want to fetch a directory listing
    // For now, we'll try to load manifests for known projects
    let project_ids = vec!["PGM-XIII", "Tractatus-Fascinatione", "Chanca", "Example"];

    let mut configs = Vec::new();

    for project_id in project_ids {
        let manifest_url = format!("public/projects/{}/manifest.json", project_id);

        match Request::get(&manifest_url).send().await {
            Ok(resp) => {
                if resp.ok() {
                    match resp.json::<ProjectConfig>().await {
                        Ok(config) => {
                            log::info!("Loaded manifest for project: {}", project_id);
                            configs.push(config);
                        }
                        Err(e) => {
                            log::warn!("Failed to parse manifest for {}: {:?}", project_id, e);
                        }
                    }
                } else {
                    log::warn!("Manifest not found for project: {}", project_id);
                }
            }
            Err(e) => {
                log::warn!("Failed to fetch manifest for {}: {:?}", project_id, e);
            }
        }
    }

    if configs.is_empty() {
        Err("No project manifests could be loaded".to_string())
    } else {
        Ok(configs)
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
