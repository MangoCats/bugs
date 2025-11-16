use crate::event::SimulationEvent;
use crate::snapshot::Snapshot;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Reads simulation events from a file
pub struct EventReader {
    event_file: BufReader<File>,
    snapshots: Vec<Snapshot>,
    current_tick: i32,
}

impl EventReader {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let base = base_path.as_ref();
        let event_path = base.with_extension("events");
        let snapshot_path = base.with_extension("snapshots");

        let event_file = BufReader::new(File::open(event_path)?);
        let snapshots = Self::load_snapshots(&snapshot_path)?;

        Ok(Self {
            event_file,
            snapshots,
            current_tick: 0,
        })
    }

    fn load_snapshots<P: AsRef<Path>>(path: P) -> Result<Vec<Snapshot>, Box<dyn std::error::Error>> {
        let mut file = BufReader::new(File::open(path)?);
        let mut snapshots = Vec::new();

        loop {
            let mut len_bytes = [0u8; 4];
            match file.read_exact(&mut len_bytes) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }

            let len = u32::from_le_bytes(len_bytes) as usize;
            let mut data = vec![0u8; len];
            file.read_exact(&mut data)?;

            let snapshot = Snapshot::from_compressed_bytes(&data)?;
            snapshots.push(snapshot);
        }

        Ok(snapshots)
    }

    /// Read the next event
    pub fn read_event(&mut self) -> Result<Option<SimulationEvent>, Box<dyn std::error::Error>> {
        let mut len_bytes = [0u8; 4];
        match self.event_file.read_exact(&mut len_bytes) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }

        let len = u32::from_le_bytes(len_bytes) as usize;
        let mut data = vec![0u8; len];
        self.event_file.read_exact(&mut data)?;

        let event: SimulationEvent = bincode::deserialize(&data)?;

        // Track current tick
        if let SimulationEvent::Tick { tick, .. } = event {
            self.current_tick = tick;
        }

        Ok(Some(event))
    }

    /// Get the nearest snapshot for a given tick
    pub fn get_nearest_snapshot(&self, tick: i32) -> Option<&Snapshot> {
        self.snapshots
            .iter()
            .rev()
            .find(|s| s.tick <= tick)
    }

    /// Get all snapshots
    pub fn snapshots(&self) -> &[Snapshot] {
        &self.snapshots
    }

    pub fn current_tick(&self) -> i32 {
        self.current_tick
    }
}
