use gloo_worker::Registrable;
use wasm_bindgen::prelude::wasm_bindgen;
use string_art_gui_web_worker::WebWorker;

#[wasm_bindgen(start)] 
async fn start_worker() {
    WebWorker::registrar().register();
}