# MemeCoin - ERC20 Token on ink!

A fully-featured ERC20-compatible memecoin smart contract built with ink! for Polkadot and Substrate-based chains.

## Example Interaction

Contract Address: `0x89f3DdAbb76103470BaBe31E5157bbF60A37c86C`

![test name function](/assets/image-i.png)

![test total supply function](/assets/image-ii.png)

## Features

### ✅ ERC20 Standard Compliance

- **Total Supply Tracking**: Monitor the total supply of tokens
- **Balance Management**: Check balances for any account
- **Transfer**: Send tokens between accounts
- **Approve & Allowance**: Delegate token spending to other accounts/contracts
- **Transfer From**: Allow approved accounts to transfer on behalf of others
- **Increase/Decrease Allowance**: Granular control over spending permissions

### 🔥 Extended Functionality

- **Mint**: Create new tokens (increases supply)
- **Burn**: Destroy tokens (decreases supply)
- **Metadata**: Token name ("MemeCoin"), symbol ("MEME"), and decimals (18)
- **Events**: `Transfer` and `Approval` events for tracking operations
- **Smart Contract Interoperability**: All functions are exposed via `#[ink(message)]` allowing other smart contracts to interact with this token

## Contract Architecture

### Storage

```rust
pub struct PspCoin {
    total_supply: U256,                         // Total token supply
    balances: Mapping<H160, U256>,              // Account balances
    allowances: Mapping<(H160, H160), U256>,    // Spending allowances (owner, spender)
    metadata: (String, String, u8),             // (name, symbol, decimals)
}
```

### Constructors

#### `new()`

Creates a new token with zero initial supply.

```rust
let token = PspCoin::new();
```

#### `new_with_supply(total_supply: U256)`

Creates a new token with an initial supply allocated to the deployer.

```rust
let initial_supply = U256::from(1_000_000_000u128) * U256::from(10u128).pow(U256::from(18u8));
let token = PspCoin::new_with_supply(initial_supply);
```

## Standard Functions

### Read-Only Functions

#### `total_supply() -> U256`

Returns the total token supply.

#### `balance_of(owner: H160) -> U256`

Returns the token balance of an account.

#### `allowance(owner: H160, spender: H160) -> U256`

Returns the remaining tokens that `spender` is allowed to spend on behalf of `owner`.

#### `name() -> Option<String>`

Returns the token name: "MemeCoin"

#### `symbol() -> Option<String>`

Returns the token symbol: "MEME"

#### `decimals() -> u8`

Returns the number of decimals: 18

### State-Changing Functions

#### `transfer(to: H160, value: U256, data: Vec<u8>) -> Result<(), PSP22Error>`

Transfers `value` tokens from the caller to `to`.

- Emits `Transfer` event
- Returns `InsufficientBalance` error if insufficient funds
- Returns `Overflow` error if recipient balance would overflow

#### `transfer_from(from: H160, to: H160, value: U256, data: Vec<u8>) -> Result<(), PSP22Error>`

Transfers `value` tokens from `from` to `to` using the allowance mechanism.

- Requires approval if caller is not `from`
- Emits `Transfer` and `Approval` events
- Returns `InsufficientBalance` or `InsufficientAllowance` errors

#### `approve(spender: H160, value: U256) -> Result<(), PSP22Error>`

Approves `spender` to spend `value` tokens on behalf of the caller.

- Emits `Approval` event
- Overwrites previous allowance

#### `increase_allowance(spender: H160, delta_value: U256) -> Result<(), PSP22Error>`

Increases the allowance granted to `spender` by `delta_value`.

- Emits `Approval` event with new allowance

#### `decrease_allowance(spender: H160, delta_value: U256) -> Result<(), PSP22Error>`

Decreases the allowance granted to `spender` by `delta_value`.

- Emits `Approval` event with new allowance
- Returns `InsufficientAllowance` if trying to decrease below zero

### Extended Functions

#### `mint(value: U256) -> Result<(), PSP22Error>`

Mints `value` new tokens to the caller's account.

- Increases total supply
- Emits `Transfer` event with `from: None`
- Returns `Overflow` error if supply would overflow

#### `burn(value: U256) -> Result<(), PSP22Error>`

Burns `value` tokens from the caller's account.

- Decreases total supply
- Emits `Transfer` event with `to: None`
- Returns `InsufficientBalance` if insufficient funds

## Smart Contract Interoperability

All functions are marked with `#[ink(message)]`, making them callable from other smart contracts. This allows:

1. **DeFi Integration**: Other contracts can interact with this token for DeFi protocols (DEXs, lending, staking)
2. **Cross-Contract Calls**: Smart contracts can check balances, transfer tokens, and manage allowances
3. **Composability**: Build complex systems on top of this token

### Example: Calling from Another Contract

```rust
#[ink::contract]
mod my_dapp {
    use psp_coin::psp_coin::PspCoinRef;

    #[ink(storage)]
    pub struct MyDapp {
        token: PspCoinRef,
    }

    impl MyDapp {
        #[ink(message)]
        pub fn check_my_balance(&self) -> U256 {
            let caller = self.env().caller();
            let caller_h160 = /* convert to H160 */;
            self.token.balance_of(caller_h160)
        }

        #[ink(message)]
        pub fn send_tokens(&mut self, to: H160, amount: U256) -> Result<(), PSP22Error> {
            self.token.transfer(to, amount, vec![])
        }
    }
}
```

## Events

### Transfer

```rust
pub struct Transfer {
    from: Option<H160>,    // None for minting
    to: Option<H160>,      // None for burning
    value: U256,
}
```

### Approval

```rust
pub struct Approval {
    owner: H160,
    spender: H160,
    value: U256,
}
```

## Error Types

```rust
pub enum PSP22Error {
    InsufficientBalance,    // Not enough tokens in account
    InsufficientAllowance,  // Not enough allowance granted
    Overflow,               // Arithmetic overflow would occur
    Custom(String),         // Custom error message
}
```

## Building the Contract

```bash
# Build the contract
cargo contract build

# Run tests
cargo test

# Deploy to a local node
cargo contract instantiate \
    --constructor new_with_supply \
    --args "1000000000000000000000000" \
    --suri //Alice
```

## Security Features

1. **Overflow Protection**: All arithmetic operations use checked math
2. **No-op Optimizations**: Zero-value transfers and self-transfers are no-ops
3. **Allowance Checks**: Transfer-from requires proper allowance
4. **Balance Validation**: All transfers check for sufficient balance
5. **Event Emission**: All state changes emit appropriate events

## Testing

The contract includes comprehensive error handling and edge case management:

- Zero-value transfers (no-ops)
- Self-transfers (no-ops)
- Overflow protection on mint and transfer
- Underflow protection on burn and decrease allowance
- Proper allowance management in transfer_from

## PSP22 Standard

This implementation follows the PSP22 token standard (Polkadot Standard Proposal), which is similar to ERC20 but adapted for the Polkadot ecosystem with H160 addresses and U256 balances.
