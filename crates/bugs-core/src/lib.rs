pub mod gene;
pub mod bug;
pub mod world;
pub mod simulation;
pub mod constants;
pub mod rng;

pub use bug::Bug;
pub use gene::{Gene, GeneType};
pub use world::World;
pub use simulation::Simulation;
pub use constants::*;
