use std::collections::HashMap;

use anyhow::{anyhow, ensure, Result};

use crate::client::ClientData;
use crate::transaction::{ClientId, Transaction, TransactionType};

#[derive(Default)]
pub struct Engine {
    clients: HashMap<ClientId, ClientData>,
}

impl Engine {
    pub fn apply_transactions<R: std::io::Read>(&mut self, rdr: R) -> Result<()> {
        let mut rdr = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(rdr);

        for tx in rdr.deserialize() {
            let tx: Transaction =
                tx.map_err(|err| anyhow!("Cannot read transaction. Error: {err}"))?;
            ensure!(tx.amount >= 0.0, "Negative amount is not allowed");

            if let Err(_err) = self.apply_transaction(&tx) {
                // eprintln!("Error: {err}");
            }
        }

        Ok(())
    }

    fn apply_transaction(&mut self, tx: &Transaction) -> Result<()> {
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

        Ok(())
    }

    pub fn print_summary(&self) {
        println!("client, available, held, total, locked");

        self.clients.iter().for_each(|(id, data)| {
            println!(
                "{}, {:.4}, {:.4}, {:.4}, {}",
                **id,
                data.available(),
                data.held(),
                data.total(),
                data.is_locked()
            )
        });
    }
}

#[cfg(test)]
mod tests {

    use crate::transaction::Amount;

    use super::Engine;

    #[test]
    fn test_deposit() {
        do_test(
            include_str!("../test_data/deposit.csv"),
            &[(1, 1.0, 0.0, 1.0, false)],
        );
    }

    #[test]
    fn test_withdraw() {
        do_test(
            include_str!("../test_data/withdraw.csv"),
            &[(1, 0.5, 0.0, 0.5, false)],
        );
    }

    #[test]
    #[should_panic]
    fn test_withdraw_too_much_fail() {
        do_test(include_str!("../test_data/withdraw_too_much_fail.csv"), &[]);
    }

    #[test]
    fn test_dispute_deposit() {
        do_test(
            include_str!("../test_data/dispute_deposit.csv"),
            &[(1, 0.0, 1.0, 1.0, false)],
        );
    }

    #[test]
    fn test_resolve_deposit() {
        do_test(
            include_str!("../test_data/resolve_deposit.csv"),
            &[(1, 1.0, 0.0, 1.0, false)],
        );
    }

    #[test]
    fn test_chargeback_deposit() {
        do_test(
            include_str!("../test_data/chargeback_deposit.csv"),
            &[(1, 0.0, 0.0, 0.0, true)],
        );
    }

    #[test]
    fn test_dispute_withdrawal() {
        do_test(
            include_str!("../test_data/dispute_withdrawal.csv"),
            &[(1, 1.0, -0.5, 0.5, false)],
        );
    }

    #[test]
    fn test_resolve_withdrawal() {
        do_test(
            include_str!("../test_data/resolve_withdrawal.csv"),
            &[(1, 0.5, 0.0, 0.5, false)],
        );
    }

    #[test]
    fn test_chargeback_withdrawal() {
        do_test(
            include_str!("../test_data/chargeback_withdrawal.csv"),
            &[(1, 1.0, 0.0, 1.0, true)],
        );
    }

    #[test]
    fn test_multi_dispute() {
        do_test(
            include_str!("../test_data/multi_dispute.csv"),
            &[(1, 17.0, 7.0, 24.0, true)],
        );
    }

    #[test]
    fn test_multi_client() {
        do_test(
            include_str!("../test_data/multi_client.csv"),
            &[(1, 11.0, 0.0, 11.0, true), (2, 12.0, 7.0, 19.0, false)],
        );
    }

    fn do_test(csv: &str, expected: &[(u16, Amount, Amount, Amount, bool)]) {
        let mut engine = Engine::default();

        engine
            .apply_transactions(csv.as_bytes())
            .expect("Cannot apply transaction");

        assert_eq!(
            expected.len(),
            engine.clients.len(),
            "Unexpected number of clients"
        );

        for (client_id, available, held, total, locked) in expected {
            let client_data = engine
                .clients
                .get(&(*client_id).into())
                .expect(&format!("Unknown client: {client_id}"));

            assert_eq!(
                *available,
                client_data.available(),
                "Wrong available amount"
            );
            assert_eq!(*held, client_data.held(), "Wrong held amount");
            assert_eq!(*total, client_data.total(), "Wrong total amount");
            assert_eq!(*locked, client_data.is_locked(), "Wrong locked flag");
        }
    }
}
