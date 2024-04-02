use std::collections::{HashMap, HashSet};

use anyhow::{ensure, Result};

use crate::transaction::{Amount, TransactionId};

#[derive(Debug, Default, PartialEq)]
pub struct ClientData {
    available: Amount,

    held: Amount,

    total: Amount,

    locked: bool,

    transactions: HashMap<TransactionId, Amount>,

    in_dispute: HashSet<TransactionId>,

    reverted: HashSet<TransactionId>,
}

impl ClientData {
    pub fn available(&self) -> Amount {
        self.available
    }

    pub fn held(&self) -> Amount {
        self.held
    }

    pub fn total(&self) -> Amount {
        self.total
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn deposit(&mut self, id: TransactionId, amount: Amount) -> Result<()> {
        ensure!(
            !self.transactions.contains_key(&id),
            "Duplicate transaction id"
        );

        self.available += amount;
        self.total += amount;

        self.transactions.insert(id, amount);

        Ok(())
    }

    pub fn withdraw(&mut self, id: TransactionId, amount: Amount) -> Result<()> {
        ensure!(
            !self.transactions.contains_key(&id),
            "Duplicate transaction id"
        );
        ensure!(
            amount <= self.available,
            "Insufficient available funds for withdrawal"
        );

        self.available -= amount;
        self.total -= amount;

        self.transactions.insert(id, -amount);

        Ok(())
    }

    pub fn dispute(&mut self, id: TransactionId) -> Result<()> {
        ensure!(
            !self.in_dispute.contains(&id),
            "Transaction is already in dispute"
        );
        ensure!(
            !self.reverted.contains(&id),
            "Transaction was already reversed"
        );

        let Some(amount) = self.transactions.get(&id) else {
            eprintln!("Unknown transaction to dispute");
            return Ok(());
        };
        ensure!(
            self.available >= *amount,
            "Insufficient available funds for dispute"
        );

        self.available -= *amount;
        self.held += *amount;

        self.in_dispute.insert(id);

        Ok(())
    }

    pub fn resolve(&mut self, id: TransactionId) -> Result<()> {
        ensure!(
            self.in_dispute.contains(&id),
            "Transaction is not in dispute"
        );

        let Some(amount) = self.transactions.get(&id) else {
            // If transaction in in_dispute list then it should be in the transactions list too
            unreachable!("Unknown transaction to resolve");
        };
        ensure!(
            self.available >= -*amount,
            "Insufficient available funds to resolve dispute"
        );

        self.held -= *amount;
        self.available += *amount;

        // Transaction dispute cancelled and can be disputed again later.
        // TODO: put transaction into the reverted list if it cannot be disputed more than once.
        self.in_dispute.remove(&id);

        Ok(())
    }

    pub fn chargeback(&mut self, id: TransactionId) -> Result<()> {
        ensure!(
            self.in_dispute.contains(&id),
            "Transaction is not in dispute"
        );

        let Some(amount) = self.transactions.get(&id) else {
            // If transaction in in_dispute list then it should be in the transactions list too
            unreachable!("Unknown transaction to chargeback");
        };

        self.held -= *amount;
        self.total -= *amount;

        self.in_dispute.remove(&id); // Not in dispute anymore
        self.reverted.insert(id); // Prevent transaction to be reverted more than once

        Ok(())
    }
}
