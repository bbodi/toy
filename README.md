

## Types
I used the type system to guarantee as much correctness as possible as it can be seen in the 
`transaction.rs` file and `InputCsvLine` type.

## Amount type
For `amount`, floating point types were not an option, in financial calculations they are not
welcomed due to their precision problems.  
I wanted to use [fixed](https://crates.io/crates/fixed) because I already have experience
with it, but it still suffers from the same problem.   
E.g. both `f64` and `FixedU64<U14>` failed the `test_precision_loss` test (can be found in `tests.rs`).

So I rather implemented a really simple fixed point integer type called `Amount`, which went
through the above test successfully and was created specifically for this task, keeping in mind
the requirements for the 4 digit precision after the decimal.  

It can represent numbers in the range of `0 .. 1 844 674 407 370 955.1615`.  
**Warning**  
Overflow results in panic. In a real scenario, the expected range, overflows,
rounding strategy and precision loss should be defined and handled correctly.

The type is documented and tested in `amount.rs`.

## Performance 

### rustc-hash

Since the task does not mention requirements about HashMap implementation and expectation to be
cryptographically safe, I experimented a bit
with [rustc-hash](https://crates.io/crates/rustc-hash), and I found it 10 times faster for a large input set (243 MB csv) that contains mostly
deposits. (~34s vs ~3s)

### Stream based reading
The app uses rust's `BufReader` to read from the input csv. I tested it with a generated 2 GB csv file that contained
only disputes (so the application did not have to store anything into its map), and the memory usage
was only around 600-800 KB.


## Testing
I am a huge believer in integration/end-to-end tests, according to my experience,
unit tests, even if we had 100% coverage, do not provide any guarantee that the
application is bug free or work as expected, and can be even detrimental if they are written poorly, exposing or
relying on implementation details.  
Of course they have their places, but based on my experience, tests that cover the
scenario as close as possible to the production usage and from as high level as possible
are the best.  

So I wrote "integration tests" for this task, they can be found in `tests.rs`.  
I tried to make it as readable as possible, and providing useful error messages in case
of a test failure. E.g. a test looks like this:
```rust
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
```
It is useful if the tests are readable, so non-technical managers/product owners can write/verify tests as well.
## Output csv
In order to be able to verify the output easily in the integration tests, the output is sorted by the Client IDs.

Sorting has some unnecessary performance penalty since it takes time, and it was not a requirement.  
In a real world scenario with more time, I would implement a more sophisticated test
utility which does not have assumption about output ordering, and then the sorting could be removed.



## Error handling
Right now, errors are either ignored according to the documentation, or they stop the whole processing.    
In a real scenario, both cases should at least be logged. Further discussion is needed
from the business side perspective about error handling, what should be ignored, how to avoid
stopping the whole processing in case of a single faulty input line, etc.   
E.g. I can imagine that it can
be better to lock a client which for the csv has invalid input, rather than stop the whole 
processing.

## Questions
I placed some `TODO @clarify` comment into the code to express my doubts about
unclear aspects of the documentation.  

- What to do when the client does not have the available amount for a disputed transaction? (currently the dispute is ignored)
- Is dispute allowed on a locked client? Currently, it is prohibited.
- In a dispute, does the client id in the dispute/withdrawal/chargeback has to be the same as in the referenced transaction?
Currently, yes, and if not, the transaction is ignored.
