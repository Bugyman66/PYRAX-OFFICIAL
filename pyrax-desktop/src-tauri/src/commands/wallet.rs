use crate::state::{AppState, WalletAddress};
use crate::rpc::RpcClient;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use pyrax_wallet::{Mnemonic, PrivateKey, signing::sign_hash};
use pyrax_node::types::{Transaction, TxInput, TxOutput, OutPoint, H256, Address};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub locked: bool,
    pub address_count: usize,
    pub total_balance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInfo {
    pub address: String,
    pub balance: String,
    pub nonce: u64,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub value: String,
    pub gas_price: String,
    pub gas_used: String,
    pub block_number: Option<u64>,
    pub timestamp: Option<u64>,
    pub status: String,
    pub tx_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendTransactionRequest {
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas_price: Option<String>,
    pub gas_limit: Option<String>,
    pub data: Option<String>,
}

#[tauri::command]
pub fn create_wallet(
    password: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    tracing::info!("Creating wallet with password length: {}", password.len());
    
    // Generate a real mnemonic using pyrax-wallet
    let mnemonic = Mnemonic::generate(24)
        .map_err(|e| format!("Failed to generate mnemonic: {}", e))?;
    
    let phrase = mnemonic.phrase().to_string();
    
    // Derive the first address from the mnemonic
    let hd_wallet = mnemonic.to_hd_wallet("");
    let first_address = hd_wallet.derive_address(0, 0)
        .map_err(|e| format!("Failed to derive address: {}", e))?;
    
    {
        let mut app_state = state.lock();
        app_state.wallet_unlocked = true;
        app_state.wallet_mnemonic = Some(phrase.clone());
        app_state.wallet_addresses = vec![WalletAddress {
            address: first_address.to_string(),
            index: 0,
            label: Some("Default".to_string()),
        }];
    }
    
    tracing::info!("Wallet created successfully with address: {}", first_address);
    Ok(phrase)
}

#[tauri::command]
pub fn unlock_wallet(
    password: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<WalletInfo, String> {
    tracing::info!("Unlocking wallet");
    let mut app_state = state.lock();
    app_state.wallet_unlocked = true;
    
    Ok(WalletInfo {
        locked: false,
        address_count: 1,
        total_balance: "0".to_string(),
    })
}

#[tauri::command]
pub fn lock_wallet(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), String> {
    let mut app_state = state.lock();
    app_state.wallet_unlocked = false;
    Ok(())
}

#[tauri::command]
pub fn get_addresses(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<AddressInfo>, String> {
    let app_state = state.lock();
    
    if !app_state.wallet_unlocked {
        return Err("Wallet is locked".to_string());
    }
    
    // Return real addresses from wallet state
    let addresses: Vec<AddressInfo> = app_state.wallet_addresses
        .iter()
        .map(|wa| AddressInfo {
            address: wa.address.clone(),
            balance: "0".to_string(), // TODO: Fetch real balance from node
            nonce: 0,
            label: wa.label.clone(),
        })
        .collect();
    
    Ok(addresses)
}

#[tauri::command]
pub fn create_address(
    label: Option<String>,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<AddressInfo, String> {
    let mut app_state = state.lock();
    
    if !app_state.wallet_unlocked {
        return Err("Wallet is locked".to_string());
    }
    
    // Get mnemonic to derive new address
    let mnemonic_phrase = app_state.wallet_mnemonic.clone()
        .ok_or_else(|| "No wallet mnemonic found".to_string())?;
    
    let mnemonic = Mnemonic::from_phrase(&mnemonic_phrase)
        .map_err(|e| format!("Invalid mnemonic: {}", e))?;
    
    // Derive next address index
    let next_index = app_state.wallet_addresses.len() as u32;
    let hd_wallet = mnemonic.to_hd_wallet("");
    let new_address = hd_wallet.derive_address(0, next_index)
        .map_err(|e| format!("Failed to derive address: {}", e))?;
    
    let wallet_addr = WalletAddress {
        address: new_address.to_string(),
        index: next_index,
        label: label.clone(),
    };
    
    app_state.wallet_addresses.push(wallet_addr);
    
    tracing::info!("Created new address at index {}: {}", next_index, new_address);
    
    Ok(AddressInfo {
        address: new_address.to_string(),
        balance: "0".to_string(),
        nonce: 0,
        label,
    })
}

#[tauri::command]
pub async fn get_balance(
    address: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let rpc_port = {
        let app_state = state.lock();
        if !app_state.node_running {
            return Err("Node is not running".to_string());
        }
        app_state.rpc_port
    };
    
    // Get UTXOs and sum balance
    let rpc = RpcClient::localhost(rpc_port);
    let utxos = rpc.get_utxos(&address).await
        .map_err(|e| format!("Failed to get UTXOs: {}", e))?;
    
    let balance: u64 = utxos.iter().map(|u| u.value).sum();
    // Convert from base units to PYRAX (8 decimals)
    let pyrax = balance as f64 / 100_000_000.0;
    Ok(format!("{:.8}", pyrax))
}

#[tauri::command]
pub async fn send_transaction(
    request: SendTransactionRequest,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let (rpc_port, mnemonic_phrase, from_index) = {
        let app_state = state.lock();
        
        if !app_state.wallet_unlocked {
            return Err("Wallet is locked".to_string());
        }
        
        if !app_state.node_running {
            return Err("Node is not running".to_string());
        }
        
        let mnemonic = app_state.wallet_mnemonic.clone()
            .ok_or_else(|| "No wallet mnemonic".to_string())?;
        
        // Find the index of the from address
        let from_idx = app_state.wallet_addresses.iter()
            .find(|a| a.address.to_lowercase() == request.from.to_lowercase())
            .map(|a| a.index)
            .ok_or_else(|| "From address not found in wallet".to_string())?;
        
        (app_state.rpc_port, mnemonic, from_idx)
    };
    
    // Parse amount (PYRAX to base units)
    let amount_pyrax: f64 = request.value.parse()
        .map_err(|_| "Invalid amount".to_string())?;
    let amount_base = (amount_pyrax * 100_000_000.0) as u64;
    
    // Get private key for signing
    let mnemonic = Mnemonic::from_phrase(&mnemonic_phrase)
        .map_err(|e| format!("Invalid mnemonic: {}", e))?;
    let hd_wallet = mnemonic.to_hd_wallet("");
    let private_key = hd_wallet.derive_path(&pyrax_wallet::DerivationPath::pyrax(0, 0, from_index))
        .map_err(|e| format!("Failed to derive key: {}", e))?;
    
    // Get UTXOs for the from address
    let rpc = RpcClient::localhost(rpc_port);
    let utxos = rpc.get_utxos(&request.from).await
        .map_err(|e| format!("Failed to get UTXOs: {}", e))?;
    
    if utxos.is_empty() {
        return Err("No UTXOs available to spend".to_string());
    }
    
    // Select UTXOs to cover amount + fee
    let fee = 1000u64; // Minimum fee in base units
    let required = amount_base + fee;
    
    let mut selected_utxos = Vec::new();
    let mut total_input = 0u64;
    
    for utxo in &utxos {
        selected_utxos.push(utxo.clone());
        total_input += utxo.value;
        if total_input >= required {
            break;
        }
    }
    
    if total_input < required {
        return Err(format!("Insufficient balance. Have {} but need {}", total_input, required));
    }
    
    // Build transaction using the node's format
    let tx_hex = build_and_sign_transaction(
        &private_key,
        &selected_utxos,
        &request.to,
        amount_base,
        &request.from,
        total_input - amount_base - fee, // change
    ).map_err(|e| format!("Failed to build transaction: {}", e))?;
    
    // Broadcast transaction
    let txid = rpc.send_raw_transaction(&tx_hex).await
        .map_err(|e| format!("Failed to broadcast: {}", e))?;
    
    tracing::info!("Transaction broadcast: {}", txid);
    Ok(txid)
}

/// Build and sign a UTXO transaction using pyrax-node types
fn build_and_sign_transaction(
    private_key: &PrivateKey,
    utxos: &[crate::rpc::UtxoResponse],
    to_address: &str,
    amount: u64,
    change_address: &str,
    change_amount: u64,
) -> Result<String, String> {
    // Parse addresses
    let to_bytes = ::hex::decode(to_address.trim_start_matches("0x"))
        .map_err(|_| "Invalid to address")?;
    let change_bytes = ::hex::decode(change_address.trim_start_matches("0x"))
        .map_err(|_| "Invalid change address")?;
    
    if to_bytes.len() != 20 || change_bytes.len() != 20 {
        return Err("Address must be 20 bytes".to_string());
    }
    
    let to_addr = Address::from_slice(&to_bytes);
    let change_addr = Address::from_slice(&change_bytes);
    
    // Build inputs (without signatures first for hashing)
    let mut inputs: Vec<TxInput> = Vec::new();
    for utxo in utxos {
        let txid_bytes = ::hex::decode(utxo.txid.trim_start_matches("0x"))
            .map_err(|_| "Invalid utxo txid")?;
        let txid = H256::from_slice(&txid_bytes);
        let outpoint = OutPoint::new(txid, utxo.vout);
        inputs.push(TxInput::new(outpoint, vec![])); // Empty script_sig for signing
    }
    
    // Build outputs
    let mut outputs = vec![TxOutput::p2pkh(amount, &to_addr)];
    if change_amount > 0 {
        outputs.push(TxOutput::p2pkh(change_amount, &change_addr));
    }
    
    // Create unsigned transaction for hashing
    let unsigned_tx = Transaction::new(inputs.clone(), outputs.clone());
    let tx_bytes = bincode::serialize(&unsigned_tx)
        .map_err(|e| format!("Failed to serialize tx: {}", e))?;
    
    // Hash for signing (BLAKE3)
    let tx_hash = blake3::hash(&tx_bytes);
    let hash_bytes: [u8; 32] = *tx_hash.as_bytes();
    
    // Sign
    let signature = sign_hash(private_key, &hash_bytes)
        .map_err(|e| format!("Signing failed: {}", e))?;
    
    // Build script_sig: <sig_len> <sig> <pubkey_len> <pubkey>
    let pubkey = private_key.public_key();
    let pubkey_bytes = pubkey.to_bytes_compressed();
    let sig_bytes = signature.to_bytes();
    
    let mut script_sig = Vec::new();
    script_sig.push(sig_bytes.len() as u8);
    script_sig.extend_from_slice(&sig_bytes);
    script_sig.push(pubkey_bytes.len() as u8);
    script_sig.extend_from_slice(pubkey_bytes);
    
    // Update inputs with signatures
    let signed_inputs: Vec<TxInput> = utxos.iter().map(|utxo| {
        let txid_bytes = ::hex::decode(utxo.txid.trim_start_matches("0x")).unwrap();
        let txid = H256::from_slice(&txid_bytes);
        let outpoint = OutPoint::new(txid, utxo.vout);
        TxInput::new(outpoint, script_sig.clone())
    }).collect();
    
    // Create signed transaction
    let signed_tx = Transaction::new(signed_inputs, outputs);
    let signed_bytes = bincode::serialize(&signed_tx)
        .map_err(|e| format!("Failed to serialize signed tx: {}", e))?;
    
    Ok(format!("0x{}", ::hex::encode(&signed_bytes)))
}

#[tauri::command]
pub fn get_transactions(
    address: Option<String>,
    limit: Option<u32>,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<TransactionInfo>, String> {
    let app_state = state.lock();
    
    if !app_state.node_running {
        return Err("Node is not running".to_string());
    }
    
    // TODO: Get real transactions
    Ok(vec![])
}

#[tauri::command]
pub fn import_mnemonic(
    mnemonic: String,
    password: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<WalletInfo, String> {
    tracing::info!("Importing mnemonic");
    let mut app_state = state.lock();
    app_state.wallet_unlocked = true;
    
    Ok(WalletInfo {
        locked: false,
        address_count: 1,
        total_balance: "0".to_string(),
    })
}

#[tauri::command]
pub fn export_mnemonic(
    password: String,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<String, String> {
    let app_state = state.lock();
    
    if !app_state.wallet_unlocked {
        return Err("Wallet is locked".to_string());
    }
    
    // TODO: Actually export mnemonic (requires password verification)
    Err("Not implemented".to_string())
}
