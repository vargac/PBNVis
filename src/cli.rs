use std::{io, process, fs};
use std::io::Write;


pub fn load_bn(args: &[String]) -> String {
    if args.len() < 2 {
        eprintln!("Use with one parameter -- path to the .aeon model");
        process::exit(1);
    }
    fs::read_to_string(&args[1]).unwrap_or_else(|err| {
        eprintln!("Cannot read the file, err: {}", err);
        process::exit(1);
    })
}

pub fn read_color(min: u32, max: u32) -> u32 {
    let mut color_i: u32 = 0;
    while color_i == 0 {
        print!("Choose one: ");
        io::stdout().flush().unwrap();
        let mut color_str = String::new();
        color_i = match io::stdin().read_line(&mut color_str) {
            Ok(_) => match color_str.trim().parse() {
                Ok(n) => n,
                Err(e) => {
                    eprintln!("Error parsing \"{}\": {}", color_str.trim(), e);
                    0
                }
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                0
            }
        };
        if color_i < min || color_i > max {
            eprintln!("Invalid color");
            color_i = 0;
        }
    }
    color_i
}
