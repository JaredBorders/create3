pub mod errors;

use std::sync::{Arc, RwLock};
use std::thread;

use errors::Create3GenerateSaltError;
use rand::{distributions::Alphanumeric, Rng};
use sha3::{Digest, Keccak256};

// proxy child bytecode; deployed bytecode does not affect the address.
const KECCAK256_PROXY_CHILD_BYTECODE: [u8; 32] = [
    33, 195, 93, 190, 27, 52, 74, 36, 136, 207, 51, 33, 214, 206, 84, 47, 142,
    159, 48, 85, 68, 255, 9, 228, 153, 58, 98, 49, 154, 73, 124, 31,
];

/// calculates the address of a contract using the deployer and salt.
///
/// arguments:
/// - deployer: create3 deployer address (bytes).
/// - salt: arbitrary bytes; its keccak256 digest is used.
///
/// returns: a 20-byte contract address.
pub fn calc_addr(deployer: &[u8], salt: &[u8]) -> [u8; 20] {
    let salt_hash = Keccak256::digest(salt);
    calc_addr_with_bytes(
        deployer,
        &salt_hash.as_slice()[0..32].try_into().unwrap(),
    )
}

/// calculates the address of a contract using deployer and salt bytes.
///
/// arguments:
/// - deployer: create3 deployer address (bytes).
/// - salt: 32-byte value used as the salt.
///
/// returns: a 20-byte contract address.
pub fn calc_addr_with_bytes(deployer: &[u8], salt: &[u8; 32]) -> [u8; 20] {
    let mut bytes = Vec::new();
    bytes.push(0xff);
    bytes.extend_from_slice(deployer);
    bytes.extend_from_slice(salt);
    bytes.extend_from_slice(&KECCAK256_PROXY_CHILD_BYTECODE);

    let hash = Keccak256::digest(&bytes);
    let mut proxy_bytes = [0u8; 20];
    proxy_bytes.copy_from_slice(&hash[12..]);

    // compute final address using proxy address and nonce 0x01.
    let mut bytes2 = Vec::new();
    bytes2.extend_from_slice(&[0xd6, 0x94]);
    bytes2.extend(&proxy_bytes);
    bytes2.push(0x01);
    let hash2 = Keccak256::digest(&bytes2);

    let mut address = [0u8; 20];
    address.copy_from_slice(&hash2[12..]);
    address
}

/// cleans and validates the prefix for salt generation.
///
/// returns: a lowercase version of the prefix if valid.
fn sanitize_prefix(prefix: &str) -> Result<String, Create3GenerateSaltError> {
    let prefix = prefix.trim();
    if prefix.len() > 20 {
        return Err(Create3GenerateSaltError::PrefixTooLong);
    } else if !prefix.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Create3GenerateSaltError::PrefixNotHexEncoded);
    }
    Ok(prefix.to_lowercase())
}

/// generates a random salt for a deployer and a given prefix.
///
/// arguments:
/// - deployer: create3 deployer address (bytes).
/// - prefix: desired address prefix (without '0x').
///
/// returns: (salt string, 32-byte keccak256 digest of salt).
pub fn generate_salt(
    deployer: &[u8],
    prefix: &str,
) -> Result<(String, [u8; 32]), Create3GenerateSaltError> {
    let mut salt_bytes = [0; 32];
    let mut salt: String;
    let prefix = sanitize_prefix(prefix)?;
    loop {
        salt = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let vanity_addr = calc_addr(deployer, salt.as_bytes());
        let vanity_addr = hex::encode(&vanity_addr);
        if vanity_addr.starts_with(&prefix) {
            let salt_hex = hex::encode(Keccak256::digest(salt.clone()));
            let salt_bytes_slice = hex::decode(&salt_hex).unwrap();
            salt_bytes.copy_from_slice(&salt_bytes_slice);
            break;
        }
    }
    Ok((salt, salt_bytes))
}

/// generates a random salt using multiple threads for a given prefix.
///
/// arguments:
/// - deployer: create3 deployer address (bytes).
/// - prefix: desired address prefix (without '0x').
/// - thread_count: number of threads to spawn.
///
/// returns: (salt string, 32-byte keccak256 digest of salt).
pub fn generate_salt_multithread(
    deployer: &[u8],
    prefix: &str,
    thread_count: u8,
) -> Result<(String, [u8; 32]), Create3GenerateSaltError> {
    generate_salt_prefix_multithread(deployer, "", prefix, thread_count)
}

/// generates a salt with a salt prefix for a given address prefix.
///
/// arguments:
/// - deployer: create3 deployer address (bytes).
/// - salt_prefix: string to append to the random salt.
/// - prefix: desired address prefix (without '0x').
///
/// returns: (salt string, 32-byte keccak256 digest of salt).
pub fn generate_salt_prefix(
    deployer: &[u8],
    salt_prefix: &str,
    prefix: &str,
) -> Result<(String, [u8; 32]), Create3GenerateSaltError> {
    let mut salt_bytes = [0; 32];
    let mut salt: String;
    let prefix = sanitize_prefix(prefix)?;
    loop {
        salt = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        salt = salt_prefix.to_owned() + &salt;
        let vanity_addr = calc_addr(deployer, salt.as_bytes());
        let vanity_addr = hex::encode(&vanity_addr);
        if vanity_addr.starts_with(&prefix) {
            let salt_hex = hex::encode(Keccak256::digest(salt.clone()));
            let salt_bytes_slice = hex::decode(&salt_hex).unwrap();
            salt_bytes.copy_from_slice(&salt_bytes_slice);
            break;
        }
    }
    Ok((salt, salt_bytes))
}

/// generates a salt with a salt prefix using multiple threads.
///
/// arguments:
/// - deployer: create3 deployer address (bytes).
/// - salt_prefix: string to append to the random salt.
/// - prefix: desired address prefix (without '0x').
/// - thread_count: number of threads to spawn.
///
/// returns: (salt string, 32-byte keccak256 digest of salt).
pub fn generate_salt_prefix_multithread(
    deployer: &[u8],
    salt_prefix: &str,
    prefix: &str,
    thread_count: u8,
) -> Result<(String, [u8; 32]), Create3GenerateSaltError> {
    let lock: Arc<RwLock<(String, [u8; 32])>> =
        Arc::new(RwLock::new(("".to_owned(), [0; 32])));
    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
    let prefix = sanitize_prefix(prefix)?;
    for _ in 0..thread_count {
        let p = prefix.to_owned();
        let d = deployer.to_owned();
        let sp = salt_prefix.to_owned();
        let lock = lock.clone();
        let handle = thread::spawn(move || {
            let mut salt: String;
            let mut salt_bytes = [0; 32];
            loop {
                salt = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(7)
                    .map(char::from)
                    .collect();
                salt = sp.to_owned() + &salt;
                let vanity_addr = calc_addr(&d, salt.as_bytes());
                let vanity_addr = hex::encode(&vanity_addr);
                let Ok(read_lock) = lock.try_read() else {
                    break;
                };
                if read_lock.0.len() > 0 {
                    break;
                }
                if !vanity_addr.starts_with(&p) {
                    continue;
                }
                drop(read_lock);
                let mut write_lock = lock.write().unwrap();
                let salt_hex = hex::encode(Keccak256::digest(salt.clone()));
                let salt_bytes_slice = hex::decode(&salt_hex).unwrap();
                salt_bytes.copy_from_slice(&salt_bytes_slice);
                *write_lock = (salt, salt_bytes);
                drop(write_lock);
                break;
            }
        });
        threads.push(handle);
    }
    for t in threads {
        t.join().unwrap();
    }
    let read_lock = lock.read().unwrap();
    Ok((read_lock.0.clone(), read_lock.1.clone()))
}

#[cfg(test)]
mod tests {
    use crate::{
        calc_addr, calc_addr_with_bytes, generate_salt,
        generate_salt_multithread, generate_salt_prefix,
        generate_salt_prefix_multithread, Create3GenerateSaltError,
    };
    use sha3::{Digest, Keccak256};

    #[test]
    fn should_calculate_correctly_with_given_salt_string() {
        let deployer: Vec<u8> =
            hex::decode("0fC5025C764cE34df352757e82f7B5c4Df39A836").unwrap();
        let correct_answers: Vec<(&str, &str)> = vec![
            ("a", "BFf47440D3A5E59714F1D995F8b105E2a04AB46A"),
            ("b", "7E10Ca8fa1c8e1528601Fea82F51646182f835b8"),
            ("c", "70b556548FF0161082fB751d5E372eFa0133805C"),
            (
                "some say the world will end in fire, some say in ice. from what \
i’ve tasted of desire i hold with those who favor fire. but if it had to \
perish twice, i think i know enough of hate to say that for destruction \
ice is also great and would suffice.",
                "C244c5dEa48e677cE7cAbD05BF8eC220b1a99Fc9"
            )
        ];
        for (salt, answer) in correct_answers.iter() {
            let addr: [u8; 20] = calc_addr(&deployer, salt.as_bytes());
            let addr_str = hex::encode(addr);
            assert_eq!(addr_str, answer.to_lowercase());
        }
    }

    #[test]
    fn should_calculate_correctly_with_given_salt() {
        let deployer =
            hex::decode("d8b934580fcE35a11B58C6D73aDeE468a2833fa8").unwrap();
        let correct_answers: Vec<(&str, &str)> = vec![
            (
                "3ac225168df54212a25c1c01fd35bebfea408fdac2e31ddd6f80a4bbf9a5f1cb",
                "442188F25da4ac213D55aE81F1BFB421a4eb4562",
            ),
            (
                "b5553de315e0edf504d9150af82dafa5c4667fa618ed0a6f19c69b41166c5510",
                "551b9d8A7106Fdf98e68c4bf12Da1f23ad70C815",
            ),
            (
                "0b42b6393c1f53060fe3ddbfcd7aadcca894465a5a438f69c87d790b2299b9b2",
                "43d8e8C69fd771f7D3F4e25697Dadd3cC11D1cDB",
            ),
            (
                "ead17456afde832907c72ba39033455130a8f4d540a869ba31312c2746bf9c4b",
                "AB3D55404C5C21D18403A71aF5f6887BD0EC8d56",
            ),
        ];
        for (salt, answer) in correct_answers.iter() {
            let salt: [u8; 32] =
                hex::decode(*salt).unwrap()[0..32].try_into().unwrap();
            let addr: [u8; 20] = calc_addr_with_bytes(&deployer, &salt);
            let addr_str = hex::encode(addr);
            assert_eq!(addr_str, answer.to_lowercase());
        }
    }

    #[test]
    fn should_generate_with_prefix() {
        let deployer: Vec<u8> =
            hex::decode("5e17b14ADd6c386305A32928F985b29bbA34Eff5").unwrap();
        let runs = vec!["0", "00", "000", "abc", "123", "789", "Def"];
        for run in runs.iter() {
            let salt = generate_salt(&deployer, run).unwrap();
            let addr: [u8; 20] = calc_addr_with_bytes(&deployer, &salt.1);
            let addr_string = calc_addr(&deployer, salt.0.as_bytes());
            assert_eq!(addr, addr_string);
            assert!(hex::encode(addr).starts_with(&run.to_lowercase()));
        }
    }

    #[test]
    fn should_generate_multithread_with_prefix() {
        let deployer: Vec<u8> =
            hex::decode("5e17b14ADd6c386305A32928F985b29bbA34Eff5").unwrap();
        let runs = vec!["0", "00", "000", "abcd", "123", "789", "Def"];
        for run in runs.iter() {
            let salt = generate_salt_multithread(&deployer, run, 6).unwrap();
            let addr: [u8; 20] = calc_addr_with_bytes(&deployer, &salt.1);
            assert_eq!(calc_addr(&deployer, salt.0.as_bytes()), addr);
            assert!(hex::encode(addr).starts_with(&run.to_lowercase()));
        }
    }

    #[test]
    fn should_generate_with_empty_prefix() {
        let deployer: Vec<u8> =
            hex::decode("0fC5025C764cE34df352757e82f7B5c4Df39A836").unwrap();
        assert!(generate_salt(&deployer, "").is_ok());
    }

    #[test]
    fn should_generate_with_salt_prefix() {
        let deployer: Vec<u8> =
            hex::decode("5e17b14ADd6c386305A32928F985b29bbA34Eff5").unwrap();
        let runs = vec!["0", "00", "000", "abc", "123", "789", "Def"];
        let salt_prefix = "testpfx_";
        for run in runs.iter() {
            let (salt, digested_salt) =
                generate_salt_prefix(&deployer, salt_prefix, run).unwrap();
            assert!(salt.starts_with(&salt_prefix.to_lowercase()));
            assert_eq!(
                Keccak256::digest(salt).as_slice()[0..32],
                digested_salt
            );
            assert!(hex::encode(calc_addr_with_bytes(
                &deployer,
                &digested_salt
            ))
            .starts_with(&run.to_lowercase()));
        }
    }

    #[test]
    fn should_generate_multithread_with_salt_prefix() {
        let deployer: Vec<u8> =
            hex::decode("5e17b14ADd6c386305A32928F985b29bbA34Eff5").unwrap();
        let runs = vec!["0", "00", "000", "abc", "123", "789", "Def"];
        let salt_prefix = "testpfx_";
        for run in runs.iter() {
            let (salt, digested_salt) = generate_salt_prefix_multithread(
                &deployer,
                salt_prefix,
                run,
                6,
            )
            .unwrap();
            assert!(salt.starts_with(&salt_prefix.to_lowercase()));
            assert_eq!(
                Keccak256::digest(salt).as_slice()[0..32],
                digested_salt
            );
            assert!(hex::encode(calc_addr_with_bytes(
                &deployer,
                &digested_salt
            ))
            .starts_with(&run.to_lowercase()));
        }
    }

    #[test]
    fn generate_salt_should_error_if_prefix_is_greater_than_20_bytes() {
        let deployer = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".as_bytes();
        let prefix = "0x00000000000000000000000000000000000000000";
        assert_eq!(
            generate_salt(deployer, prefix),
            Err(Create3GenerateSaltError::PrefixTooLong)
        );
    }

    #[test]
    fn generate_salt_should_error_if_prefix_is_not_hex_encoded() {
        let deployer = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".as_bytes();
        let runs = vec!["hey", "abcg", "0x123", "Ab45[", "lightning mcqueen"];
        for run in runs.iter() {
            assert_eq!(
                generate_salt(deployer, run),
                Err(Create3GenerateSaltError::PrefixNotHexEncoded)
            );
        }
    }

    #[test]
    fn generate_salt_prefix_should_error_if_prefix_is_greater_than_20_bytes() {
        let deployer = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".as_bytes();
        let salt_prefix = "";
        let prefix = "0x00000000000000000000000000000000000000000";
        assert_eq!(
            generate_salt_prefix(deployer, salt_prefix, prefix),
            Err(Create3GenerateSaltError::PrefixTooLong)
        );
    }

    #[test]
    fn generate_salt_prefix_should_error_if_prefix_is_not_hex_encoded() {
        let deployer = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".as_bytes();
        let salt_prefix = "";
        let runs = vec!["hey", "abcg", "0x123", "Ab45[", "lightning mcqueen"];
        for run in runs.iter() {
            assert_eq!(
                generate_salt_prefix(deployer, salt_prefix, run),
                Err(Create3GenerateSaltError::PrefixNotHexEncoded)
            );
        }
    }
}
