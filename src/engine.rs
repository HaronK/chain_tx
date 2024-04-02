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

#[cfg(test)]
mod tests {

    use crate::transaction::Amount;

    use super::Engine;

    #[test]
    fn test_deposit() {
        do_test("deposit, 1, 1, 1.0", &[(1, 1.0, 0.0, 1.0, false)]);
    }

    #[test]
    fn test_withdraw() {
        do_test(
            "deposit, 1, 1, 1.0\nwithdrawal, 1, 2, 0.5",
            &[(1, 0.5, 0.0, 0.5, false)],
        );
    }

    #[test]
    #[should_panic]
    fn test_withdraw_too_much_fail() {
        do_test("deposit, 1, 1, 1.0\nwithdrawal, 1, 2, 1.5", &[]);
    }

    #[test]
    fn test_dispute_deposit() {
        do_test(
            "deposit, 1, 1, 1.0\ndispute, 1, 1, 0.0",
            &[(1, 0.0, 1.0, 1.0, false)],
        );
    }

    #[test]
    fn test_resolve_deposit() {
        do_test(
            "deposit, 1, 1, 1.0\ndispute, 1, 1, 0.0\nresolve, 1, 1, 0.0",
            &[(1, 1.0, 0.0, 1.0, false)],
        );
    }

    #[test]
    fn test_chargeback_deposit() {
        do_test(
            "deposit, 1, 1, 1.0\ndispute, 1, 1, 0.0\nchargeback, 1, 1, 0.0",
            &[(1, 0.0, 0.0, 0.0, true)],
        );
    }

    #[test]
    fn test_dispute_withdraw() {
        do_test(
            "deposit, 1, 1, 1.0\nwithdrawal, 1, 2, 0.5\ndispute, 1, 2, 0.0",
            &[(1, 1.0, -0.5, 0.5, false)],
        );
    }

    #[test]
    fn test_resolve_withdraw() {
        do_test(
            "deposit, 1, 1, 1.0\nwithdrawal, 1, 2, 0.5\ndispute, 1, 2, 0.0\nresolve, 1, 2, 0.0",
            &[(1, 0.5, 0.0, 0.5, false)],
        );
    }

    #[test]
    fn test_chargeback_withdraw() {
        do_test(
            "deposit, 1, 1, 1.0\nwithdrawal, 1, 2, 0.5\ndispute, 1, 2, 0.0\nchargeback, 1, 2, 0.0",
            &[(1, 1.0, 0.0, 1.0, true)],
        );
    }

    fn do_test(csv: &str, expected: &[(u16, Amount, Amount, Amount, bool)]) {
        let csv_string = "type, client, tx, amount\n".to_string() + csv;
        let mut engine = Engine::default();

        engine
            .apply_transactions(csv_string.as_bytes())
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
