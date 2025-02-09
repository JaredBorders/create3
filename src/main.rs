use create3::{
    calc_addr, calc_addr_with_bytes, errors::Create3GenerateSaltError,
    generate_salt, generate_salt_prefix,
};
use sha3::{Digest, Keccak256};
use std::io::{self, Write};

/// reads a line from stdin and returns a trimmed string.
fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line.trim().to_owned()
}

/// main entry point for the create3 address tool.
fn main() {
    println!("\x1b[32m=========================\x1b[0m");
    println!("\x1b[32m=  create3 address tool  =\x1b[0m");
    println!("\x1b[32m=========================\x1b[0m");

    loop {
        println!("\n\x1b[36mwhat would you like to do?\x1b[0m");
        println!("\x1b[33m1. generate create3 address\x1b[0m");
        println!("\x1b[33m2. generate salt for prefixed address\x1b[0m");
        println!("\x1b[33m3. generate optimised suffix for prefixed address and salt\x1b[0m");
        println!(
            "\x1b[33m4. generate multiple salts for a prefixed address\x1b[0m"
        );

        let choice = read_input("\x1b[36menter your choice (1/2/3/4):\x1b[0m ");
        match choice.as_str() {
            "1" => {
                // generate create3 address using user-provided salt.
                let deployer = request_deployer_address();
                let salt = read_input("\x1b[36menter salt (utf8):\x1b[0m ");
                let address = calc_addr(&deployer, salt.as_bytes());
                println!(
                    "\x1b[32mcreate3 address:\x1b[0m {}",
                    to_checksum_address(&address)
                );
                break;
            }
            "2" => {
                // generate salt that yields an address with the given prefix.
                let deployer = request_deployer_address();
                let mut prefix = String::new();
                let salt;
                loop {
                    prefix = read_input(
                        "\x1b[36menter prefix (without '0x' prefix):\x1b[0m ",
                    );
                    // use generate_salt to validate prefix.
                    match generate_salt(&deployer, &prefix) {
                        Ok(s) => {
                            salt = s;
                            break;
                        }
                        Err(Create3GenerateSaltError::PrefixNotHexEncoded) => {
                            println!(
                                "\x1b[36minput was not hex encoded.\x1b[0m"
                            );
                        }
                        Err(Create3GenerateSaltError::PrefixTooLong) => {
                            println!(
                                "\x1b[36mprefix was too long (over 20 characters).\x1b[0m"
                            );
                        }
                    }
                }
                let vanity_addr = calc_addr_with_bytes(&deployer, &salt.1);
                println!(
                    "\x1b[32mvanity address:\x1b[0m {}",
                    to_checksum_address(&vanity_addr)
                );
                println!("\x1b[32msalt string:\x1b[0m {}", salt.0);
                println!(
                    "\x1b[32mhashed salt for prefix {}:\x1b[0m 0x{}",
                    prefix,
                    hex::encode(salt.1)
                );
                break;
            }
            "3" => {
                // generate salt with a salt prefix to yield a vanity address.
                let deployer = request_deployer_address();
                let salt_prefix =
                    read_input("\x1b[36menter salt prefix (utf8):\x1b[0m ");
                let mut prefix = String::new();
                let generated;
                let vanity_addr;
                loop {
                    prefix = read_input(
                        "\x1b[36menter address prefix (without '0x' prefix):\x1b[0m ",
                    );
                    match generate_salt_prefix(&deployer, &salt_prefix, &prefix)
                    {
                        Ok(s) => {
                            generated = s;
                            vanity_addr =
                                calc_addr_with_bytes(&deployer, &generated.1);
                            break;
                        }
                        Err(Create3GenerateSaltError::PrefixNotHexEncoded) => {
                            println!(
                                "\x1b[36minput was not hex encoded.\x1b[0m"
                            );
                        }
                        Err(Create3GenerateSaltError::PrefixTooLong) => {
                            println!(
                                "\x1b[36mprefix was too long (over 20 characters).\x1b[0m"
                            );
                        }
                    }
                }
                println!(
                    "\x1b[32mvanity address:\x1b[0m {}",
                    to_checksum_address(&vanity_addr)
                );
                println!(
                    "\x1b[32msalt string for prefix {}:\x1b[0m {}",
                    salt_prefix, generated.0
                );
                println!(
                    "\x1b[32mhashed salt :\x1b[0m 0x{}",
                    hex::encode(&generated.1)
                );
                break;
            }
            "4" => {
                // batch generate salts for a given prefix.
                let deployer = request_deployer_address();
                let mut prefix = String::new();
                loop {
                    prefix = read_input(
                        "\x1b[36menter prefix (without '0x' prefix):\x1b[0m ",
                    );
                    // validate prefix using generate_salt.
                    match generate_salt(&deployer, &prefix) {
                        Ok(_) => break,
                        Err(Create3GenerateSaltError::PrefixNotHexEncoded) => {
                            println!(
                                "\x1b[36minput was not hex encoded.\x1b[0m"
                            );
                        }
                        Err(Create3GenerateSaltError::PrefixTooLong) => {
                            println!(
                                "\x1b[36mprefix was too long (over 20 characters).\x1b[0m"
                            );
                        }
                    }
                }
                let num_str = read_input(
                    "\x1b[36menter number of addresses to generate:\x1b[0m ",
                );
                let num: u32 = num_str.parse().expect("invalid number entered");
                for i in 1..=num {
                    let salt = generate_salt(&deployer, &prefix).unwrap();
                    let vanity_addr = calc_addr_with_bytes(&deployer, &salt.1);
                    println!("\x1b[32mresult {}:\x1b[0m", i);
                    println!("  salt string: {}", salt.0);
                    println!(
                        "  vanity address: {}",
                        to_checksum_address(&vanity_addr)
                    );
                    println!(
                        "  hashed salt for prefix {}: 0x{}",
                        prefix,
                        hex::encode(salt.1)
                    );
                }
                break;
            }
            _ => {
                println!("\x1b[31minvalid choice, please try again.\x1b[0m");
            }
        }
    }
}

/// reads and validates the deployer address from stdin.
/// the address should be in hex (without '0x') and 40 chars long.
fn request_deployer_address() -> Vec<u8> {
    loop {
        let input = read_input("\x1b[36menter deployer address:\x1b[0m ");
        let addr = input.trim_start_matches("0x");
        if !addr.chars().all(|c| c.is_ascii_hexdigit()) {
            println!("\x1b[36minput was not hex encoded.\x1b[0m");
            continue;
        } else if addr.len() != 40 {
            println!(
                "\x1b[36minput has an incorrect length (expected 40).\x1b[0m"
            );
            continue;
        }
        return hex::decode(addr).unwrap();
    }
}

/// converts a 20-byte ethereum address into its eip-55 checksummed form.
/// the address is converted to lowercase hex, hashed with keccak256, and
/// characters are uppercased based on the hash.
fn to_checksum_address(address: &[u8]) -> String {
    let address_hex = hex::encode(address);
    let mut hasher = Keccak256::new();
    hasher.update(address_hex.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode(hash);
    let mut result = String::from("0x");
    for (i, ch) in address_hex.chars().enumerate() {
        let hash_digit = hash_hex.chars().nth(i).unwrap();
        let hash_val = hash_digit.to_digit(16).unwrap();
        if hash_val >= 8 {
            result.push(ch.to_ascii_uppercase());
        } else {
            result.push(ch);
        }
    }
    result
}
