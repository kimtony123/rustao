use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct TransactionData {
    pub format: usize,
    pub id: String,
    pub last_tx: String,
    pub owner: String,
    pub tags: Vec<super::schema::Tag>,
    pub target: String,
    pub quantity: String,
    pub data: Vec<u8>,
    pub reward: String,
    pub signature: String,
    pub data_size: String,
    pub data_root: String,
}
