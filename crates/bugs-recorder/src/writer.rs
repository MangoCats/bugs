use crate::event::SimulationEvent;
use crate::snapshot::Snapshot;
use bugs_core::world::World;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Records simulation events to a file
pub struct EventWriter {
    event_file: BufWriter<File>,
    snapshot_file: BufWriter<File>,
    events_written: usize,
    bytes_written: u64,
    last_snapshot_tick: i32,
    snapshot_interval: i32,
}

impl EventWriter {
    pub fn new<P: AsRef<Path>>(base_path: P, snapshot_interval: i32) -> std::io::Result<Self> {
        let base = base_path.as_ref();
        let event_path = base.with_extension("events");
        let snapshot_path = base.with_extension("snapshots");

        Ok(Self {
            event_file: BufWriter::new(File::create(event_path)?),
            snapshot_file: BufWriter::new(File::create(snapshot_path)?),
            events_written: 0,
            bytes_written: 0,
            last_snapshot_tick: -snapshot_interval,
            snapshot_interval,
        })
    }

    /// Write an event to the log
    pub fn write_event(&mut self, event: &SimulationEvent) -> Result<(), Box<dyn std::error::Error>> {
        let encoded = bincode::serialize(event)?;
        let len = encoded.len() as u32;

        // Write length prefix
        self.event_file.write_all(&len.to_le_bytes())?;
        // Write event data
        self.event_file.write_all(&encoded)?;

        self.events_written += 1;
        self.bytes_written += (4 + len) as u64;

        Ok(())
    }

    /// Write a snapshot if interval has elapsed
    pub fn maybe_write_snapshot(&mut self, tick: i32, world: &World) -> Result<(), Box<dyn std::error::Error>> {
        if tick - self.last_snapshot_tick >= self.snapshot_interval {
            self.write_snapshot(tick, world)?;
            self.last_snapshot_tick = tick;
        }
        Ok(())
    }

    /// Force write a snapshot
    pub fn write_snapshot(&mut self, tick: i32, world: &World) -> Result<(), Box<dyn std::error::Error>> {
        let snapshot = Snapshot::new(tick, world.clone(), self.bytes_written);
        let compressed = snapshot.to_compressed_bytes()?;
        let len = compressed.len() as u32;

        // Write length prefix
        self.snapshot_file.write_all(&len.to_le_bytes())?;
        // Write snapshot data
        self.snapshot_file.write_all(&compressed)?;

        Ok(())
    }

    /// Flush all buffers
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.event_file.flush()?;
        self.snapshot_file.flush()?;
        Ok(())
    }

    pub fn events_written(&self) -> usize {
        self.events_written
    }

    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
}

impl Drop for EventWriter {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}
