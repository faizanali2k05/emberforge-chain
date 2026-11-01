use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

pub const SYSTEM_ADDRESS: &str = "SYSTEM";

#[derive(Debug)]
pub enum ChainError {
    InvalidTransaction(String),
    InvalidChain,
    Io(std::io::Error),
    Serialization(serde_json::Error),
}
impl fmt::Display for ChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction(reason) => write!(f, "invalid transaction: {reason}"),
            Self::InvalidChain => write!(f, "invalid blockchain"),
            Self::Io(error) => write!(f, "storage error: {error}"),
            Self::Serialization(error) => write!(f, "serialization error: {error}"),
        }
    }
}
impl std::error::Error for ChainError {}
impl From<std::io::Error> for ChainError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
impl From<serde_json::Error> for ChainError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
    pub nonce: u64,
    pub signature: Option<String>,
}
impl Transaction {
    pub fn new(
        sender: impl Into<String>,
        recipient: impl Into<String>,
        amount: u64,
        nonce: u64,
    ) -> Self {
        Self {
            sender: sender.into(),
            recipient: recipient.into(),
            amount,
            nonce,
            signature: None,
        }
    }
    pub fn signing_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&(&self.sender, &self.recipient, self.amount, self.nonce)).unwrap()
    }
    pub fn sign(mut self, wallet: &Wallet) -> Self {
        self.signature = Some(hex::encode(wallet.sign(&self.signing_bytes()).to_bytes()));
        self
    }
    pub fn is_signed_by_sender(&self) -> bool {
        if self.sender == SYSTEM_ADDRESS {
            return self.signature.is_none();
        }
        let Ok(public_bytes) = hex::decode(&self.sender) else {
            return false;
        };
        let Some(signature_text) = &self.signature else {
            return false;
        };
        let Ok(public_array) = <[u8; 32]>::try_from(public_bytes.as_slice()) else {
            return false;
        };
        let Ok(public_key) = VerifyingKey::from_bytes(&public_array) else {
            return false;
        };
        let Ok(signature_bytes) = hex::decode(signature_text) else {
            return false;
        };
        let Ok(signature) = Signature::from_slice(&signature_bytes) else {
            return false;
        };
        public_key.verify(&self.signing_bytes(), &signature).is_ok()
    }
}

#[derive(Debug)]
pub struct Wallet {
    signing_key: SigningKey,
}
impl Wallet {
    pub fn generate() -> Self {
        Self {
            signing_key: SigningKey::generate(&mut OsRng),
        }
    }
    pub fn address(&self) -> String {
        hex::encode(self.signing_key.verifying_key().to_bytes())
    }
    fn sign(&self, bytes: &[u8]) -> Signature {
        self.signing_key.sign(bytes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub index: u64,
    pub timestamp: i64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub nonce: u64,
    pub hash: String,
}
impl Block {
    fn mine(
        index: u64,
        transactions: Vec<Transaction>,
        previous_hash: &str,
        difficulty: usize,
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        let target = "0".repeat(difficulty);
        let mut nonce = 0;
        loop {
            let hash = calculate_hash(index, timestamp, &transactions, previous_hash, nonce);
            if hash.starts_with(&target) {
                return Self {
                    index,
                    timestamp,
                    transactions,
                    previous_hash: previous_hash.into(),
                    nonce,
                    hash,
                };
            }
            nonce += 1;
        }
    }
    fn has_valid_hash(&self, difficulty: usize) -> bool {
        self.hash
            == calculate_hash(
                self.index,
                self.timestamp,
                &self.transactions,
                &self.previous_hash,
                self.nonce,
            )
            && self.hash.starts_with(&"0".repeat(difficulty))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub difficulty: usize,
    pub mining_reward: u64,
    pub genesis_balances: HashMap<String, u64>,
    pub pending_transactions: Vec<Transaction>,
}
impl Blockchain {
    pub fn new(difficulty: usize, genesis_balances: HashMap<String, u64>) -> Self {
        Self {
            blocks: vec![Block::mine(0, vec![], "0", difficulty)],
            difficulty,
            mining_reward: 50,
            genesis_balances,
            pending_transactions: vec![],
        }
    }
    pub fn balance_of(&self, address: &str) -> u64 {
        self.balances().get(address).copied().unwrap_or(0)
    }
    pub fn submit_transaction(&mut self, transaction: Transaction) -> Result<(), ChainError> {
        if transaction.amount == 0 || transaction.sender == transaction.recipient {
            return Err(ChainError::InvalidTransaction(
                "amount must be positive and addresses must differ".into(),
            ));
        }
        if !transaction.is_signed_by_sender() {
            return Err(ChainError::InvalidTransaction(
                "signature does not match sender".into(),
            ));
        }
        let mut balances = self.balances();
        for pending in &self.pending_transactions {
            apply_transaction(&mut balances, pending)?;
        }
        apply_transaction(&mut balances, &transaction)?;
        self.pending_transactions.push(transaction);
        Ok(())
    }
    pub fn mine_pending(&mut self, miner: &str) -> Result<(), ChainError> {
        let mut transactions = std::mem::take(&mut self.pending_transactions);
        transactions.push(Transaction::new(
            SYSTEM_ADDRESS,
            miner,
            self.mining_reward,
            0,
        ));
        let previous = self.blocks.last().ok_or(ChainError::InvalidChain)?.clone();
        self.blocks.push(Block::mine(
            previous.index + 1,
            transactions,
            &previous.hash,
            self.difficulty,
        ));
        if self.is_valid() {
            Ok(())
        } else {
            Err(ChainError::InvalidChain)
        }
    }
    pub fn balances(&self) -> HashMap<String, u64> {
        let mut balances = self.genesis_balances.clone();
        for block in &self.blocks {
            for transaction in &block.transactions {
                if apply_transaction(&mut balances, transaction).is_err() {
                    return HashMap::new();
                }
            }
        }
        balances
    }
    pub fn is_valid(&self) -> bool {
        if self.blocks.is_empty() {
            return false;
        }
        let mut balances = self.genesis_balances.clone();
        for (position, block) in self.blocks.iter().enumerate() {
            if !block.has_valid_hash(self.difficulty) || block.index as usize != position {
                return false;
            }
            if position == 0 {
                if block.previous_hash != "0" || !block.transactions.is_empty() {
                    return false;
                }
            } else if block.previous_hash != self.blocks[position - 1].hash {
                return false;
            }
            for transaction in &block.transactions {
                if apply_transaction(&mut balances, transaction).is_err() {
                    return false;
                }
            }
        }
        true
    }
    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), ChainError> {
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, ChainError> {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    }
}

fn apply_transaction(
    balances: &mut HashMap<String, u64>,
    transaction: &Transaction,
) -> Result<(), ChainError> {
    if transaction.amount == 0 || !transaction.is_signed_by_sender() {
        return Err(ChainError::InvalidTransaction("invalid transaction".into()));
    }
    if transaction.sender != SYSTEM_ADDRESS {
        let sender_balance = balances
            .get_mut(&transaction.sender)
            .ok_or_else(|| ChainError::InvalidTransaction("sender has no balance".into()))?;
        if *sender_balance < transaction.amount {
            return Err(ChainError::InvalidTransaction(
                "insufficient balance".into(),
            ));
        }
        *sender_balance -= transaction.amount;
    }
    *balances.entry(transaction.recipient.clone()).or_default() += transaction.amount;
    Ok(())
}
fn calculate_hash(
    index: u64,
    timestamp: i64,
    transactions: &[Transaction],
    previous_hash: &str,
    nonce: u64,
) -> String {
    let payload =
        serde_json::to_vec(&(index, timestamp, transactions, previous_hash, nonce)).unwrap();
    hex::encode(Sha256::digest(payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (Blockchain, Wallet, Wallet) {
        let alice = Wallet::generate();
        let bob = Wallet::generate();
        let mut balances = HashMap::new();
        balances.insert(alice.address(), 100);
        (Blockchain::new(1, balances), alice, bob)
    }
    #[test]
    fn signed_transactions_update_balances() {
        let (mut chain, alice, bob) = setup();
        chain
            .submit_transaction(
                Transaction::new(alice.address(), bob.address(), 25, 0).sign(&alice),
            )
            .unwrap();
        chain.mine_pending(&alice.address()).unwrap();
        assert_eq!(chain.balance_of(&bob.address()), 25);
        assert!(chain.is_valid());
    }
    #[test]
    fn rejects_forged_transactions() {
        let (mut chain, alice, bob) = setup();
        let attacker = Wallet::generate();
        let forged = Transaction::new(alice.address(), bob.address(), 25, 0).sign(&attacker);
        assert!(chain.submit_transaction(forged).is_err());
    }
    #[test]
    fn detects_tampering() {
        let (mut chain, alice, bob) = setup();
        chain
            .submit_transaction(
                Transaction::new(alice.address(), bob.address(), 25, 0).sign(&alice),
            )
            .unwrap();
        chain.mine_pending(&alice.address()).unwrap();
        chain.blocks[1].transactions[0].amount = 90;
        assert!(!chain.is_valid());
    }
    #[test]
    fn persists_and_reloads() {
        let (chain, _, _) = setup();
        let path = std::env::temp_dir().join("emberforge-chain-test.json");
        chain.save_json(&path).unwrap();
        let loaded = Blockchain::load_json(&path).unwrap();
        assert!(loaded.is_valid());
        let _ = std::fs::remove_file(path);
    }
}
