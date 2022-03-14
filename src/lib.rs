pub mod amount;
mod client;
mod transaction;

use crate::amount::Amount;
use crate::client::{Client, ClientId};
use crate::transaction::{
    DepositedTransaction, Dispute, DisputeState, TransactionId, Transfer, TransferType,
};
use csv::{Reader, StringRecord};
use std::error::Error;
use std::io::Read;

/// A type definition for HashMap, so it is easy to replace the implementation if needed.
/// FxHashMap is 10 times faster on my computer
type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;

// uncomment this if you want the implementation from the standard lib
//type HashMap<K, V> = std::collections::HashMap<K, V>;

/// It reads the csv in the expected format from the `input` and write the result client states into
/// the `output`
pub fn process_input_then_write_output(input: impl std::io::Read, mut output: impl std::io::Write) {
    match run_transactions(input) {
        Ok(result) => {
            if let Err(err) = write_client_states_to(result, &mut output) {
                writeln!(output, "Error: {}", err).unwrap();
            }
        }
        Err(err) => {
            writeln!(output, "Error: {}", err).unwrap();
        }
    }
}

/// Writes the clients state passed in the `result` argument into `writer`.
/// The output format is a csv defined in the task description.
///
/// It can return an `Err` only when there is an error writing to `writer`.
fn write_client_states_to(
    result: HashMap<ClientId, Client>,
    writer: &mut impl std::io::Write,
) -> Result<(), Box<dyn Error>> {
    writeln!(writer, "client, available, held, total, locked")?;
    // @doc
    // In order to be able to verify the output easily in the integration tests,
    // the output is ordered.
    // It has some unnecessary performance penalty since it is not a requirement.
    // In a real world scenario with more time I would implement a more sophisticated test
    // utility which does not have assumption about output ordering.
    let mut keys: Vec<ClientId> = result.keys().map(|it| *it).collect();
    keys.sort();

    for client_id in keys.iter() {
        let client = &result[client_id];
        writeln!(
            writer,
            "{},{},{},{},{}",
            client_id.0, client.available, client.held, client.total, client.locked
        )?;
    }

    Ok(())
}

/// Reads the csv from `reader` and process them according to the documentation.
/// The output is a `HashMap<ClientId, Client>`, the state of the clients after the transactions have affected them.
fn run_transactions(
    reader: impl std::io::Read,
) -> Result<HashMap<ClientId, Client>, Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(reader);

    validate_header(&mut rdr)?;

    let mut transactions: HashMap<TransactionId, Transfer> = HashMap::default();
    let mut clients: HashMap<ClientId, Client> = HashMap::default();
    for (record_index, result) in rdr.records().enumerate() {
        let line_index = record_index + 1;
        let csv_line: InputCsvLine = parse_transaction(line_index, result?)?;
        match csv_line {
            InputCsvLine::Transfer(tx) => {
                if transactions.contains_key(&tx.id) {
                    continue;
                }
                let client = get_or_create_client(&mut clients, tx.client_id);
                match &tx.typ {
                    TransferType::Deposit(DepositedTransaction { amount, .. }) => {
                        if !client.locked {
                            client.deposit(*amount);
                        }
                    }
                    TransferType::Withdrawal { amount } => {
                        if !client.locked {
                            client.withdrawal(*amount);
                        }
                    }
                }
                transactions.insert(tx.id, tx);
            }
            InputCsvLine::Dispute(dispute) => {
                match dispute.state {
                    DisputeState::Dispute => {
                        if let Some(deposit) = get_deposit_transaction(&mut transactions, &dispute)
                        {
                            if deposit.disputed == false {
                                let client = get_or_create_client(&mut clients, dispute.client_id);
                                // TODO @clarify What to do when client does not have the available amount?
                                if client.locked == false && client.available >= deposit.amount {
                                    client.dispute(deposit.amount);
                                    deposit.disputed = true;
                                }
                            }
                        } else {
                            // according to the business requirements, non existing referenced transactions are expected
                        }
                    }
                    DisputeState::Resolve => {
                        if let Some(deposit) = get_deposit_transaction(&mut transactions, &dispute)
                        {
                            if deposit.disputed {
                                let client = get_or_create_client(&mut clients, dispute.client_id);
                                client.resolve(deposit.amount);
                                deposit.disputed = false;
                            } else {
                                // according to the business requirements, it is an error on our partner's side
                            }
                        } else {
                            // according to the business requirements, non existing referenced transactions are expected
                        }
                    }
                    DisputeState::Chargeback => {
                        if let Some(deposit) = get_deposit_transaction(&mut transactions, &dispute)
                        {
                            if deposit.disputed {
                                let client = get_or_create_client(&mut clients, dispute.client_id);
                                if !client.locked {
                                    client.chargeback(deposit.amount);
                                    deposit.disputed = false;
                                }
                            } else {
                                // according to the business requirements, it is an error on our partner's side
                            }
                        } else {
                            // according to the business requirements, non existing referenced transactions are expected
                        }
                    }
                }
            }
        }
    }
    return Ok(clients);
}

/// Validate the header of the input csv.
fn validate_header(rdr: &mut Reader<impl Read>) -> Result<(), Box<dyn Error>> {
    let headers = rdr.headers()?;

    let error_msg = "Expected columns: type, client, tx, amount";
    if headers.len() != 4 {
        return Err(Box::new(CsvParsingError::new(error_msg)));
    }
    if headers[0].trim().to_ascii_lowercase() != "type" {
        return Err(Box::new(CsvParsingError::new(error_msg)));
    } else if headers[1].trim().to_ascii_lowercase() != "client" {
        return Err(Box::new(CsvParsingError::new(error_msg)));
    } else if headers[2].trim().to_ascii_lowercase() != "tx" {
        return Err(Box::new(CsvParsingError::new(error_msg)));
    } else if headers[3].trim().to_ascii_lowercase() != "amount" {
        return Err(Box::new(CsvParsingError::new(error_msg)));
    }

    Ok(())
}

#[derive(Debug)]
pub struct CsvParsingError {
    details: String,
}

impl CsvParsingError {
    fn new(msg: impl Into<String>) -> CsvParsingError {
        CsvParsingError {
            details: msg.into(),
        }
    }
}

impl std::fmt::Display for CsvParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for CsvParsingError {
    fn description(&self) -> &str {
        &self.details
    }
}

/// A type that represents a line from the input csv file in a typesafe manner.
enum InputCsvLine {
    Transfer(Transfer),
    Dispute(Dispute),
}

/// Parses a single input csv line
fn parse_transaction(
    line_index: usize,
    columns: StringRecord,
) -> Result<InputCsvLine, CsvParsingError> {
    let typ = columns[0].trim();
    let client_id = ClientId(columns[1].trim().parse().map_err(|_err| {
        CsvParsingError::new(format!("Invalid Client ID at line {}", line_index))
    })?);
    let tx_id = TransactionId(columns[2].trim().parse().map_err(|_err| {
        CsvParsingError::new(format!("Invalid Transaction ID at line {}", line_index))
    })?);
    match typ {
        "withdrawal" => Ok(InputCsvLine::Transfer(Transfer {
            id: tx_id,
            client_id,
            typ: TransferType::Withdrawal {
                amount: Amount::parse(columns[3].trim()).ok_or_else(|| {
                    CsvParsingError::new(format!("Invalid amount at line {}", line_index))
                })?,
            },
        })),
        "deposit" => Ok(InputCsvLine::Transfer(Transfer {
            id: tx_id,
            client_id,
            typ: TransferType::Deposit(DepositedTransaction {
                amount: Amount::parse(columns[3].trim()).ok_or_else(|| {
                    CsvParsingError::new(format!("Invalid amount at line {}", line_index))
                })?,
                disputed: false,
            }),
        })),
        "dispute" => Ok(InputCsvLine::Dispute(Dispute {
            disputed_tx_id: tx_id,
            client_id,
            state: DisputeState::Dispute,
        })),
        "resolve" => Ok(InputCsvLine::Dispute(Dispute {
            disputed_tx_id: tx_id,
            client_id,
            state: DisputeState::Resolve,
        })),
        "chargeback" => Ok(InputCsvLine::Dispute(Dispute {
            disputed_tx_id: tx_id,
            client_id,
            state: DisputeState::Chargeback,
        })),
        _ => {
            return Err(CsvParsingError::new(format!(
                "Invalid transaction type: {}",
                typ
            )))
        }
    }
}

/// A utility function which returns a transaction referenced by the dispute, if the
/// transaction is a dispute and has the same Client ID as the dispute. Otherwise it returns `None`.
///
/// This function is needed to hide the pattern matching and so make the caller code more readable.
fn get_deposit_transaction<'a>(
    transactions: &'a mut HashMap<TransactionId, Transfer>,
    dispute: &Dispute,
) -> Option<&'a mut DepositedTransaction> {
    transactions
        .get_mut(&dispute.disputed_tx_id)
        .and_then(|tx| match tx {
            Transfer {
                typ: TransferType::Deposit(deposit),
                ..
            } if tx.client_id == dispute.client_id => Some(deposit),
            _ => None,
        })
}

/// Returns the referenced client, or if it does not exists, it creates one with the default values.
fn get_or_create_client(
    clients: &mut HashMap<ClientId, Client>,
    client_id: ClientId,
) -> &mut Client {
    clients.entry(client_id).or_insert_with(|| Client::new())
}
