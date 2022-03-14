use crate::Amount;

#[derive(Debug, Clone, Copy, Ord, Eq, PartialOrd, PartialEq, Hash)]
pub struct ClientId(pub u16);

pub struct Client {
    pub available: Amount,
    pub held: Amount,
    pub total: Amount,
    pub locked: bool,
}

impl Client {
    pub fn new() -> Client {
        Client {
            available: Amount::zero(),
            held: Amount::zero(),
            total: Amount::zero(),
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: Amount) {
        self.available += amount;
        self.total += amount;
    }

    pub fn withdrawal(&mut self, amount: Amount) {
        if amount > self.available {
            return;
        }
        self.available -= amount;
        self.total -= amount;
    }

    pub fn dispute(&mut self, amount: Amount) {
        self.held += amount;
        self.available -= amount;
    }

    pub fn resolve(&mut self, amount: Amount) {
        self.held -= amount;
        self.available += amount;
    }

    pub fn chargeback(&mut self, amount: Amount) {
        self.held -= amount;
        self.total -= amount;
        self.locked = true;
    }
}
