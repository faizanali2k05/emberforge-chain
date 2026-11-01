use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A deliberately small transaction model for the first milestone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: u64,
}

impl Transaction {
    pub fn new(sender: impl Into<String>, recipient: impl Into<String>, amount: u64) -> Self {
        Self {
            sender: sender.into(),
            recipient: recipient.into(),
            amount,
        }
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
    pub fn genesis() -> Self {
        Self::mine(0, vec![], "0", 2)
    }

    pub fn mine(
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
                    previous_hash: previous_hash.to_owned(),
                    nonce,
                    hash,
                };
            }
            nonce += 1;
        }
    }

    pub fn has_valid_hash(&self, difficulty: usize) -> bool {
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
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        Self {
            blocks: vec![Block::mine(0, vec![], "0", difficulty)],
            difficulty,
        }
    }

    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let previous = self.blocks.last().expect("chain always has genesis");
        let block = Block::mine(
            previous.index + 1,
            transactions,
            &previous.hash,
            self.difficulty,
        );
        self.blocks.push(block);
    }

    pub fn is_valid(&self) -> bool {
        if self.blocks.is_empty() {
            return false;
        }

        for (position, block) in self.blocks.iter().enumerate() {
            if !block.has_valid_hash(self.difficulty) {
                return false;
            }
            if position == 0 {
                if block.index != 0 || block.previous_hash != "0" {
                    return false;
                }
            } else {
                let previous = &self.blocks[position - 1];
                if block.index != previous.index + 1 || block.previous_hash != previous.hash {
                    return false;
                }
            }
        }
        true
    }
}

fn calculate_hash(
    index: u64,
    timestamp: i64,
    transactions: &[Transaction],
    previous_hash: &str,
    nonce: u64,
) -> String {
    let payload = serde_json::to_vec(&(index, timestamp, transactions, previous_hash, nonce))
        .expect("block fields are serializable");
    hex::encode(Sha256::digest(payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_a_valid_chain() {
        let mut chain = Blockchain::new(2);
        chain.add_block(vec![Transaction::new("alice", "bob", 10)]);
        assert_eq!(chain.blocks.len(), 2);
        assert!(chain.is_valid());
    }

    #[test]
    fn detects_tampering() {
        let mut chain = Blockchain::new(2);
        chain.add_block(vec![Transaction::new("alice", "bob", 10)]);
        chain.blocks[1].transactions[0].amount = 1_000_000;
        assert!(!chain.is_valid());
    }

    #[test]
    fn detects_broken_link() {
        let mut chain = Blockchain::new(2);
        chain.add_block(vec![Transaction::new("alice", "bob", 10)]);
        chain.blocks[1].previous_hash = "fake-parent".into();
        assert!(!chain.is_valid());
    }
}
