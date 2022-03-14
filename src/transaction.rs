use crate::client::ClientId;
use crate::Amount;

#[derive(Debug, Clone, Copy, Eq, Ord, PartialOrd, PartialEq, Hash)]
pub struct TransactionId(pub u32);

/// Represents either a deposit or a withdrawal.
/// As a csv, it looks like:
/// ```csv
/// type       ,client ,tx , amount
/// deposit    ,1      ,3  , 100.0
/// withdrawal ,1      ,4  , 50.0
/// ```

#[derive(Debug)]
pub struct Transfer {
    pub id: TransactionId,
    pub client_id: ClientId,
    pub typ: TransferType,
}

#[derive(Debug)]
pub enum TransferType {
    Deposit(DepositedTransaction),
    Withdrawal { amount: Amount },
}

/// Represents either a dispute, resolve or a chargeback.
/// As a csv, it looks like:
/// ```csv
/// type       ,client ,tx , amount
/// dispute    ,1      ,3
/// resolve    ,1      ,3
/// chargeback ,1      ,3
/// ```
#[derive(Debug)]
pub struct Dispute {
    pub disputed_tx_id: TransactionId,
    pub client_id: ClientId,
    pub state: DisputeState,
}

#[derive(Debug)]
pub enum DisputeState {
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug)]
pub struct DepositedTransaction {
    pub amount: Amount,
    pub disputed: bool,
}
