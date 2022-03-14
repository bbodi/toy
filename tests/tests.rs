use std::io::BufWriter;
use transactions_lib::process_input_then_write_output;

#[test]
fn simple_simple_deposit_test() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 1.0
         deposit    ,2      ,2  , 2.0
         deposit    ,1      ,3  , 2.0",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,3         ,0    ,3     , false
         2      ,2         ,0    ,2     , false",
    );
}

#[test]
fn simple_simple_withdrawal_test() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 10.0
         deposit    ,2      ,2  , 20.0
         withdrawal ,1      ,4  , 1.5
         withdrawal ,2      ,5  , 3.0",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,8.5       ,0    ,8.5   , false
         2      ,17        ,0    ,17    , false",
    );
}

#[test]
fn simple_insufficient_withdrawal_test() {
    // The values must be unchanged because the transaction should be ignored due to insufficient funds
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         withdrawal ,1      ,4  , 2.0",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,0         ,0    ,0     , false",
    );
}

#[test]
fn simple_insufficient_withdrawal_test2() {
    // The values must be unchanged because the transaction should be ignored due to insufficient funds
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 1.0
         withdrawal ,1      ,4  , 2.0",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,1         ,0    ,1     , false",
    );
}

#[test]
fn excessive_precision_is_ignored() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,4  , 2.99991",
        // OUTPUT CSV
        "client ,available ,held ,total  , locked
         1      ,2.9999    ,0    ,2.9999 , false",
    );
}

#[test]
fn test_precision() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,4  , 2.1111
         deposit    ,1      ,5  , 2.8889",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,5         ,0    ,5     , false",
    );
}

// This test fails with both f64 and the 'fixed' crate :( For both, the result is 0.2999 instead of 0.3
#[test]
fn test_precision_loss() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,3  , 0.1
         deposit    ,1      ,4  , 0.1
         deposit    ,1      ,5  , 0.1",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,0.3       ,0    ,0.3   , false",
    );
}

#[test]
fn invalid_input_stops_processing() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx      , amount
         deposit    ,aaa    ,222     , 333",
        // OUTPUT CSV
        "Error: Invalid Client ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx      , amount
         deposit    ,111    ,bbb     , 333",
        // OUTPUT CSV
        "Error: Invalid Transaction ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx      , amount
         deposit    ,111    ,222     , ccc",
        // OUTPUT CSV
        "Error: Invalid amount at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx      , amount
         deposit    ,111    ,222     , 333
         deposit    ,111    ,222     , ccc",
        // OUTPUT CSV
        "Error: Invalid amount at line 2",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx      , amount
         dispute    ,aaa    ,222     ,",
        // OUTPUT CSV
        "Error: Invalid Client ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx      , amount
         resolve    ,aaa    ,222     ,",
        // OUTPUT CSV
        "Error: Invalid Client ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx      , amount
         chargeback ,aaa    ,222     ,",
        // OUTPUT CSV
        "Error: Invalid Client ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx      , amount
         dispute    ,111    ,bbb     ,",
        // OUTPUT CSV
        "Error: Invalid Transaction ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx      , amount
         resolve    ,111    ,bbb     ,",
        // OUTPUT CSV
        "Error: Invalid Transaction ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx      , amount
         chargeback ,111    ,bbb     ,",
        // OUTPUT CSV
        "Error: Invalid Transaction ID at line 1",
    );
}

#[test]
fn test_dispute_simple() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 1.0
         dispute    ,1      ,1",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,0         ,1    ,1     , false",
    );
}

#[test]
fn test_dispute_non_existing_tx() {
    {
        assert_csv_eq(
            // INPUT CSV
            "type       ,client ,tx , amount
             dispute    ,1      ,1",
            // OUTPUT CSV
            "client ,available ,held ,total , locked",
        );
    }
    {
        assert_csv_eq(
            // INPUT CSV
            "type       ,client ,tx , amount
             deposit    ,1      ,1  , 1.0
             dispute    ,1      ,2",
            // OUTPUT CSV
            "client ,available ,held ,total , locked
             1      ,1         ,0    ,1     , false",
        );
    }
}

#[test]
fn disputing_deposit_is_not_allowed_for_widthrawals() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 1.0
         withdrawal ,1      ,2  , 0.3
         dispute    ,1      ,2  ,", // this dispute should be ignored
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,0.7       ,0    ,0.7   , false",
    );
}

#[test]
fn transactions_with_same_id_is_ignored() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 1.0
         deposit    ,1      ,1  , 2.0", // this one should be ignored
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,1         ,0    ,1     , false",
    );
}

#[test]
fn multiple_disputes_are_allowed() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 1.0
         dispute    ,1      ,1  ,
         dispute    ,1      ,1  ,",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,0         ,1    ,1     , false",
    );
}

#[test]
fn test_resolve_simple() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 1.0
         dispute    ,1      ,1 
         resolve    ,1      ,1",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,1         ,0    ,1     , false",
    );
}

#[test]
fn multiple_resolves_are_ignored() {
    // resolve twice
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 100.0
         deposit    ,1      ,2  ,  50.0
         dispute    ,1      ,2 
         resolve    ,1      ,2
         resolve    ,1      ,2",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,150       ,0    ,150   , false",
    );
    // first dispute again the already resolved transaction, then resolve for the second time
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 100.0
         deposit    ,1      ,2  ,  50.0
         dispute    ,1      ,2 
         resolve    ,1      ,2
         dispute    ,1      ,2 
         resolve    ,1      ,2",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,150       ,0    ,150   , false",
    );
}

#[test]
fn resolving_an_undisputed_tx_should_be_ignored() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 100.0
         resolve    ,1      ,1",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,100       ,0    ,100   , false",
    );
}

#[test]
fn chargebacking_an_undisputed_tx_should_be_ignored() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 100.0
         chargeback ,1      ,1",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,100       ,0    ,100   , false",
    );
}

#[test]
fn test_chargeback_simple() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 1.0
         dispute    ,1      ,1 
         chargeback ,1      ,1",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,0         ,0    ,0     , true",
    );
}

#[test]
fn test_chargeback_with_two_deposits() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 100.0
         deposit    ,1      ,2  ,  50.0
         dispute    ,1      ,2",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,100       ,50   ,150   , false",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 100.0
         deposit    ,1      ,2  ,  50.0
         dispute    ,1      ,2 
         chargeback ,1      ,2",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,100       ,0    ,100   , true",
    );
}

#[test]
fn multiple_chargebacks_are_not_allowed() {
    // chargeback twice
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 100.0
         deposit    ,1      ,2  ,  50.0
         dispute    ,1      ,2 
         chargeback ,1      ,2
         chargeback ,1      ,2",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,100       ,0    ,100   , true",
    );
    // TODO @clarify Is dispute allowed on a locked client? Currently it is prohibited
    // first dispute again the already chargebacked transaction, then chargeback for the second time
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 100.0
         deposit    ,1      ,2  ,  50.0
         dispute    ,1      ,2 
         chargeback ,1      ,2
         dispute    ,1      ,2 
         chargeback ,1      ,2",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,100       ,0    ,100   , true",
    );
}

#[test]
fn test_no_deposit_allowed_for_a_locked_client() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 100.0
         dispute    ,1      ,1 
         chargeback ,1      ,1
         deposit    ,1      ,2  ,  50.0",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,0         ,0    ,0     , true",
    );
}

#[test]
fn test_no_withdrawal_allowed_for_a_locked_client() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,1  , 100.0
         deposit    ,1      ,2  , 100.0
         dispute    ,1      ,2 
         chargeback ,1      ,2
         withdrawal ,1      ,3  ,  50.0",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,100       ,0    ,100   , true",
    );
}

#[test]
fn test_invalid_input_test() {
    assert_csv_eq(
        // INPUT CSV
        "invalid",
        // OUTPUT CSV
        "Error: Expected columns: type,client,tx,amount",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client
         deposit    ,1",
        // OUTPUT CSV
        "Error: Expected columns: type,client,tx,amount",
    );
    assert_csv_eq(
        // INPUT CSV
        "client ,type    ,tx , amount
         1      ,deposit ,1  , 100.0",
        // OUTPUT CSV
        "Error: Expected columns: type,client,tx,amount",
    );
}

#[test]
fn test_empty_input_test() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount",
        // OUTPUT CSV
        "client ,available ,held ,total , locked",
    );
}

#[test]
fn test_client_ids_can_be_unordered() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,2      ,3  , 100.0
         deposit    ,2      ,2  , 100.0
         deposit    ,1      ,1  , 100.0
         withdrawal ,1      ,4  ,  50.0",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,50        ,0    ,50    , false
         2      ,200       ,0    ,200   , false",
    );
}

#[test]
fn disputing_tx_with_different_client_id_is_ignored() {
    // Disputing Tx 3 but with Client ID 3 TODO @clarify is my assumption correct about prohibiting this?
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,2      ,2  , 100.0
         deposit    ,3      ,3  , 100.0
         dispute    ,2      ,3 ",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         2      ,100       ,0    ,100   , false
         3      ,100       ,0    ,100   , false",
    );
}

#[test]
fn referencing_to_nonexisting_client_is_ignored() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,2      ,3  , 100.0
         dispute    ,999    ,3 ",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         2      ,100       ,0    ,100   , false",
    );
}

#[test]
fn referencing_to_nonexisting_tx_is_ignored() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,2      ,3  , 100.0
         dispute    ,2      ,666 ",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         2      ,100       ,0    ,100   , false",
    );
}

#[test]
fn negative_client_id_is_ignored() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,-2     ,3  , 100.0",
        // OUTPUT CSV
        "Error: Invalid Client ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         withdrawal ,-2     ,3  , 100.0",
        // OUTPUT CSV
        "Error: Invalid Client ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         dispute    ,-2     ,3 ",
        // OUTPUT CSV
        "Error: Invalid Client ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         resolve    ,-2     ,3 ",
        // OUTPUT CSV
        "Error: Invalid Client ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         chargeback ,-2     ,3 ",
        // OUTPUT CSV
        "Error: Invalid Client ID at line 1",
    );
}

#[test]
fn negative_tx_id_is_ignored() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,2      ,-3  , 100.0",
        // OUTPUT CSV
        "Error: Invalid Transaction ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         withdrawal ,2     ,-3  , 100.0",
        // OUTPUT CSV
        "Error: Invalid Transaction ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         dispute    ,2     ,-3 ",
        // OUTPUT CSV
        "Error: Invalid Transaction ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         resolve    ,2     ,-3 ",
        // OUTPUT CSV
        "Error: Invalid Transaction ID at line 1",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         chargeback ,2     ,-3 ",
        // OUTPUT CSV
        "Error: Invalid Transaction ID at line 1",
    );
}

#[test]
fn negative_amount_is_ignored() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,2      ,3  , -100.0",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         2      ,0         ,0    ,0     , false",
    );
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         withdrawal ,2      ,3  , -100.0",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         2      ,0         ,0    ,0     , false",
    );
}

#[test]
fn client_has_no_available_amount_for_dispute() {
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,3  , 100.0
         withdrawal ,1      ,4  , 50.0
         dispute    ,1      ,3",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,50        ,0    ,50     , false",
    );
    // The dispute was ignored, so it cannot be chargebacked
    assert_csv_eq(
        // INPUT CSV
        "type       ,client ,tx , amount
         deposit    ,1      ,3  , 100.0
         withdrawal ,1      ,4  , 50.0
         dispute    ,1      ,3
         chargeback ,1      ,3",
        // OUTPUT CSV
        "client ,available ,held ,total , locked
         1      ,50        ,0    ,50     , false",
    );
}

fn assert_csv_eq(input: &str, expected: &str) {
    let mut actual_output = BufWriter::new(Vec::new());
    process_input_then_write_output(input.as_bytes(), &mut actual_output);

    fn remove_whitespace(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }
    let actual_output = String::from_utf8(actual_output.into_inner().unwrap()).unwrap();
    let expected_line_count = expected.lines().count();
    let actual_line_count = actual_output.lines().count();
    // Compare them line by line so it is easier to find the differences in case of mismatch
    assert_eq!(
        expected_line_count,
        actual_line_count,
        "Expected {} lines but got {}\nExpected:\n{}\nActual:\n{}",
        expected_line_count,
        actual_line_count,
        expected.trim_start(),
        actual_output
    );
    for (i, (expected_line, actual_line)) in expected.lines().zip(actual_output.lines()).enumerate()
    {
        assert_eq!(
            remove_whitespace(expected_line),
            remove_whitespace(actual_line),
            "Mismatch in the {}. line",
            i,
        );
    }
}
