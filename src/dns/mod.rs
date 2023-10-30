pub mod header;
pub mod message;

pub trait Serialize {
    fn serialize(&self) -> Result<Vec<u8>, anyhow::Error>;
}

pub trait DeSerialize {
    type Item;
    fn deserialize(raw: &[u8]) -> Result<Self::Item, anyhow::Error>;
}
