// src/main.rs
mod components;
mod project_config;
mod tei_data;
mod tei_parser;
mod utils;

use components::tei_viewer::TeiViewer;
use gloo_net::http::Request;
use project_config::ProjectConfig;
use utils::resource_url;
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
                    <p class="subtitle">{format!("Un especial agradecimiento al profesor Robert W. Daniel por permitirnos utilizar su edición
                        diplomática del papiro y al profesor Juan Felipe González por su dirección en este trabajo. ¡Gracias!")}</p>
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
                    <a href="https://github.com/federicogaviriaz/tei-viewer"
                       target="_blank"
                       rel="noopener noreferrer"
                       style="display: inline-flex; align-items: center; gap: 4px; text-decoration: none;">


                      <svg height="16" width="16" viewBox="0 0 16 16" aria-hidden="true">
                        <path fill="white"
                          d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29
                          6.53 5.47 7.59.4.07.55-.17.55-.38
                          0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52
                          -.01-.53.63-.01 1.08.58 1.23.82.72
                          1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2
                          -3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2
                          -.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18
                          1.32-.27 2-.27.68 0 1.36.09 2 .27
                          1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08
                          2.12.51.56.82 1.27.82 2.15 0 3.07-1.87
                          3.75-3.65 3.95.29.25.54.73.54 1.48
                          0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013
                          8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
                      </svg>


                    </a>

                </footer>
            </div>
        }
    }
}

async fn load_all_manifests() -> Result<Vec<ProjectConfig>, String> {
    // List of known project directories to check
    // In a real implementation, you might want to fetch a directory listing
    // For now, we'll try to load manifests for known projects
    let project_ids = vec!["PGM-XIII"];

    let mut configs = Vec::new();

    for project_id in project_ids {
        let manifest_url = resource_url(&format!("public/projects/{}/manifest.json", project_id));

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
