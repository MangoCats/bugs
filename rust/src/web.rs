use std::sync::Arc;

use axum::{
    Router,
    extract::State,
    http::header,
    response::{Html, IntoResponse},
    routing::get,
};
use parking_lot::RwLock;
use tower_http::cors::CorsLayer;

pub struct FrameStore {
    pub bug_frame: Vec<u8>,   // Latest bug map JPEG
    pub env_frame: Vec<u8>,   // Latest environment map JPEG
    pub today: i64,
    pub n_bugs: i64,
}

impl FrameStore {
    pub fn new() -> Self {
        Self {
            bug_frame: Vec::new(),
            env_frame: Vec::new(),
            today: 0,
            n_bugs: 0,
        }
    }
}

pub type SharedFrameStore = Arc<RwLock<FrameStore>>;

async fn index(State(store): State<SharedFrameStore>) -> Html<String> {
    let s = store.read();
    Html(format!(r#"<!DOCTYPE html>
<html>
<head><title>Bugs 0.28 - Day {} ({} bugs)</title>
<style>
  body {{ background: #000; color: #ccc; font-family: monospace; margin: 0; padding: 10px; }}
  img {{ max-width: 100%; height: auto; margin: 5px 0; }}
  h1 {{ color: #fff; }}
  .info {{ color: #0f0; }}
</style>
</head>
<body>
<h1>Bugs 0.28 Simulation</h1>
<p class="info">Day: {} | Bugs: {} | Year: {:.2}</p>
<h2>Bug Map</h2>
<img src="/frame/bugs" id="bugs">
<h2>Environment</h2>
<img src="/frame/env" id="env">
<script>
  setInterval(() => {{
    document.getElementById('bugs').src = '/frame/bugs?' + Date.now();
    document.getElementById('env').src = '/frame/env?' + Date.now();
    fetch('/status').then(r=>r.json()).then(d=>{{
      document.querySelector('.info').textContent =
        'Day: ' + d.today + ' | Bugs: ' + d.n_bugs + ' | Year: ' + d.year;
    }});
  }}, 1000);
</script>
</body>
</html>"#, s.today, s.n_bugs, s.today, s.n_bugs, s.today as f64 / 16384.0))
}

async fn bug_frame(State(store): State<SharedFrameStore>) -> impl IntoResponse {
    let jpeg = store.read().bug_frame.clone();
    ([(header::CONTENT_TYPE, "image/jpeg")], jpeg)
}

async fn env_frame(State(store): State<SharedFrameStore>) -> impl IntoResponse {
    let jpeg = store.read().env_frame.clone();
    ([(header::CONTENT_TYPE, "image/jpeg")], jpeg)
}

async fn status(State(store): State<SharedFrameStore>) -> impl IntoResponse {
    let s = store.read();
    let json = format!(r#"{{"today":{},"n_bugs":{},"year":{:.4}}}"#,
        s.today, s.n_bugs, s.today as f64 / 16384.0);
    ([(header::CONTENT_TYPE, "application/json")], json)
}

pub fn create_router(store: SharedFrameStore) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/frame/bugs", get(bug_frame))
        .route("/frame/env", get(env_frame))
        .route("/status", get(status))
        .layer(CorsLayer::permissive())
        .with_state(store)
}
