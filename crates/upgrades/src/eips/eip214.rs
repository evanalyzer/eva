//! EIP-214: New opcode STATICCALL.
//!
//! ## Simple Summary
//!
//! To increase smart contract security, this proposal adds a new opcode that can be used to call another contract (or itself) while disallowing any modifications to the state during the call (and its subcalls, if present).
//!
//! ## Abstract
//!
//! This proposal adds a new opcode that can be used to call another contract (or itself) while disallowing any modifications to the state during the call (and its subcalls, if present). Any opcode that attempts to perform such a modification (see below for details) will result in an exception instead of performing the modification.
//!
//! ## Motivation
//!
//! Currently, there is no restriction about what a called contract can do, as long as the computation can be performed with the amount of gas provided. This poses certain difficulties about smart contract engineers; after a regular call, unless you know the called contract, you cannot make any assumptions about the state of the contracts. Furthermore, because you cannot know the order of transactions before they are confirmed by miners, not even an outside observer can be sure about that in all cases.
//!
//! This EIP adds a way to call other contracts and restrict what they can do in the simplest way. It can be safely assumed that the state of all accounts is the same before and after a static call.
//!
//! ## Specification
//!
//! Introduce a new `STATIC` flag to the virtual machine. This flag is set to `false` initially. Its value is always copied to sub-calls with an exception for the new opcode below.
//!
//! Opcode: `0xfa`.
//!
//! `STATICCALL` functions equivalently to a `CALL`, except it takes only 6 arguments (the "value" argument is not included and taken to be zero), and calls the child with the `STATIC` flag set to `true` for the execution of the child. Once this call returns, the flag is reset to its value before the call.
//!
//! Any attempts to make state-changing operations inside an execution instance with `STATIC` set to `true` will instead throw an exception. These operations include `CREATE`, `CREATE2`, `LOG0`, `LOG1`, `LOG2`, `LOG3`, `LOG4`, `SSTORE`, and `SELFDESTRUCT`. They also include `CALL` with a non-zero value. As an exception, `CALLCODE` is not considered state-changing, even with a non-zero value.
//!
//! ## Rationale
//!
//! This allows contracts to make calls that are clearly non-state-changing, reassuring developers and reviewers that re-entrancy bugs or other problems cannot possibly arise from that particular call; it is a pure function that returns an output and does nothing else. This may also make purely functional HLLs easier to implement.
//!
//! ## Backwards Compatibility
//!
//! This proposal adds a new opcode but does not modify the behaviour of other opcodes and thus is backwards compatible for old contracts that do not use the new opcode and are not called via the new opcode.
//!
//! Vitalik Buterin <vitalik@ethereum.org>, Christian Reitwiessner <chris@ethereum.org>, "EIP-214: New opcode STATICCALL," Ethereum Improvement Proposals, no. 214, February 2017. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-214>.

use asm::instruction::StaticCall;

use crate::eip::{Eip, macros::introduces_instructions};

/// EIP-214: New opcode STATICCALL.
pub struct Eip214;

impl Eip for Eip214 {
    const NUMBER: u32 = 214;
}

introduces_instructions!(Eip214, StaticCall);
