use std::fs::File;
use std::io::BufReader;
use transactions_lib::process_input_then_write_output;

fn main() {
    if let Some(input_filename) = std::env::args().skip(1).next() {
        let file_reader =
            BufReader::new(File::open(input_filename).expect("Could not open the input file"));
        process_input_then_write_output(file_reader, std::io::stdout());
    } else {
        println!("Input file path is missing.");
    }
}
