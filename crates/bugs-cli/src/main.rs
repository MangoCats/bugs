use bugs_core::simulation::{SimConfig, Simulation};
use bugs_recorder::{EventWriter, SimulationEvent};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "bugs")]
#[command(about = "Bugs - A genetic programming evolution simulator", long_about = None)]
struct Args {
    /// Random seed for the simulation
    #[arg(short, long, default_value = "42")]
    seed: u64,

    /// Maximum number of ticks to simulate
    #[arg(short, long)]
    max_ticks: Option<i32>,

    /// Output file for recording (without extension)
    #[arg(short, long, default_value = "simulation")]
    output: PathBuf,

    /// Snapshot interval (ticks between snapshots)
    #[arg(long, default_value = "1000")]
    snapshot_interval: i32,

    /// Progress update interval (ticks)
    #[arg(long, default_value = "100")]
    progress_interval: i32,

    /// Disable recording
    #[arg(long)]
    no_record: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Bugs 0.29 - Rust Edition");
    println!("========================");
    println!("Seed: {}", args.seed);
    println!("Max ticks: {}", args.max_ticks.map_or("unlimited".to_string(), |t| t.to_string()));
    println!();

    // Create simulation
    let config = SimConfig {
        seed: args.seed,
        max_ticks: args.max_ticks,
    };

    let mut sim = Simulation::new(config);

    // Create event writer if recording
    let mut writer = if !args.no_record {
        Some(EventWriter::new(&args.output, args.snapshot_interval)?)
    } else {
        None
    };

    // Progress bar
    let progress = if let Some(max) = args.max_ticks {
        ProgressBar::new(max as u64)
    } else {
        ProgressBar::new_spinner()
    };

    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) | Bugs: {msg}")?
            .progress_chars("#>-"),
    );

    // Main simulation loop
    let mut running = true;
    while running {
        running = sim.step();

        // Record tick event
        if let Some(ref mut w) = writer {
            w.write_event(&SimulationEvent::Tick {
                tick: sim.world.current_tick,
                bug_count: sim.world.bug_count(),
            })?;

            // Write snapshot periodically
            w.maybe_write_snapshot(sim.world.current_tick, &sim.world)?;
        }

        // Update progress
        if sim.world.current_tick % args.progress_interval == 0 {
            progress.set_position(sim.world.current_tick as u64);
            let stats = sim.stats();
            progress.set_message(format!(
                "{} (avg weight: {}, avg genes: {:.1})",
                stats.bug_count, stats.avg_bug_mass, stats.avg_genes
            ));
        }

        // Check termination
        if sim.world.bug_count() == 0 {
            println!("\nAll bugs died at tick {}", sim.world.current_tick);
            break;
        }
    }

    progress.finish_with_message("Simulation complete");

    // Final statistics
    let stats = sim.stats();
    println!("\nFinal Statistics:");
    println!("  Tick: {}", stats.tick);
    println!("  Bugs: {}", stats.bug_count);
    println!("  Total food: {}", stats.total_food);
    println!("  Avg bug mass: {}", stats.avg_bug_mass);
    println!("  Avg genes: {:.2}", stats.avg_genes);

    if let Some(mut w) = writer {
        // Write final snapshot
        w.write_snapshot(sim.world.current_tick, &sim.world)?;
        w.flush()?;

        println!("\nRecording saved to:");
        println!("  Events: {}", args.output.with_extension("events").display());
        println!("  Snapshots: {}", args.output.with_extension("snapshots").display());
        println!("  Total events: {}", w.events_written());
        println!("  Total bytes: {} KB", w.bytes_written() / 1024);
    }

    Ok(())
}
