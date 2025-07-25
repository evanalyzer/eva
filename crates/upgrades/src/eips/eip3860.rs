//! EIP-3860: Limit and meter initcode.
//!
//! ## Abstract
//!
//! We extend EIP-170 by introducing a maximum size limit for `initcode` (`MAX_INITCODE_SIZE = 2 * MAX_CODE_SIZE = 49152`).
//!
//! Furthermore, we introduce a charge of `2` gas for every 32-byte chunk of `initcode` to represent the cost of jumpdest-analysis.
//!
//! Lastly, the size limit results in the nice-to-have property that EVM code size, code offset (`PC`), and jump offset fits a 16-bit value.
//!
//! ## Motivation
//!
//! During contract creation the client has to perform jumpdest-analysis on the `initcode` prior to execution. The work performed scales linearly with the size of the `initcode`. This work currently is not metered, nor is there a protocol enforced upper bound for the size.
//!
//! There are three costs charged today:
//!
//! 1. Cost for calldata aka `initcode`: 4 gas for a byte with the value of zero, and 16 gas otherwise.
//! 2. Cost for the resulting deployed code: 200 gas per byte.
//! 3. Cost of address calculation (hashing of code) in case of `CREATE2` only: 6 gas per word.
//!
//! Only the first cost applies to `initcode`, but only in the case of contract creation transactions. For the case of `CREATE`/`CREATE2` there is no such cost, and it is possible to programmatically generate variations of `initcode` in a relatively cheap manner. In the past it was possible to craft malicious `initcode` due to a vulnerability fixed in 2017 by geth 1.6.5.
//!
//! Furthermore, the lack of a limit has caused lengthy discussions for some EVM proposals, influencing the design, or even causing a delay or cancellation of a feature.
//!
//! We are motivated by three reasons:
//!
//! 1. Ensuring `initcode` is fairly charged (most importantly cost is proportional to `initcode`'s length) to minimize the risks for the future.
//! 2. To have a cost system which is extendable in the future.
//! 3. To simplify EVM engines by the explicit limits (code size, code offsets (`PC`), and jump offsets fit 16-bits).
//!
//! ## Specification
//!
//! ### Parameters
//!
//! | Constant             | Value               |
//! | -------------------- | ------------------- |
//! | `INITCODE_WORD_COST` | `2`                 |
//! | `MAX_INITCODE_SIZE`  | `2 * MAX_CODE_SIZE` |
//!
//! Where `MAX_CODE_SIZE` is defined by [EIP-170](./eip-170.md) as `24576`.
//!
//! We define `initcode_cost(initcode)` to equal `INITCODE_WORD_COST * ceil(len(initcode) / 32)`.
//!
//! ### Rules
//!
//! 1. If length of transaction data (`initcode`) in a create transaction exceeds `MAX_INITCODE_SIZE`, transaction is invalid. (*Note that this is similar to transactions considered invalid for not meeting the intrinsic gas cost requirement.*)
//! 2. For a create transaction, extend the transaction data cost formula to include `initcode_cost(initcode)`. (*Note that this is included in transaction intrinsic cost, i.e. transaction with not enough gas to cover initcode cost is invalid.*)
//! 3. If length of `initcode` to `CREATE` or `CREATE2` instructions exceeds `MAX_INITCODE_SIZE`, instruction execution exceptionally aborts (as if it runs out of gas).
//! 4. For the `CREATE` and `CREATE2` instructions charge an extra gas cost equaling to `initcode_cost(initcode)`. This cost is deducted before the calculation of the resulting contract address and the execution of `initcode`. (*Note that this means before or at the same time as the hashing cost is applied in `CREATE2`.*)
//!
//! ## Rationale
//!
//! ### Gas cost constant
//!
//! The value of `INITCODE_WORD_COST` is selected based on performance benchmarks of differing worst-cases per implementation. The baseline for the benchmarks is the performance of `KECCAK256` hashing in geth 1.10.9, which matches the 70 Mgas/s gas limit target on a 4.0 GHz `x86_64` CPU.
//!
//! | EVM             | version | MB/s | B/CPUcycle | CPUcycle/B | cost of 1 B | cost of 32 B |
//! | --------------- | ------- | ---- | ---- | ---- | ---- | ---- |
//! | geth/KECCAK256  | 1.10.9  |  357 |  1.8 |  0.6 |  0.2 |  6.0 |
//! | geth            | 1.10.9  | 1091 |  5.5 |  0.2 |  0.1 |  2.0 |
//! | evmone/Baseline | 0.8.2   |  727 |  3.7 |  0.3 |  0.1 |  2.9 |
//! | evmone/Advanced | 0.8.2   |  155 |  0.8 |  1.3 |  0.4 | 13.8 |
//!
//! ### Gas cost per word (32-byte chunk)
//!
//! We have chosen the cost of 2 gas per word based on Geth's implementation and comparing with `KECCAK256` performance. This means the per byte cost is `0.0625`. While fractional gas costs are not permitted in the EVM, we can approximate it by charging per-word.
//!
//! Moreover, calculating gas per word is compatible with the calculation of `CREATE2`'s *hashcost* of [EIP-1014](./eip-1014.md). Therefore, the same implementation may be used for `CREATE` and `CREATE2` with different cost constants: before activation `0` for `CREATE` and `6` for `CREATE2`, after activation `2` for `CREATE` and `6 + 2` for `CREATE2`.
//!
//! ### Reason for size limit of initcode
//!
//! Estimating and creating worst case scenarios is easier with an upper bound in place, given one parameter for the search is greatly reduced. This allows for selecting a much more optimistic gas per byte.
//!
//! Should there be no upper bound, the cost would need to be higher accounting for unknown unknowns. Given most *initcode* (*TODO: state maximum initcode size resulting in deployment seen on mainnet here*) does not exceed the proposed limit, penalising contracts by overly conservative costs seems unnecessary.
//!
//! ### Effect of size limit of initcode
//!
//! In most, if not all cases when a new contract is being created, the resulting runtime code is copied from the initcode itself. For the basic case the `2 * MAX_CODE_SIZE` limit allows `MAX_CODE_SIZE` for runtime code and another `MAX_CODE_SIZE` for contract constructor code. However, the limit may have practical implications for cases where multiple contracts are deployed in a single create transaction.
//!
//! ### Initcode cost for create transaction
//!
//! The initcode cost for create transaction data (0.0625 gas per byte) is negligible compared to the transaction data cost (4 or 16 gas per byte). Despite that, we decided to include it in the specification for consistency, and more importantly for forward compatibility.
//!
//! ### How to report initcode limit violation?
//!
//! We specified that initcode size limit violation for `CREATE`/`CREATE2` results in exceptional abort of the execution. This places it in the group of early out-of-gas checks, including: stack underflow, memory expansion, static call violation, initcode hashing cost, and initcode cost introduced by this EIP. They precede the later "light" checks: call depth and balance. The choice gives consistency to the order of checks and lowers implementation complexity (out-of-gas checks can be performed in any order).
//!
//! ## Backwards Compatibility
//!
//! This EIP requires a "network upgrade", since it modifies consensus rules.
//!
//! Already deployed contracts should not be affected, but certain transactions (with `initcode` beyond the proposed limit) would still be includable in a block, but result in an exceptional abort.
//!
//! ## Test Cases
//!
//! Tests should include the following cases:
//!
//! - Creation transaction with gas limit enough to cover initcode cost
//! - Creation transaction with gas limit enough to cover intrinsic cost except initcode cost
//! - `CREATE`/`CREATE2`/creation transaction with `len(initcode)` at `MAX_INITCODE_SIZE`
//! - `CREATE`/`CREATE2`/creation transaction with `len(initcode)` at `MAX_INITCODE_SIZE+1`
//!
//! ## Security Considerations
//!
//! For client implementations, this EIP makes attacks based on jumpdest-analysis less problematic, so should increase the robustness of clients.
//!
//! For layer 2, this EIP introduces failure-modes where there previously were none. There *could* exist factory-contracts which deploy multi-level contract hierarchies, such that the code for multiple contracts are included in the initcode of the first contract. The author(s) of this EIP are not aware of any such contracts.
//!
//! Currently, on London, with `30M` gas limit, it would be possible to trigger jumpdest-analysis of a total `~1.3GB` of initcode. With this EIP, the cost for such an attack would increase by roughly `80M` gas.
//!
//! Martin Holst Swende (@holiman), Paweł Bylica (@chfast), Alex Beregszaszi (@axic), Andrei Maiboroda (@gumb0), "EIP-3860: Limit and meter initcode," Ethereum Improvement Proposals, no. 3860, July 2021. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-3860>.

use crate::eip::Eip;

/// EIP-3860: Limit and meter initcode.
pub struct Eip3860;

impl Eip for Eip3860 {
    const NUMBER: u32 = 3860;
}
