//! Test transaction creation and submission
//! 
//! Creates a signed transaction spending a UTXO and submits it via RPC

use pyrax_node::types::{Transaction, TxInput, TxOutput, OutPoint, H256, Address};

fn main() {
    // Source UTXO (from miner address) - real UTXO from node
    let source_txid = "0090e2b2fd3eed8dcd8984bd2b1ac726ec712ef9e435d4054c5f9fbd70fec7af";
    let source_vout = 0u32;
    let source_value = 5_000_000_000u64; // 50 PYRAX
    
    // Miner address (source)
    let from_bytes = hex::decode("7a3b9c4e5f6d8a2b1c0e9f8d7a6b5c4d3e2f1a0b").unwrap();
    let from_addr = Address::from_slice(&from_bytes);
    
    // Destination address (different test address)
    let to_bytes = hex::decode("1111111111111111111111111111111111111111").unwrap();
    let to_addr = Address::from_slice(&to_bytes);
    
    // Amount to send: 10 PYRAX
    let send_amount = 1_000_000_000u64;
    let fee = 1000u64;
    let change_amount = source_value - send_amount - fee;
    
    // Create outpoint for the UTXO we're spending
    let txid_bytes = hex::decode(source_txid.trim_start_matches("0x")).unwrap();
    let txid = H256::from_slice(&txid_bytes);
    let outpoint = OutPoint::new(txid, source_vout);
    
    // Create unsigned transaction first (for signing)
    let input = TxInput::new(outpoint, vec![]); // Empty script_sig for now
    let outputs = vec![
        TxOutput::p2pkh(send_amount, &to_addr),
        TxOutput::p2pkh(change_amount, &from_addr),
    ];
    
    let unsigned_tx = Transaction::new(vec![input], outputs.clone());
    
    // Serialize for hashing
    let tx_bytes = bincode::serialize(&unsigned_tx).unwrap();
    let tx_hash = blake3::hash(&tx_bytes);
    
    println!("Transaction hash for signing: 0x{}", hex::encode(tx_hash.as_bytes()));
    println!("Unsigned tx hex: 0x{}", hex::encode(&tx_bytes));
    
    // For a real test, we'd need the private key to sign
    // For now, create a dummy signature to test the RPC validation
    let dummy_sig = vec![0u8; 64]; // 64-byte dummy signature
    let dummy_pubkey = vec![0u8; 33]; // 33-byte compressed pubkey
    
    let mut script_sig = Vec::new();
    script_sig.push(dummy_sig.len() as u8);
    script_sig.extend_from_slice(&dummy_sig);
    script_sig.push(dummy_pubkey.len() as u8);
    script_sig.extend_from_slice(&dummy_pubkey);
    
    // Create signed transaction
    let signed_input = TxInput::new(outpoint, script_sig);
    let signed_tx = Transaction::new(vec![signed_input], outputs);
    
    let signed_bytes = bincode::serialize(&signed_tx).unwrap();
    println!("\nSigned tx hex (for RPC test): 0x{}", hex::encode(&signed_bytes));
    println!("\nTransaction ID: {}", signed_tx.txid());
}
