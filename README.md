# 🚀 CREATE3 Address Tool

A command-line tool written in Rust that deterministically generates Ethereum contract
addresses using the CREATE3 method. This tool computes contract addresses based on a
deployer address, a salt value, and a fixed proxy child bytecode. It also supports vanity
address generation as well as batch generation of multiple addresses.

## Table of Contents

- [Background](#background)
- [Features](#features)
- [How It Works](#how-it-works)
- [Installation](#installation)
- [Usage](#usage)
- [Examples](#examples)
- [References](#references)
- [License](#license)

## Background

Ethereum contracts are typically deployed using the `CREATE` or `CREATE2` opcodes.
The CREATE3 method extends these concepts by allowing you to precompute a contract's
address before deployment. The computed address depends on the deployer address, a salt,
and a fixed proxy child bytecode.

**Key Concepts:**

- **Deployer Address:**  
  The address that deploys the contract. It ensures that the same salt and code
  produce different addresses when used by different deployers.

- **Salt:**  
  A value that, when combined with the deployer address and proxy bytecode, produces a
  deterministic contract address. The salt can be provided manually or generated randomly
  (with an optional user-specified prefix).

- **Proxy Bytecode:**  
  A fixed bytecode used during the computation. While it does not affect the contract's
  logic, it is essential for deterministic address calculation.

- **Vanity Addresses:**  
  Addresses that begin with a desired hexadecimal prefix. The tool searches for a salt
  that yields an address starting with the specified prefix.

## Features

- **Deterministic Address Calculation:**  
  Compute contract addresses based on a deployer address and a salt.

- **Vanity Address Generation:**  
  Generate salts that produce addresses with a specified prefix.

- **Batch Generation:**  
  Produce multiple salts and corresponding vanity addresses for a single deployer
  and prefix.

- **Multithreaded Salt Generation:**  
  Improve performance by utilizing multiple threads for salt generation.

## How It Works

1. **Input Collection:**  
   The tool prompts you for a deployer address (without the `0x` prefix) and either a
   salt or a desired address prefix.

2. **Salt Generation and Validation:**  
   If a prefix is provided, the tool generates random salts and computes their keccak256
   digest until it finds one that results in an address beginning with the specified prefix.

3. **Address Calculation:**  
   The contract address is computed by concatenating a constant prefix byte, the deployer
   address, the 32-byte salt, and the proxy child bytecode. The result is then hashed using
   keccak256 and processed with an rlp-like encoding method to produce the final address.

4. **Output:**  
   The computed contract address is displayed in checksummed format along with the generated
   salt and its hashed version.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)

### Steps

1. **Clone the Repository Using SSH:**

   ```bash
   git clone git@github.com:jaredborders/create3.git
   cd create3
   ```

2. **Build the Project:**

   ```bash
   cargo build --release
   ```

3. **Run the Tool:**

   ```bash
   cargo run --release
   ```

## Usage

When you run the tool, you will be presented with a menu offering the following options:

1. **Generate a CREATE3 Address:**  
   Provide a deployer address and a salt to compute a deterministic contract address.

2. **Generate a Salt for a Vanity Address:**  
   Provide a deployer address and a desired prefix. The tool will generate a salt such
   that the resulting address begins with the given prefix.

3. **Generate an Optimized Salt with a Salt Prefix:**  
   Similar to option 2, but allows you to prepend an additional salt prefix to the random
   portion.

4. **Batch Generate Multiple Vanity Addresses:**  
   Provide a deployer address, a desired prefix, and the number of addresses to generate.
   The tool will output multiple salt/address pairs that meet the specified criteria.

**Note:** When entering addresses, do not include the `0x` prefix.

## Examples

### Example 1: Single Address Generation

```bash
enter your choice (1/2/3/4): 1
enter deployer address: 0xDe2E5D408865Ec68a49AE5c0AdDa7f0Ebf0343B5
enter salt (utf8): nacl
create3 address: 0x8b9A192B07bb8de5615545C620738c2713B97D4d
```

### Example 2: Vanity Address Generation

```bash
enter your choice (1/2/3/4): 2
enter deployer address: 0x8b9A192B07bb8de5615545C620738c2713B97D4d
enter prefix (without '0x' prefix): 99999
vanity address: 0x99999A1dC707CB0C8eDd97Fe523ea960ECE326B0
salt string: owWgMkdoTK
hashed salt for prefix 0000: 0x036aa49cb041f17413cc0729f166092f0018b7290fa06c849edcaaf95e5558b1
```

### Example 3: Batch Vanity Address Generation

```bash
enter your choice (1/2/3/4): 4
enter deployer address: 0x99999A1dC707CB0C8eDd97Fe523ea960ECE326B0
enter prefix (without '0x' prefix): AAA
enter number of addresses to generate: 3
result 1:
  salt string: opOVXK2sYh
  vanity address: 0xAAA3e02D8F10662C7900462901D721f726457287
  hashed salt for prefix AAA: 0x07dceac94c4bdc90fcdc5cda72bd336896b2c445113add0de328987d6382c7e2
result 2:
  salt string: C649cDzLzn
  vanity address: 0xAaA9AA1b432c9f63ED6C7901C4Db92eD6B1e464b
  hashed salt for prefix AAA: 0xe6f1c3dba99c67b04eae00bbd16e7240cbdd62cd28c078d52b097d6bada98506
result 3:
  salt string: 51QLrBqUdT
  vanity address: 0xaaaCDdf0147A973933C41Bb601038F3a7DC159b8
  hashed salt for prefix AAA: 0x007ab8f0003ec7352092d7556c86d9c43a7dd58b2f6b17b716bae2c922140bc8
```

## References

- **CREATE3 & Deterministic Contract Addresses:**

  - [Ethereum StackExchange on Deterministic Contract Addresses](https://ethereum.stackexchange.com/)
  - [Solady CREATE3 Implementation](https://github.com/Vectorized/solady/blob/main/src/utils/CREATE3.sol)

- **Keccak256 & Hashing:**

  - [Keccak Hash Function](https://keccak.team/)
  - [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf)

- **RLP Encoding:**

  - [RLP Explained](https://eth.wiki/fundamentals/rlp)

- **Rust Cryptography Libraries:**
  - [Rust Crypto on crates.io](https://crates.io/keywords/cryptography)

---

_Happy coding!_
