//! EIP-2929: Gas cost increases for state access opcodes.
//!
//! ## Simple Summary
//!
//! Increases gas cost for `SLOAD`, `*CALL`, `BALANCE`, `EXT*` and `SELFDESTRUCT` when used for the first time in a transaction.
//!
//! ## Abstract
//!
//! Increase the gas cost of `SLOAD` (`0x54`) to 2100, and the `*CALL` opcode family (`0xf1`, `f2`, `f4`, `fA`), `BALANCE` `0x31` and the `EXT*` opcode family (`0x3b`, `0x3c`, `0x3f`) to 2600. Exempts (i) precompiles, and (ii) addresses and storage slots that have already been accessed in the same transaction, which get a decreased gas cost. Additionally reforms `SSTORE` metering and `SELFDESTRUCT` to ensure "de-facto storage loads" inherent in those opcodes are priced correctly.
//!
//! ## Motivation
//!
//! Generally, the main function of gas costs of opcodes is to be an estimate of the time needed to process that opcode, the goal being for the gas limit to correspond to a limit on the time needed to process a block. However, storage-accessing opcodes (`SLOAD`, as well as the `*CALL`, `BALANCE` and `EXT*` opcodes) have historically been underpriced. In the 2016 Shanghai `DoS` attacks, once the most serious client bugs were fixed, one of the more durably successful strategies used by the attacker was to simply send transactions that access or call a large number of accounts.
//!
//! Gas costs were increased to mitigate this, but recent numbers suggest they were not increased enough. Quoting [https://arxiv.org/pdf/1909.07220.pdf](https://arxiv.org/pdf/1909.07220.pdf):
//!
//! > Although by itself, this issue might seem benign, `EXTCODESIZE` forces the  client to search the contract ondisk, resulting in IO heavy transactions.  While replaying the Ethereum history on our hardware, the malicious transactions took around 20 to 80 seconds to execute, compared to a few milliseconds for the average transactions
//!
//! This proposed EIP increases the costs of these opcodes by a factor of ~3, reducing the worst-case processing time to ~7-27 seconds. Improvements in database layout that involve redesigning the client to read storage directly instead of hopping through the Merkle tree would decrease this further, though these technologies may take a long time to fully roll out, and even with such technologies the IO overhead of accessing storage would remain substantial.
//!
//! A secondary benefit of this EIP is that it also performs most of the work needed to make [stateless witness sizes](https://ethereum-magicians.org/t/protocol-changes-to-bound-witness-size/3885) in Ethereum acceptable. Assuming [a switch to binary tries](https://ethresear.ch/t/binary-trie-format/7621), the theoretical maximum witness size not including code size (hence "most of the work" and not "all") would decrease from `(12500000 gas limit) / (700 gas per BALANCE) * (800 witness bytes per BALANCE) ~= 14.3M bytes` to `12500000 / 2600 * 800 ~= 3.85M bytes`. Pricing for code access could be changed when code merklization is implemented.
//!
//! In the further future, there are similar benefits in the case of SNARK/STARK witnesses. Recent numbers from Starkware suggest that they are able to prove 10000 Rescue hashes per second on a consumer desktop; assuming 25 hashes per Merkle branch, and a block full of state accesses, at present this would imply a witness would take `12500000 / 700 * 25 / 10000 ~= 44.64` seconds to generate, but after this EIP that would reduce to `12500000 / 2500 * 25 / 10000 ~= 12.5` seconds, meaning that a single desktop computer would be able to generate witnesses on time under any conditions. Future gains in STARK proving could be spent on either (i) using a more expensive but robust hash function or (ii) reducing proving times further, reducing the delay and hence improving user experience of stateless clients that rely on such witnesses.
//!
//! ## Specification
//!
//! ### Parameters
//!
//! | Constant | Value |
//! | - | - |
//! | `FORK_BLOCK` | 12244000 |
//! | `COLD_SLOAD_COST` | 2100 |
//! | `COLD_ACCOUNT_ACCESS_COST` | 2600 |
//! | `WARM_STORAGE_READ_COST` | 100 |
//!
//! For blocks where `block.number >= FORK_BLOCK`, the following changes apply.
//!
//! When executing a transaction, maintain a set `accessed_addresses: Set[Address]` and `accessed_storage_keys: Set[Tuple[Address, Bytes32]]` .
//!
//! The sets are transaction-context-wide, implemented identically to other transaction-scoped constructs such as the self-destruct-list and global `refund` counter. In particular, if a scope reverts, the access lists should be in the state they were in before that scope was entered.
//!
//! When a transaction execution begins,
//!   - `accessed_storage_keys` is initialized to empty, and
//!   - `accessed_addresses` is initialized to include
//!     - the `tx.sender`, `tx.to` (or the address being created if it is a contract creation transaction)
//!     - and the set of all precompiles.
//!
//!
//! ### Storage read changes
//!
//! When an address is either the target of a (`EXTCODESIZE` (`0x3B`), `EXTCODECOPY` (`0x3C`), `EXTCODEHASH` (`0x3F`) or `BALANCE` (`0x31`)) opcode or the target of a (`CALL` (`0xF1`), `CALLCODE` (`0xF2`), `DELEGATECALL` (`0xF4`), `STATICCALL` (`0xFA`)) opcode, the gas costs are computed as follows:
//!
//! * If the target is not in `accessed_addresses`, charge `COLD_ACCOUNT_ACCESS_COST` gas, and add the address to `accessed_addresses`.
//! * Otherwise, charge `WARM_STORAGE_READ_COST` gas.
//!
//! In all cases, the gas cost is charged and the map is updated at the time that the opcode is being called.
//! When a `CREATE` or `CREATE2` opcode is called, immediately (ie. before checks are done to determine whether or not the address is unclaimed) add the address being created to `accessed_addresses`, but gas costs of `CREATE` and `CREATE2` are unchanged.
//! Clarification: If a `CREATE`/`CREATE2` operation fails later on, e.g during the execution of `initcode` or has insufficient gas to store the code in the state, the `address` of the contract itself remains in `access_addresses` (but any additions made within the inner scope are reverted).
//!
//! For `SLOAD`, if the `(address, storage_key)` pair (where `address` is the address of the contract whose storage is being read) is not yet in `accessed_storage_keys`, charge `COLD_SLOAD_COST` gas and add the pair to `accessed_storage_keys`. If the pair is already in `accessed_storage_keys`, charge `WARM_STORAGE_READ_COST` gas.
//!
//! Note: For call-variants, the `100`/`2600` cost is applied immediately (exactly like how `700` was charged before this EIP), i.e: before calculating the `63/64ths` available for entering the call.
//!
//! Note 2: There is currently no way to perform a 'cold sload read/write' on a 'cold account', simply because in order to read/write a `slot`, the execution must already be inside the `account`. Therefore, the behaviour of cold  storage reads/writes on cold accounts is undefined as of this EIP. Any future EIP which
//! proposes to add 'remote read/write' would need to define the pricing behaviour of that change.
//!
//! ### SSTORE changes
//!
//! When calling `SSTORE`, check if the `(address, storage_key)` pair is in `accessed_storage_keys`. If it is not, charge an additional `COLD_SLOAD_COST` gas, and add the pair to `accessed_storage_keys`. Additionally, modify the parameters defined in [EIP-2200](./eip-2200.md) as follows:
//!
//! | Parameter | Old value | New value |
//! | - | - | - |
//! | `SLOAD_GAS` | 800 | `= WARM_STORAGE_READ_COST` |
//! | `SSTORE_RESET_GAS` | 5000 | `5000 - COLD_SLOAD_COST` |
//!
//! The other parameters defined in EIP 2200 are unchanged.
//! Note: The constant `SLOAD_GAS` is used in several places in EIP 2200, e.g `SSTORE_SET_GAS - SLOAD_GAS`. Implementations that are using composite definitions have to ensure to update those definitions too.
//!
//! ### SELFDESTRUCT changes
//!
//! If the ETH recipient of a `SELFDESTRUCT` is not in `accessed_addresses` (regardless of whether or not the amount sent is nonzero), charge an additional `COLD_ACCOUNT_ACCESS_COST` on top of the existing gas costs, and add the ETH recipient to the set.
//!
//! Note: `SELFDESTRUCT` does not charge a `WARM_STORAGE_READ_COST` in case the recipient is already warm, which differs from how the other call-variants work. The reasoning behind this is to keep the changes small, a `SELFDESTRUCT` already costs `5K` and is a no-op if invoked more than once.
//!
//! ## Rationale
//!
//! ### Opcode costs vs charging per byte of witness data
//!
//! The natural alternative path to changing gas costs to reflect witness sizes is to charge per byte of witness data. However, that would take a longer time to implement, hampering the goal of providing short-term security relief. Furthermore, following that path faithfully would lead to extremely high gas costs to transactions that touch contract code, as one would need to charge for all 24576 contract code bytes; this would be an unacceptably high burden on developers. It is better to wait for [code merklization](https://medium.com/ewasm/evm-bytecode-merklization-2a8366ab0c90) to start trying to properly account for gas costs of accessing individual chunks of code; from a short-term `DoS` prevention standpoint, accessing 24 kB from disk is not much more expensive than accessing 32 bytes from disk, so worrying about code size is not necessary.
//!
//! ### Adding the `accessed_addresses` / `accessed_storage_keys` sets
//!
//! The sets of already-accessed accounts and storage slots are added to avoid needlessly charging for things that can be cached (and in all performant implementations already are cached). Additionally, it removes the current undesirable status quo where it is needlessly unaffordable to do self-calls or call precompiles, and enables contract breakage mitigations that involve pre-fetching some storage key allowing a future execution to still take the expected amount of gas.
//!
//! ### SSTORE gas cost change
//!
//! The change to SSTORE is needed to avoid the possibility of a `DoS` attack that "pokes" a randomly chosen zero storage slot, changing it from 0 to 0 at a cost of 800 gas but requiring a de-facto storage load. The `SSTORE_RESET_GAS` reduction ensures that the total cost of SSTORE (which now requires paying the `COLD_SLOAD_COST`) remains unchanged. Additionally, note that applications that do `SLOAD` followed by `SSTORE` (eg. `storage_variable += x`) _would actually get cheaper_!
//!
//! ### Change SSTORE accounting only minimally
//!
//! The SSTORE gas costs continue to use Wei Tang's original/current/new approach, instead of being redesigned to use a dirty map, because Wei Tang's approach correctly accounts for the actual costs of changing storage, which only care about current vs final value and not intermediate values.
//!
//! ### How would gas consumption of average applications increase under this proposal?
//!
//! #### Rough analysis from witness sizes
//!
//! We can look at [Alexey Akhunov's earlier work](https://medium.com/@akhounov/data-from-the-ethereum-stateless-prototype-8c69479c8abc) for data on average-case blocks. In summary, average blocks have witness sizes of ~1000 kB, of which ~750 kB is Merkle proofs and not code. Assuming a conservative 2000 bytes per Merkle branch this implies ~375 accesses per block (SLOADs have a similar gas-increase-to-bytes ratio so there's no need to analyze them separately).
//!
//! Data on [txs per day](https://etherscan.io/chart/tx) and [blocks per day](https://etherscan.io/chart/blocks) from Etherscan gives ~160 transactions per block (reference date: Jul 1), implying a large portion of those accesses are just the `tx.sender` and `tx.to` which are excluded from gas cost increases, though likely less than 320 due to duplicate addresses.
//!
//! Hence, this implies ~50-375 chargeable accesses per block, and each access suffers a gas cost increase of 1900; `50 * 1900 = 95000` and `375 * 1900 = 712500`, implying the gas limit would need to be raised by ~1-6% to compensate. However, this analysis may be complicated further in either direction by (i) accounts / storage keys being accessed in multiple transactions, which would appear once in the witness but twice in gas cost increases, and (ii) accounts / storage keys being accessed multiple times in the same transaction, which lead to gas cost _decreases_.
//!
//! #### Goerli analysis
//!
//! A more precise analysis can be found by scanning Goerli transactions, as done by Martin Swende here: <https://github.com/holiman/gasreprice>
//!
//! The conclusion is that on average gas costs increase by ~2.36%. One major contributing factor to reducing gas costs is that a large number of contracts inefficiently read the same storage slot multiple times, which leads to this EIP giving a few transactions gas cost _savings_ of over 10%.
//!
//! ## Backwards Compatibility
//!
//! These gas cost increases may potentially break contracts that depend on fixed gas costs; see the security considerations section for details and arguments for why we expect the total risks to be low and how if desired they can be reduced further.
//!
//! ## Test Cases
//!
//! Some test cases can be found here: <https://gist.github.com/holiman/174548cad102096858583c6fbbb0649a>
//!
//! Ideally we would test the following:
//!
//! * SLOAD the same storage slot {1, 2, 3} times
//! * CALL the same address {1, 2, 3} times
//! * (SLOAD | CALL) in a sub-call, then revert, then (SLOAD | CALL) the same (storage slot | address) again
//! * Sub-call, SLOAD, sub-call again, revert the inner sub-call, SLOAD the same storage slot
//! * SSTORE the same storage slot {1, 2, 3} times, using all combinations of zero/nonzero for original value and the value being set
//! * SSTORE then SLOAD the same storage slot
//! * `OP_1` then `OP_2` to the same address where `OP_1` and `OP_2` are all combinations of (`*CALL`, `EXT*`, `SELFDESTRUCT`)
//! * Try to `CALL` an address but with all possible failure modes (not enough gas, not enough ETH...), then (`CALL` | `EXT*`) that address again successfully
//!
//! Vitalik Buterin (@vbuterin), Martin Swende (@holiman), "EIP-2929: Gas cost increases for state access opcodes," Ethereum Improvement Proposals, no. 2929, September 2020. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-2929>.

use crate::eip::Eip;

/// EIP-2929: Gas cost increases for state access opcodes.
pub struct Eip2929;

impl Eip for Eip2929 {
    const NUMBER: u32 = 2929;
}
