#[allow(dead_code)]
mod constants;
mod hex;
mod render;
mod rng;
mod simulation;
mod types;
mod web;

use std::sync::Arc;

use parking_lot::RwLock;

use render::Renderer;
use simulation::Simulation;
use web::{FrameStore, SharedFrameStore, create_router};

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main());
}

async fn async_main() {
    let store: SharedFrameStore = Arc::new(RwLock::new(FrameStore::new()));
    let renderer = Renderer::new();

    // Start web server
    let web_store = store.clone();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3028").await.unwrap();
    println!("Web server listening on http://localhost:3028");

    tokio::spawn(async move {
        let app = create_router(web_store);
        axum::serve(listener, app).await.unwrap();
    });

    // Run simulation in a blocking task
    let sim_store = store.clone();
    tokio::task::spawn_blocking(move || {
        run_simulation(sim_store, renderer);
    })
    .await
    .unwrap();
}

fn run_simulation(store: SharedFrameStore, renderer: Renderer) {
    let mut sim = Simulation::new();
    let interval: i64 = 4;
    let mut lastbugcount: i64 = 0;

    loop {
        let alive = sim.step();
        if !alive {
            break;
        }

        // Console output matching C
        if sim.today % 100 == 0 || (lastbugcount < 100 && sim.n_bugs != lastbugcount) {
            let avg_food = sim.totalfood as f64 / (constants::WORLD_X * constants::WORLD_Y) as f64;
            let avg_bug = sim.totalbug as f64 / (sim.n_bugs as f64 * 1024.0);
            let avg_genes = sim.genecount as f64 / sim.n_bugs as f64;
            let pct = (sim.n_bugs as f64 * 100.0) / (constants::WORLD_X * constants::WORLD_Y) as f64;
            println!(
                "{:6}Dy {:5}Bg {:4.1}% {:10} {:10} F={:5.0} B={:5.0} Gns={:6.2} AD{:4} FH{:6.3} FM{:02x}",
                sim.today,
                sim.n_bugs,
                pct,
                sim.bug_order.first().and_then(|&i| sim.bugs[i].as_ref()).map(|b| b.brain.eth.uid).unwrap_or(0),
                sim.idcounter,
                avg_food,
                avg_bug,
                avg_genes,
                sim.agediv,
                sim.foodhump,
                sim.forcemate,
            );
            lastbugcount = sim.n_bugs;
        }

        // Generate frames at interval
        if sim.should_output_frame(interval) {
            let bug_pixels = renderer.render_bug_frame(&sim);
            let env_pixels = renderer.render_env_frame(&sim);
            let bug_jpeg = renderer.encode_jpeg(&bug_pixels);
            let env_jpeg = renderer.encode_jpeg(&env_pixels);

            let mut s = store.write();
            s.bug_frame = bug_jpeg;
            s.env_frame = env_jpeg;
            s.today = sim.today;
            s.n_bugs = sim.n_bugs;
        }
    }
}
