use serde::{Deserialize, Serialize};
use crate::types::{Block, BlockHeader, SignedTransaction, H256, BlockNumber};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageType {
    // Handshake
    Hello = 0,
    HelloAck = 1,
    Disconnect = 2,
    
    // Block sync
    GetHeaders = 10,
    Headers = 11,
    GetBlocks = 12,
    BlockData = 13,
    NewBlock = 14,
    NewBlockHashes = 15,
    
    // Transactions
    InvTx = 20,
    GetTx = 21,
    Tx = 22,
    TxPool = 23,
    
    // Status
    Ping = 30,
    Pong = 31,
    GetStatus = 32,
    Status = 33,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // Handshake
    Hello(HelloMessage),
    HelloAck(HelloAckMessage),
    Disconnect(DisconnectMessage),
    
    // Block sync
    GetHeaders(GetHeadersMessage),
    Headers(HeadersMessage),
    GetBlocks(GetBlocksMessage),
    BlockData(BlockDataMessage),
    NewBlock(NewBlockMessage),
    NewBlockHashes(NewBlockHashesMessage),
    
    // Transactions
    InvTx(InvTxMessage),
    GetTx(GetTxMessage),
    Tx(TxMessage),
    TxPool(TxPoolMessage),
    
    // Status
    Ping(PingMessage),
    Pong(PongMessage),
    GetStatus,
    Status(StatusMessage),
}

impl Message {
    pub fn message_type(&self) -> MessageType {
        match self {
            Message::Hello(_) => MessageType::Hello,
            Message::HelloAck(_) => MessageType::HelloAck,
            Message::Disconnect(_) => MessageType::Disconnect,
            Message::GetHeaders(_) => MessageType::GetHeaders,
            Message::Headers(_) => MessageType::Headers,
            Message::GetBlocks(_) => MessageType::GetBlocks,
            Message::BlockData(_) => MessageType::BlockData,
            Message::NewBlock(_) => MessageType::NewBlock,
            Message::NewBlockHashes(_) => MessageType::NewBlockHashes,
            Message::InvTx(_) => MessageType::InvTx,
            Message::GetTx(_) => MessageType::GetTx,
            Message::Tx(_) => MessageType::Tx,
            Message::TxPool(_) => MessageType::TxPool,
            Message::Ping(_) => MessageType::Ping,
            Message::Pong(_) => MessageType::Pong,
            Message::GetStatus => MessageType::GetStatus,
            Message::Status(_) => MessageType::Status,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }

    pub fn decode(data: &[u8]) -> Option<Self> {
        bincode::deserialize(data).ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloMessage {
    pub protocol_version: u32,
    pub network_id: u32,
    pub genesis_hash: H256,
    pub best_height: BlockNumber,
    pub best_hash: H256,
    pub node_id: [u8; 32],
    pub listen_port: u16,
    pub client_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloAckMessage {
    pub protocol_version: u32,
    pub network_id: u32,
    pub genesis_hash: H256,
    pub best_height: BlockNumber,
    pub best_hash: H256,
    pub node_id: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisconnectMessage {
    pub reason: DisconnectReason,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisconnectReason {
    Requested,
    TooManyPeers,
    IncompatibleProtocol,
    InvalidGenesis,
    Timeout,
    BadBehavior,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHeadersMessage {
    pub start_hash: H256,
    pub start_height: BlockNumber,
    pub max_headers: u32,
    pub reverse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadersMessage {
    pub headers: Vec<BlockHeader>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBlocksMessage {
    pub hashes: Vec<H256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDataMessage {
    pub block: Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBlockMessage {
    pub block: Block,
    pub total_difficulty: ethereum_types::U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBlockHashesMessage {
    pub hashes: Vec<(H256, BlockNumber)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvTxMessage {
    pub hashes: Vec<H256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTxMessage {
    pub hashes: Vec<H256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxMessage {
    pub transactions: Vec<SignedTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxPoolMessage {
    pub hashes: Vec<H256>,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingMessage {
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongMessage {
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusMessage {
    pub best_height: BlockNumber,
    pub best_hash: H256,
    pub total_difficulty: ethereum_types::U256,
    pub genesis_hash: H256,
}

pub const MAX_HEADERS_PER_REQUEST: u32 = 2000;
pub const MAX_BLOCKS_PER_REQUEST: usize = 128;
pub const MAX_TXS_PER_MESSAGE: usize = 256;
pub const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024; // 16 MB
