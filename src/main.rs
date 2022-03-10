use std::{fs::File, io::Read, time::Instant};

use jsonschema::json::{Key, KeyPart, Lexer, Parser};

fn main() -> Result<(), String> {
    let mut args = std::env::args();
    args.next();

    let file_name = if let Some(value) = args.next() {
        value
    } else {
        return Err("No file provided".to_string());
    };

    let string = {
        let start = Instant::now();
        let mut file =
            File::open(file_name).map_err(|_| "Could not find input file".to_string())?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).ok();

        let string = String::from_utf8(data)
            .map_err(|e| format!("Failed to read data to string. {:?}", e))?;

        let end = Instant::now();

        println!("Read file in {} ms", (end - start).as_millis());

        string
    };

    println!("Lexing");
    let start = Instant::now();
    let tokens = Lexer::lex_str(&string).map_err(|e| format!("Failed to lex. {:?}", e))?;
    let end = Instant::now();
    println!("Lexed in {} ms", (end - start).as_millis());

    println!("Parsing");
    let start = Instant::now();
    let parsed = Parser::parse_tokens(&tokens).map_err(|e| format!("Failed to parse: {:?}", e))?;
    let end = Instant::now();
    println!("Lexed in {} ms", (end - start).as_millis());

    println!("{:?}", parsed.is_some());

    println!(
        "{:?}",
        parsed.unwrap().get(&mut Key::new(vec![
            KeyPart::Identifier("features".to_string()),
            KeyPart::Index(0),
            KeyPart::Identifier("properties".to_string()),
            KeyPart::Identifier("BLOCK_NUM".to_string()),
        ]))
    );

    Ok(())
}
