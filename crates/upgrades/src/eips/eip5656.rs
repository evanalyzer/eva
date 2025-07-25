//! EIP-5656: MCOPY - Memory copying instruction.
//!
//! ## Abstract
//!
//! Provide an efficient EVM instruction for copying memory areas.
//!
//! ## Motivation
//!
//! Memory copying is a basic operation, yet implementing it on the EVM comes with overhead.
//!
//! This was recognised and alleviated early on with the introduction of the "identity" precompile, which accomplishes
//! memory copying by the use of `CALL`'s input and output memory offsets. Its cost is `15 + 3 * (length / 32)` gas, plus
//! the call overhead. The identity precompile was rendered ineffective by the raise of the cost of `CALL` to 700, but subsequently
//! the reduction by [EIP-2929](./eip-2929.md) made it slightly more economical.
//!
//! Copying exact words can be accomplished with `<offset> MLOAD <offset> MSTORE` or `<offset> DUP1 MLOAD DUP2 MSTORE`,
//! at a cost of at least 12 gas per word. This is fairly efficient if the offsets are known upfront and the copying can be unrolled.
//! In case copying is implemented at runtime with arbitrary starting offsets, besides the control flow overhead, the offset
//! will need to be incremented using `32 ADD`, adding at least 6 gas per word.
//!
//! Copying non-exact words is more tricky, as for the last partial word, both the source and destination needs to be loaded,
//! masked, or'd, and stored again. This overhead is significant. One edge case is if the last "partial word" is a single byte,
//! it can be efficiently stored using `MSTORE8`.
//!
//! As example use case, copying 256 bytes costs:
//!
//! - at least 757 gas pre-EIP-2929 using the identity precompile
//! - at least 157 gas post-EIP-2929 using the identity precompile
//! - at least 96 gas using unrolled `MLOAD`/`MSTORE` instructions
//! - 27 gas using this EIP
//!
//! According to an analysis of blocks 10537502 to 10538702, roughly 10.5% of memory copies would have had improved performance with the
//! availability of an `MCOPY` instruction.
//!
//! Memory copying is used by languages like Solidity and Vyper, where we expect this improvement to provide efficient means of building
//! data structures, including efficient sliced access and copies of memory objects. Having a dedicated `MCOPY` instruction would also add
//! forward protection against future gas cost changes to `CALL` instructions in general.
//!
//! Having a special `MCOPY` instruction makes the job of static analyzers and optimizers easier, since the effects of a `CALL` in general
//! have to be fenced, whereas an `MCOPY` instruction would be known to only have memory effects. Even if special cases are added
//! for precompiles, a future hard fork could change `CALL` effects, and so any analysis of code using the identity precompile would only
//! be valid for a certain range of blocks.
//!
//! Finally, we expect memory copying to be immensely useful for various computationally heavy operations, such as EVM384,
//! where it is identified as a significant overhead.
//!
//! ## Specification
//!
//! The instruction `MCOPY` is introduced at `0x5E`.
//!
//! ### Input stack
//!
//! | Stack | Value |
//! |-------|-------|
//! | top - 0 | `dst` |
//! | top - 1 | `src` |
//! | top - 2 | `length` |
//!
//! This ordering matches the other copying instructions, i.e. `CALLDATACOPY`, `RETURNDATACOPY`.
//!
//! ### Gas costs
//!
//! Per yellow paper terminology, it should be considered part of the `W_copy` group of opcodes, and follow the gas calculation for `W_copy` in the yellow paper. While the calculation in the yellow paper should be considered the final word, for reference, as of time of this writing, that currently means its gas cost is:
//!
//! ```python
//! words_copied = (length + 31) // 32
//! g_verylow    = 3
//! g_copy       = 3 * words_copied + memory_expansion_cost
//! gas_cost     = g_verylow + g_copy
//! ```
//!
//! ### Output stack
//!
//! This instruction returns no stack items.
//!
//! ### Semantics
//!
//! It copies `length` bytes from the offset pointed at `src` to the offset pointed at `dst` in memory.
//! Copying takes place as if an intermediate buffer was used, allowing the destination and source to overlap.
//!
//! If `length > 0` and (`src + length` or `dst + length`) is beyond the current memory length, the memory is extended with respective gas cost applied.
//!
//! The gas cost of this instruction mirrors that of other `Wcopy` instructions and is `Gverylow + Gcopy * ceil(length / 32)`.
//!
//! ## Rationale
//!
//! Production implementation of exact-word memory copying and partial-word memory copying can be found in the Solidity, Vyper and Fe compilers.
//!
//! With [EIP-2929](./eip-2929.md) the call overhead using the identity precompile was reduced from 700 to 100 gas.
//! This is still prohibitive for making the precompile a reasonable alternative again.
//!
//! ## Backwards Compatibility
//!
//! This EIP introduces a new instruction which did not exist previously. Already deployed contracts using this instruction could change their behaviour after this EIP.
//!
//! ## Test Cases
//!
//! `MCOPY 0 32 32` - copy 32 bytes from offset 32 to offset 0.
//!
//! pre (spaces included for readability):
//!
//! ```python
//! 0000000000000000000000000000000000000000000000000000000000000000 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
//! ```
//!
//! post:
//!
//! ```python
//! 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
//! ```
//!
//! gas used: 6
//!
//! `MCOPY 0 0 32` - copy 32 bytes from offset 0 to offset 0.
//!
//! pre:
//!
//! ```python
//! 0101010101010101010101010101010101010101010101010101010101010101
//! ```
//!
//! post:
//!
//! ```python
//! 0101010101010101010101010101010101010101010101010101010101010101
//! ```
//!
//! gas used: 6
//!
//! `MCOPY 0 1 8` - copy 8 bytes from offset 1 to offset 0 (overlapping).
//!
//! pre (space at byte 8):
//!
//! ```python
//! 0001020304050607 080000000000000000000000000000000000000000000000
//! ```
//!
//! post:
//!
//! ```python
//! 0102030405060708 080000000000000000000000000000000000000000000000
//! ```
//!
//! gas used: 6
//!
//! `MCOPY 1 0 8` - copy 8 bytes from offset 0 to offset 1 (overlapping).
//!
//! pre (space at byte 8):
//!
//! ```python
//! 0001020304050607 080000000000000000000000000000000000000000000000
//! ```
//!
//! post:
//!
//! ```python
//! 0000010203040506 070000000000000000000000000000000000000000000000
//! ```
//!
//! gas used: 6
//!
//! ### Full test suite
//!
//! A full suite of tests can be found in the execution spec tests: [MCOPY suite](https://github.com/ethereum/execution-spec-tests/tree/c0065176a79f89d93f4c326186fc257ec5b8d5f1/tests/cancun/eip5656_mcopy).
//!
//! ## Security Considerations
//!
//! Clients should take care that their implementation does not use an intermediate buffer (see for instance that the C stdlib `memmove` function does not use an intermediate buffer), as this is a potential Denial of Service (`DoS`) vector. Most language builtins / standard library functions for moving bytes have the correct performance characteristics here.
//!
//! This aside, the analysis for Denial of Service (`DoS`) and memory exhaustion attacks is identical to other opcodes which touch memory, as the memory expansion follows the same pricing rules.
//!
//! Alex Beregszaszi (@axic), Paul Dworzanski (@poemm), Jared Wasinger (@jwasinger), Casey Detrio (@cdetrio), Pawel Bylica (@chfast), Charles Cooper (@charles-cooper), "EIP-5656: MCOPY - Memory copying instruction," Ethereum Improvement Proposals, no. 5656, February 2021. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-5656>.

use asm::instruction::MCopy;

use crate::eip::{Eip, macros::introduces_instructions};

/// EIP-5656: MCOPY - Memory copying instruction.
pub struct Eip5656;

impl Eip for Eip5656 {
    const NUMBER: u32 = 5656;
}

introduces_instructions!(Eip5656, MCopy);
