use emberforge_chain::{Blockchain, Transaction};

fn main() {
    let mut chain = Blockchain::new(3);

    chain.add_block(vec![
        Transaction::new("alice", "bob", 10),
        Transaction::new("bob", "carol", 3),
    ]);
    chain.add_block(vec![Transaction::new("carol", "dave", 1)]);

    println!("Emberforge Chain");
    println!("================");
    println!("blocks: {}", chain.blocks.len());
    println!("valid:  {}", chain.is_valid());
    println!();

    for block in &chain.blocks {
        println!("block #{}", block.index);
        println!("  hash:      {}", block.hash);
        println!("  prev_hash: {}", block.previous_hash);
        println!("  nonce:     {}", block.nonce);
        println!("  tx count:  {}", block.transactions.len());
    }
}
