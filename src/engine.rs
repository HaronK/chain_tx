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
    use std::collections::HashMap;

    use crate::transaction::{Amount, ClientId};

    use super::Engine;

    fn do_test(csv: &str, expected_clients: &HashMap<ClientId, (Amount, Amount, Amount, bool)>) {
        let csv_string = "type, client, tx, amount\n".to_string() + csv;
        let mut engine = Engine::default();

        engine
            .apply_transactions(csv_string.as_bytes())
            .expect("Cannot apply transaction");

        assert_eq!(
            expected_clients.len(),
            engine.clients.len(),
            "Unexpected number of clients"
        );

        for (id, (available, held, total, locked)) in expected_clients {
            let client_data = engine
                .clients
                .get(&id)
                .expect(&format!("Unknown client: {}", **id));

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

    #[test]
    fn test_deposit() {
        let mut expected_clients = HashMap::new();
        expected_clients.insert(1.into(), (1.0, 0.0, 1.0, false));

        do_test("deposit, 1, 1, 1.0\n", &expected_clients);
    }
}
