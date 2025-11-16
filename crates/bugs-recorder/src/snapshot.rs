use bugs_core::world::World;
use serde::{Deserialize, Serialize};

/// Full world state snapshot for fast seeking
#[derive(Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: i32,
    pub world: World,
    pub file_offset: u64,  // Where in the event stream this snapshot was taken
}

impl Snapshot {
    pub fn new(tick: i32, world: World, file_offset: u64) -> Self {
        Self {
            tick,
            world,
            file_offset,
        }
    }

    /// Serialize to compressed bytes
    pub fn to_compressed_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let encoded = bincode::serialize(self)?;
        let mut compressed = Vec::new();
        let mut encoder = lz4::EncoderBuilder::new()
            .level(4)
            .build(&mut compressed)?;

        std::io::copy(&mut &encoded[..], &mut encoder)?;
        let (_output, result) = encoder.finish();
        result?;

        Ok(compressed)
    }

    /// Deserialize from compressed bytes
    pub fn from_compressed_bytes(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut decoder = lz4::Decoder::new(data)?;
        let mut decompressed = Vec::new();
        std::io::copy(&mut decoder, &mut decompressed)?;

        Ok(bincode::deserialize(&decompressed)?)
    }
}
