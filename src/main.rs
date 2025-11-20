mod components;


use components::image_viewer::ImageViewer;
use yew::prelude::*;


#[function_component(App)]
fn app() -> Html {
    let images = vec![
            "static/images/p1.jpg".to_string(),
            "static/images/p2.jpg".to_string(),      
            "static/images/p3.jpg".to_string(),
            "static/images/p4.jpg".to_string(),
            "static/images/p5.jpg".to_string(),
            "static/images/p6.jpg".to_string(),
            "static/images/p7.jpg".to_string(),
            "static/images/p8.jpg".to_string(),
        ];
    html!{
        
        <div>

            <h1>{"TEI viewer (initial app)"}</h1>
            <ImageViewer images={images}/>
        </div>
    }
}

fn main(){
    yew::Renderer::<App>::new().render();
}
