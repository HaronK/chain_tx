use std::collections::HashMap;

use anyhow::{anyhow, ensure, Result};

use crate::client::ClientData;
use crate::transaction::{ClientId, Transaction, TransactionType};

#[derive(Default)]
pub struct Engine {
    clients: HashMap<ClientId, ClientData>,
}

impl Engine {
    pub fn apply_transactions(&mut self, csv_path: &str) -> Result<()> {
        let mut rdr = csv::Reader::from_path(csv_path)
            .map_err(|err| anyhow!("Cannot read CSV file: {csv_path}. Error: {err}"))?;

        for tx in rdr.deserialize() {
            let tx: Transaction =
                tx.map_err(|err| anyhow!("Cannot read transaction. Error: {err}"))?;
            ensure!(tx.amount >= 0.0, "Negative amount is not allowed");

            let client = if tx.ty == TransactionType::Deposit {
                self.clients.entry(tx.client).or_default()
            } else {
                self.clients
                    .get_mut(&tx.client)
                    .ok_or_else(|| anyhow!("Unknown client: {}", *tx.client))?
            };

            match tx.ty {
                TransactionType::Deposit => client.deposit(tx.tx, tx.amount)?,
                TransactionType::Withdrawal => client.withdraw(tx.tx, tx.amount)?,
                TransactionType::Dispute => client.dispute(tx.tx)?,
                TransactionType::Resolve => client.resolve(tx.tx)?,
                TransactionType::Chargeback => client.chargeback(tx.tx)?,
            }
        }

        Ok(())
    }

    pub fn print_summary(&self) {
        println!("client, available, held, total, locked");

        self.clients.iter().for_each(|(id, data)| {
            println!(
                "{}, {}, {}, {}, {}",
                **id,
                data.available(),
                data.held(),
                data.total(),
                data.is_locked()
            )
        });
    }
}
