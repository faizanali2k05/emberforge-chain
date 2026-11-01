use emberforge_chain::{Blockchain, Transaction, Wallet};
use std::collections::HashMap;

fn main() {
    let alice = Wallet::generate();
    let bob = Wallet::generate();
    let mut genesis = HashMap::new();
    genesis.insert(alice.address(), 100);
    let mut chain = Blockchain::new(2, genesis);

    let payment = Transaction::new(alice.address(), bob.address(), 25, 0).sign(&alice);
    chain
        .submit_transaction(payment)
        .expect("valid signed transaction");
    chain.mine_pending(&alice.address()).expect("valid block");

    println!("Emberforge Chain");
    println!("================");
    println!(
        "blocks: {} | valid: {}",
        chain.blocks.len(),
        chain.is_valid()
    );
    println!("alice:  {}", chain.balance_of(&alice.address()));
    println!("bob:    {}", chain.balance_of(&bob.address()));
    println!("tip:    {}", chain.blocks.last().unwrap().hash);
}
