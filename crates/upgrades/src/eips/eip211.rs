//! EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY.
//!
//! ## Simple Summary
//!
//! A mechanism to allow returning arbitrary-length data inside the EVM has been requested for quite a while now. Existing proposals always had very intricate problems associated with charging gas. This proposal solves the same problem while at the same time, it has a very simple gas charging mechanism and requires minimal changes to the call opcodes. Its workings are very similar to the way calldata is handled already; after a call, return data is kept inside a virtual buffer from which the caller can copy it (or parts thereof) into memory. At the next call, the buffer is overwritten. This mechanism is 100% backwards compatible.
//!
//! ## Abstract
//!
//! Please see summary.
//!
//! ## Motivation
//!
//! In some situations, it is vital for a function to be able to return data whose length cannot be anticipated before the call. In principle, this can be solved without alterations to the EVM, for example by splitting the call into two calls where the first is used to compute only the size. All of these mechanisms, though, are very expensive in at least some situations. A very useful example of such a worst-case situation is a generic forwarding contract; a contract that takes call data, potentially makes some checks and then forwards it as is to another contract. The return data should of course be transferred in a similar way to the original caller. Since the contract is generic and does not know about the contract it calls, there is no way to determine the size of the output without adapting the called contract accordingly or trying a logarithmic number of calls.
//!
//! Compiler implementors are advised to reserve a zero-length area for return data if the size of the return data is unknown before the call and then use `RETURNDATACOPY` in conjunction with `RETURNDATASIZE` to actually retrieve the data.
//!
//! Note that this proposal also makes the EIP that proposes to allow to return data in case of an intentional state reversion ([EIP-140](./eip-140.md)) much more useful. Since the size of the failure data might be larger than the regular return data (or even unknown), it is possible to retrieve the failure data after the CALL opcode has signalled a failure, even if the regular output area is not large enough to hold the data.
//!
//! ## Specification
//!
//! If `block.number >= BYZANTIUM_FORK_BLKNUM`, add two new opcodes and amend the semantics of any opcode that creates a new call frame (like `CALL`, `CREATE`, `DELEGATECALL`, ...) called call-like opcodes in the following. It is assumed that the EVM (to be more specific: an EVM call frame) has a new internal buffer of variable size, called the return data buffer. This buffer is created empty for each new call frame. Upon executing any call-like opcode, the buffer is cleared (its size is set to zero). After executing a call-like opcode, the complete return data (or failure data, see [EIP-140](./eip-140.md)) of the call is stored in the return data buffer (of the caller), and its size changed accordingly. As an exception, `CREATE` and `CREATE2` are considered to return the empty buffer in the success case and the failure data in the failure case. If the call-like opcode is executed but does not really instantiate a call frame (for example due to insufficient funds for a value transfer or if the called contract does not exist), the return data buffer is empty.
//!
//! As an optimization, it is possible to share the return data buffer across call frames because at most one will be non-empty at any time.
//!
//! `RETURNDATASIZE`: `0x3d`
//!
//! Pushes the size of the return data buffer onto the stack.
//! Gas costs: 2 (same as `CALLDATASIZE`)
//!
//! `RETURNDATACOPY`: `0x3e`
//!
//! This opcode has similar semantics to `CALLDATACOPY`, but instead of copying data from the call data, it copies data from the return data buffer. Furthermore, accessing the return data buffer beyond its size results in a failure; i.e. if `start + length` overflows or results in a value larger than `RETURNDATASIZE`, the current call stops in an out-of-gas condition. In particular, reading 0 bytes from the end of the buffer will read 0 bytes; reading 0 bytes from one-byte out of the buffer causes an exception.
//!
//! Gas costs: `3 + 3 * ceil(amount / 32)` (same as `CALLDATACOPY`)
//!
//! ## Rationale
//!
//! Other solutions that would allow returning dynamic data were considered, but they all had to deduct the gas from the call opcode and thus were both complicated to implement and specify ([5/8](https://github.com/ethereum/EIPs/issues/8)). Since this proposal is very similar to the way calldata is handled, it fits nicely into the concept. Furthermore, the eWASM architecture already handles return data in exactly the same way.
//!
//! Note that the EVM implementation needs to keep the return data until the next call or the return from the current call. Since this resource was already paid for as part of the memory of the callee, it should not be a problem. Implementations may either choose to keep the full memory of the callee alive until the next call or copy only the return data to a special memory area.
//!
//! Keeping the memory of the callee until the next call-like opcode does not increase the peak memory usage in the following sense; any memory allocation in the caller's frame that happens after the return from the call can be moved before the call without a change in gas costs, but will add this allocation to the peak allocation.
//!
//! The number values of the opcodes were allocated in the same nibble block that also contains `CALLDATASIZE` and `CALLDATACOPY`.
//!
//! ## Backwards Compatibility
//!
//! This proposal introduces two new opcodes and stays fully backwards compatible apart from that.
//!
//! Christian Reitwiessner <chris@ethereum.org>, "EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY," Ethereum Improvement Proposals, no. 211, February 2017. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-211>.

use asm::instruction::{ReturnDataCopy, ReturnDataSize};

use crate::eip::{Eip, macros::introduces_instructions};

/// EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY.
pub struct Eip211;

impl Eip for Eip211 {
    const NUMBER: u32 = 211;
}

introduces_instructions!(Eip211, ReturnDataSize, ReturnDataCopy);
