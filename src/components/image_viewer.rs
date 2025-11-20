// src/components/image_viewer.rs
use gloo::events::EventListener;
use gloo::utils::document;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::events::WheelEvent;
use yew::prelude::*;
use yew::{Callback, MouseEvent};

#[derive(Properties, PartialEq)]
pub struct ImageViewerProps {
    pub images: Vec<String>,
}

#[function_component(ImageViewer)]
pub fn image_viewer(props: &ImageViewerProps) -> Html {
    let current_index = use_state(|| 0usize);
    let scale = use_state(|| 1.0f32);
    let offset_x = use_state(|| 0.0f32);
    let offset_y = use_state(|| 0.0f32);

    // DRAG STATE
    let dragging = use_state(|| false);
    let last_mouse = use_state(|| (0.0f32, 0.0f32));

    // Container ref for keyboard focus
    let container_ref = use_node_ref();

    // ------ KEYBOARD NAVIGATION ------
    {
        let current_index = current_index.clone();
        let images = props.images.clone();
        let scale = scale.clone();
        let offset_x = offset_x.clone();
        let offset_y = offset_y.clone();

        use_effect_with(
            (
                current_index.clone(),
                images.clone(),
                scale.clone(),
                offset_x.clone(),
                offset_y.clone(),
            ),
            move |(_, _, _, _, _)| {
                let current_index = current_index.clone();
                let images = images.clone();
                let scale = scale.clone();
                let offset_x = offset_x.clone();
                let offset_y = offset_y.clone();

                let listener = EventListener::new(&document(), "keydown", move |event| {
                    let keyboard_event = event.dyn_ref::<KeyboardEvent>().unwrap();

                    match keyboard_event.key().as_str() {
                        "ArrowRight" | "d" | " " => {
                            // Next image
                            let new_index = (*current_index + 1) % images.len();
                            current_index.set(new_index);
                            keyboard_event.prevent_default();
                        }
                        "ArrowLeft" | "a" => {
                            // Previous image
                            let new_index = if *current_index == 0 {
                                images.len() - 1
                            } else {
                                *current_index - 1
                            };
                            current_index.set(new_index);
                            keyboard_event.prevent_default();
                        }
                        "r" | "R" => {
                            // Reset view
                            scale.set(1.0);
                            offset_x.set(0.0);
                            offset_y.set(0.0);
                            keyboard_event.prevent_default();
                        }
                        "+" | "=" => {
                            // Zoom in
                            let new_scale = (*scale * 1.2).min(6.0);
                            scale.set(new_scale);
                            keyboard_event.prevent_default();
                        }
                        "-" | "_" => {
                            // Zoom out
                            let new_scale = (*scale / 1.2).max(0.3);
                            scale.set(new_scale);
                            keyboard_event.prevent_default();
                        }
                        _ => {}
                    }
                });

                // Cleanup closure
                || drop(listener)
            },
        );
    }

    // ------ SCROLL ZOOM ------
    let onwheel = {
        let scale = scale.clone();
        Callback::from(move |e: WheelEvent| {
            e.prevent_default();
            let delta = -e.delta_y() as f32 / 500.0;
            let new_scale = (*scale + delta).clamp(0.3, 6.0);
            scale.set(new_scale);
        })
    };

    // ------ DRAG START ------
    let onmousedown = {
        let dragging = dragging.clone();
        let last_mouse = last_mouse.clone();
        Callback::from(move |e: MouseEvent| {
            dragging.set(true);
            last_mouse.set((e.client_x() as f32, e.client_y() as f32));
        })
    };

    // ------ DRAG END ------
    let onmouseup = {
        let dragging = dragging.clone();
        Callback::from(move |_| dragging.set(false))
    };

    // ------ DRAG MOVE ------
    let onmousemove = {
        let dragging = dragging.clone();
        let last_mouse = last_mouse.clone();
        let offset_x = offset_x.clone();
        let offset_y = offset_y.clone();

        Callback::from(move |e: MouseEvent| {
            if *dragging {
                let (lx, ly) = *last_mouse;
                let (cx, cy) = (e.client_x() as f32, e.client_y() as f32);

                let dx = cx - lx;
                let dy = cy - ly;

                offset_x.set(*offset_x + dx);
                offset_y.set(*offset_y + dy);

                last_mouse.set((cx, cy));
            }
        })
    };

    // ------ CHANGE PAGES ------
    let next_image = {
        let current_index = current_index.clone();
        let images = props.images.clone();
        Callback::from(move |_| {
            let new_index = (*current_index + 1) % images.len();
            current_index.set(new_index);
        })
    };

    let prev_image = {
        let current_index = current_index.clone();
        let images = props.images.clone();
        Callback::from(move |_| {
            let new_index = if *current_index == 0 {
                images.len() - 1
            } else {
                *current_index - 1
            };
            current_index.set(new_index);
        })
    };

    // ------ RESET VIEW ------
    let reset_view = {
        let scale = scale.clone();
        let offset_x = offset_x.clone();
        let offset_y = offset_y.clone();
        Callback::from(move |_| {
            scale.set(1.0);
            offset_x.set(0.0);
            offset_y.set(0.0);
        })
    };

    // Current image
    let src = props.images[*current_index].clone();

    html! {
        <div ref={container_ref} style="user-select:none; outline: none;" tabindex="0">
            <div style="margin-bottom:10px; display:flex; gap:8px; align-items:center; flex-wrap: wrap;">
                <button onclick={prev_image} title="Previous (← or A)">{"← Prev"}</button>
                <button onclick={next_image} title="Next (→, D, or Space)">{"Next →"}</button>

                <button onclick={{
                    let scale = scale.clone();
                    Callback::from(move |_| scale.set((*scale * 1.2).min(6.0)))
                }} title="Zoom In (+)">{"🔍 +"}</button>

                <button onclick={{
                    let scale = scale.clone();
                    Callback::from(move |_| scale.set((*scale / 1.2).max(0.3)))
                }} title="Zoom Out (-)">{"🔍 -"}</button>

                <button onclick={reset_view} title="Reset View (R)">{"⟲ Reset"}</button>

                <div style="margin-left:auto; padding: 0 8px; font-family: monospace; background: #f0f0f0; border-radius: 4px; padding: 4px 8px;">
                    { format!("{}/{} | {}% | Pan: ({:.0}, {:.0})",
                        *current_index + 1,
                        props.images.len(),
                        (*scale * 100.0) as i32,
                        *offset_x,
                        *offset_y
                    ) }
                </div>
            </div>

            <div
                onwheel={onwheel}
                {onmousedown}
                {onmouseup}
                {onmousemove}
                style="
                    width:100%;
                    height:80vh;
                    overflow:hidden;
                    border:1px solid #bbb;
                    position:relative;
                    cursor:grab;
                    background: #fafafa;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                "
            >
                <img
                    src={src}
                    style={format!(
                        "
                            position:absolute;
                            transform: translate({}px, {}px) scale({});
                            transform-origin: top left;
                            max-width:none;
                            pointer-events:none;
                        ",
                        *offset_x,
                        *offset_y,
                        *scale
                    )}
                />
            </div>

            <div style="margin-top: 10px; font-size: 12px; color: #666;">
                <p>{"📖 Keyboard shortcuts: ← / A (prev), → / D / Space (next), + / - (zoom), R (reset)"}</p>
            </div>
        </div>
    }
}
