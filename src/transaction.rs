use std::ops::Deref;

use anyhow::{anyhow, bail, ensure, Result};

#[derive(Debug)]
pub struct Transaction {
    pub ty: TransactionType,

    pub client: ClientId,

    pub tx: TransactionId,

    pub amount: Amount,
}

impl Transaction {
    pub fn from_fields(fields: &[&str]) -> Result<Self> {
        ensure!(
            fields.len() == 3 || fields.len() == 4,
            "Not enough fields in a record"
        );

        let ty = TransactionType::from_str(fields[0])?;
        let client = fields[1]
            .parse::<u16>()
            .map_err(|err| anyhow!("Cannot parse client id: {}. Error: {err}", fields[1]))?
            .into();
        let tx = fields[2]
            .parse::<u32>()
            .map_err(|err| anyhow!("Cannot parse transaction id: {}. Error: {err}", fields[2]))?
            .into();
        let amount = if fields.len() > 3 {
            fields[3]
                .parse::<f32>()
                .map_err(|err| anyhow!("Cannot parse amount: {}. Error: {err}", fields[3]))?
        } else {
            0.0
        };

        Ok(Self {
            ty,
            client,
            tx,
            amount,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl TransactionType {
    fn from_str(s: &str) -> Result<Self> {
        let ty = match s {
            "deposit" => Self::Deposit,
            "withdrawal" => Self::Withdrawal,
            "dispute" => Self::Dispute,
            "resolve" => Self::Resolve,
            "chargeback" => Self::Chargeback,
            _ => bail!("Unsupported transaction type: {s}"),
        };

        Ok(ty)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(u16);

impl From<u16> for ClientId {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl Deref for ClientId {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId(u32);

impl From<u32> for TransactionId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Deref for TransactionId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Amount = f32;
