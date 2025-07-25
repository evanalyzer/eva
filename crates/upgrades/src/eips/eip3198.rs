//! EIP-3198: BASEFEE opcode.
//!
//! ## Simple Summary
//! Adds an opcode that gives the EVM access to the block's base fee.
//!
//! ## Abstract
//!
//! Add a `BASEFEE (0x48)` that returns the value of the base fee of the current block it is executing in.
//!
//! ## Motivation
//! The intended use case would be for contracts to get the value of the base fee. This feature would enable or improve existing use cases, such as:
//! - Contracts that need to set bounties for anyone to "poke" them with a transaction could set the bounty to be `BASEFEE + x`, or `BASEFEE * (1 + x)`. This makes the mechanism more reliable, because they will always pay "enough" regardless of market conditions.
//! - Gas futures can be implemented based on it. This would be more precise than gastokens.
//! - Improve the security for state channels, plasma, optirolls and other fraud proof driven solutions. Having the `BASEFEE` as an input allows you to lengthen the challenge period automatically if you see that the `BASEFEE` is high.
//!
//! ## Specification
//! Add a `BASEFEE` opcode at `(0x48)`, with gas cost `G_base`.
//!
//! ```python
//! |  Op      | Input     | Output     | Cost     |
//! |:----:    |:-----:    |:------:    |:----:    |
//! | 0x48     |   0       |    1       |   2      |
//! ```
//!
//! ## Rationale
//!
//! ### Gas cost
//! The value of the base fee is needed to process transactions. That means it's value is already available before running the EVM code.
//! The opcode does not add extra complexity and additional read/write operations, hence the choice of `G_base` gas cost.
//!
//! ## Backwards Compatibility
//! There are no known backward compatibility issues with this opcode.
//!
//! ## Test Cases
//!
//! ### Nominal case
//! Assuming current block base fee is `7 wei`.
//! This should push the value `7` (left padded byte32) to the stack.
//!
//! Bytecode: `0x4800` (`BASEFEE, STOP`)
//! ```python
//! |  Pc   |      Op     | Cost |   Stack   |   `RStack`  |
//! |-------|-------------|------|-----------|-----------|
//! |    0  |    BASEFEE  |    2 |        [] |        [] |
//! |    1  |    STOP     |    0 |       [7] |        [] |
//! ```
//! Output: 0x
//! Consumed gas: `2`
//!
//! ## Security Considerations
//! The value of the base fee is not sensitive and is publicly accessible in the block header. There are no known security implications with this opcode.
//!
//! Abdelhamid Bakhta (@abdelhamidbakhta), Vitalik Buterin (@vbuterin), "EIP-3198: BASEFEE opcode," Ethereum Improvement Proposals, no. 3198, January 2021. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-3198>.

use asm::instruction::BaseFee;

use crate::eip::{Eip, macros::introduces_instructions};

/// EIP-3198: BASEFEE opcode.
pub struct Eip3198;

impl Eip for Eip3198 {
    const NUMBER: u32 = 3198;
}

introduces_instructions!(Eip3198, BaseFee);
