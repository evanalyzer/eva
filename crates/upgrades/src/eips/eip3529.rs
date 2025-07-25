//! EIP-3529: Reduction in refunds.
//!
//! ## Simple Summary
//!
//! Remove gas refunds for `SELFDESTRUCT`, and reduce gas refunds for `SSTORE` to a lower level where the refunds are still substantial, but they are no longer high enough for current "exploits" of the refund mechanism to be viable.
//!
//! ## Motivation
//!
//! Gas refunds for `SSTORE` and `SELFDESTRUCT` were originally introduced to motivate application developers to write applications that practice "good state hygiene", clearing storage slots and contracts that are no longer needed. However, the benefits of this technique have proven to be far lower than anticipated, and gas refunds have had multiple unexpected harmful consequences:
//!
//! * Refunds give rise to `GasToken`. `GasToken` has benefits in moving gas space from low-fee periods to high-fee periods, but it also has downsides to the network, particularly in exacerbating state size (as state slots are effectively used as a "battery" to save up gas) and inefficiently clogging blockchain gas usage
//! * Refunds increase block size variance. The theoretical maximum amount of actual gas consumed in a block is nearly twice the on-paper gas limit (as refunds add gas space for subsequent transactions in a block, though refunds are capped at 50% of a transaction's gas used). This is not fatal, but is still undesirable, especially given that refunds can be used to maintain 2x usage spikes for far longer than EIP-1559 can.
//!
//!
//! ## Specification
//!
//! ### Parameters
//!
//! | Constant | Value |
//! | - | - |
//! | `FORK_BLOCK` | TBD |
//! | `MAX_REFUND_QUOTIENT` | 5 |
//!
//! For blocks where `block.number >= FORK_BLOCK`, the following changes apply.
//!
//! 1. Remove the `SELFDESTRUCT` refund.
//! 2. Replace `SSTORE_CLEARS_SCHEDULE` (as defined in [EIP-2200](./eip-2200.md)) with `SSTORE_RESET_GAS + ACCESS_LIST_STORAGE_KEY_COST` (4,800 gas as of [EIP-2929](./eip-2929.md) + [EIP-2930](./eip-2930.md))
//! 3. Reduce the max gas refunded after a transaction to `gas_used // MAX_REFUND_QUOTIENT`
//!
//! Remark: Previously _max gas refunded_ was defined as `gas_used // 2`. Here we
//! name the constant `2` as `MAX_REFUND_QUOTIENT` and change its value to `5`.
//!
//! ## Rationale
//!
//! In [EIP-2200](./eip-2200.md#specification), three cases for refunds were introduced:
//!
//! 1. If the original value is nonzero, and the new value is zero, add `SSTORE_CLEARS_SCHEDULE` (currently 15,000) gas to the refund counter
//! 2. If the original value is zero, the current value is nonzero, and the new value is zero, add `SSTORE_SET_GAS - SLOAD_GAS` (currently 19,900) gas to the refund counter
//! 3. If the original value is nonzero, the current value is a different nonzero value, and the new value equals the original value, add `SSTORE_RESET_GAS - SLOAD_GAS` (currently 4,900) gas to the refund counter
//!
//! Of these three, only (1) enables gastokens and allows a block to expend more gas on execution than the block gas limit. (2) does not have this property, because for the 19,900 refund to be obtained, _the same storage slot_ must have been changed from zero to nonzero previously, costing 20,000 gas. The inability to obtain gas from clearing one storage slot and use it to edit another storage slot means that it cannot be used for gas tokens. Additionally, obtaining the refund requires _reverting_ the effect of the storage write and expansion, so the refunded gas does not contribute to a client's load in processing a block. (3) behaves similarly: the 4,900 refund can only be obtained when 5,000 gas had previously been spent on the same storage slot.
//!
//! This EIP deals with case (1). We can establish under what conditions a gastoken is nonviable (ie. you cannot get more gas out of a storage slot than you put in) by using a similar "pairing" argument, mapping each refund to a previous expenditure in the same transaction on the same storage slot. lf a storage slot is changed to zero when its original value is nonzero, there are two possibilities:
//!
//! 1. This could be the first time that the storage slot is set to zero. In this case, we can pair this event with the `SSTORE_RESET_GAS + ACCESS_LIST_STORAGE_KEY_COST` minimum cost of reading and editing the storage slot for the first time.
//! 2. This could be the second or later time that the storage slot is set to zero. In this case, we can pair this event with the most recent previous time that the value was set _away_ from zero, in which `SSTORE_CLEARS_SCHEDULE` gas is _removed_ from the refund.
//!
//! For the second and later event, it does not matter what value `SSTORE_CLEARS_SCHEDULE` has, because every refund of that size is paired with a refund _removal_ of the same size. This leaves the first event. For the total gas expended on the slot to be guaranteed to be positive, we need `SSTORE_CLEARS_SCHEDULE <= SSTORE_RESET_GAS + ACCESS_LIST_STORAGE_KEY_COST`. And so this EIP simply decreases `SSTORE_CLEARS_SCHEDULE` to the sum of those two costs.
//!
//! One alternative intuition for this EIP is that there will not be a net refund for clearing data that has not yet been read (which is often "useless" data), but there will continue to be a net refund for clearing data that has been read (which is likely to be "useful" data).
//!
//! ## Backwards Compatibility
//!
//! Refunds are currently only applied _after_ transaction execution, so they cannot affect how much gas is available to any particular call frame during execution. Hence, removing them will not break the ability of any code to execute, though it will render some applications economically nonviable.
//!
//! Gas tokens will become valueless. `DeFi` arbitrage bots, which today frequently use either established gas token schemes or a custom alternative to reduce on-chain costs, would benefit from rewriting their code to remove calls to these no-longer-functional gas storage mechanisms.
//!
//! However, fully preserving refunds in the `new = original = 0 != current` case, and keeping _some_ refund in the other `nonzero -> zero` cases, ensures that a few key use cases that receive (and deserve) favorable gas cost treatment continue to do so. For example, `zero -> nonzero -> zero` storage set patterns continue to cost only ~100 gas. Two important examples of such patterns include:
//!
//! * Anti-reentrancy locks (typically flipped from 0 to 1 right before a child call begins, and then flipped back to 0 when the child call ends)
//! * ERC20 approve-and-send (the "approved value" goes from zero to nonzero when the token transfer is approved, and then back to zero when the token transfer processes)
//!
//! ### Effect on storage clearing incentives
//!
//! A criticism of earlier refund removal EIPs ([EIP-3298](./eip-3298.md) and [EIP-3403](./eip-3403.md)) is that these EIPs fully remove the incentive to set a value to zero, encouraging users to not fully clear a storage slot if they expect even the smallest probability that they will want to use that storage slot again.
//!
//! For example, if you have 1 unit of an ERC20 token and you are giving away or selling your entire balance, you could instead only give away 0.999999 units and leave the remainder behind. If you ever decide to re-acquire more of that token with the same account in the future, you would only have to pay 5000 gas (2100 for the read + 2900 for nonzero-to-nonzero set) for the `SSTORE` instead of 22100 (20000 for the zero-to-nonzero set). Today, this is counterbalanced by the 15000 refund for clearing, so you only have an incentive to do this if you are more than `15000 / 17100 = 87.7%` sure that you will use the slot again; with EIP-3298 or EIP-3403 the counterbalancing incentive would not exist, so setting to nonzero is better if your chance of using the slot again is _any_ value greater than 0%.
//!
//! A refund of 4800 gas remains, so there is only be an incentive to keep a storage slot nonzero if you expect a probability of more than `4800 / 17100 = 28.1%` that you will use that slot again. This is not perfect, but it is likely higher than the average person's expectations of later re-acquiring a token with the same address if they clear their entire balance of it.
//!
//! The capping of refunds to 1/5 of gas expended means that this refund can only be used to increase the amount of storage write operations needed to process a block by at most 25%, limiting the ability to use this mechanic for storage-write-focused denial-of-service attacks.
//!
//! ## Test Cases
//!
//! ### EIP-2929 Gas Costs
//!
//! Note, there is a difference between 'hot' and 'cold' slots. This table shows the values as of [EIP-2929](./eip-2929.md) assuming that all touched storage slots were already 'hot' (the difference being a one-time cost of `2100` gas).
//!
//! | Code | Used Gas | Refund | Original | 1st | 2nd | 3rd | Effective gas (after refund)
//! | -- | -- | -- | -- | -- | -- | -- | -- |
//! | `0x60006000556000600055` | 212 | 0| 0 | 0 |  0 |  |  212 |
//! | `0x60006000556001600055` | 20112 | 0| 0 | 0 |  1 |  |  20112 |
//! | `0x60016000556000600055` | 20112 | 19900| 0 | 1 |  0 |  |  212 |
//! | `0x60016000556002600055` | 20112 | 0| 0 | 1 |  2 |  |  20112 |
//! | `0x60016000556001600055` | 20112 | 0| 0 | 1 |  1 |  |  20112 |
//! | `0x60006000556000600055` | 3012 | 15000| 1 | 0 |  0 |  |  -11988 |
//! | `0x60006000556001600055` | 3012 | 2800| 1 | 0 |  1 |  |  212 |
//! | `0x60006000556002600055` | 3012 | 0| 1 | 0 |  2 |  |  3012 |
//! | `0x60026000556000600055` | 3012 | 15000| 1 | 2 |  0 |  |  -11988 |
//! | `0x60026000556003600055` | 3012 | 0| 1 | 2 |  3 |  |  3012 |
//! | `0x60026000556001600055` | 3012 | 2800| 1 | 2 |  1 |  |  212 |
//! | `0x60026000556002600055` | 3012 | 0| 1 | 2 |  2 |  |  3012 |
//! | `0x60016000556000600055` | 3012 | 15000| 1 | 1 |  0 |  |  -11988 |
//! | `0x60016000556002600055` | 3012 | 0| 1 | 1 |  2 |  |  3012 |
//! | `0x60016000556001600055` | 212 | 0| 1 | 1 |  1 |  |  212 |
//! | `0x600160005560006000556001600055` | 40118 | 19900| 0 | 1 |  0 |  1 |  20218 |
//! | `0x600060005560016000556000600055` | 5918 | 17800| 1 | 0 |  1 |  0 |  -11882 |
//!
//! ### With reduced refunds
//!
//! If refunds were to be partially removed, by changing `SSTORE_CLEARS_SCHEDULE` from 15000 to 4800 (and removing selfdestruct refund) this would be the comparative table.
//!
//! | Code | Used Gas | Refund | Original | 1st | 2nd | 3rd | Effective gas (after refund)
//! | -- | -- | -- | -- | -- | -- | -- | -- |
//! | `0x60006000556000600055` | 212 | 0| 0 | 0 |  0 |  |  212 |
//! | `0x60006000556001600055` | 20112 | 0| 0 | 0 |  1 |  |  20112 |
//! | `0x60016000556000600055` | 20112 | 19900| 0 | 1 |  0 |  |  212 |
//! | `0x60016000556002600055` | 20112 | 0| 0 | 1 |  2 |  |  20112 |
//! | `0x60016000556001600055` | 20112 | 0| 0 | 1 |  1 |  |  20112 |
//! | `0x60006000556000600055` | 3012 | 4800| 1 | 0 |  0 |  |  -1788 |
//! | `0x60006000556001600055` | 3012 | 2800| 1 | 0 |  1 |  |  212 |
//! | `0x60006000556002600055` | 3012 | 0| 1 | 0 |  2 |  |  3012 |
//! | `0x60026000556000600055` | 3012 | 4800| 1 | 2 |  0 |  |  -1788 |
//! | `0x60026000556003600055` | 3012 | 0| 1 | 2 |  3 |  |  3012 |
//! | `0x60026000556001600055` | 3012 | 2800| 1 | 2 |  1 |  |  212 |
//! | `0x60026000556002600055` | 3012 | 0| 1 | 2 |  2 |  |  3012 |
//! | `0x60016000556000600055` | 3012 | 4800| 1 | 1 |  0 |  |  -1788 |
//! | `0x60016000556002600055` | 3012 | 0| 1 | 1 |  2 |  |  3012 |
//! | `0x60016000556001600055` | 212 | 0| 1 | 1 |  1 |  |  212 |
//! | `0x600160005560006000556001600055` | 40118 | 19900| 0 | 1 |  0 |  1 |  20218 |
//! | `0x600060005560016000556000600055` | 5918 | 7600| 1 | 0 |  1 |  0 |  -1682 |
//!
//! ## Security Considerations
//!
//! Refunds are not visible to transaction execution, so this should not have any impact on transaction execution logic.
//!
//! The maximum amount of gas that can be spent on execution in a block is limited to the gas limit, if we do not count zero-to-nonzero `SSTORE`s that were later reset back to zero. It is okay to not count those, because if such an `SSTORE` is reset, storage is not expanded and the client does not need to actually adjust the Merke tree; the gas consumption is refunded, but the effort normally required by the client to process those opcodes is also cancelled. **Clients should make sure to not do a storage write if `new_value = original_value`; this was a prudent optimization since the beginning of Ethereum but it becomes more important now.**
//!
//! Vitalik Buterin (@vbuterin), Martin Swende (@holiman), "EIP-3529: Reduction in refunds," Ethereum Improvement Proposals, no. 3529, April 2021. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-3529>.

use crate::eip::Eip;

/// EIP-3529: Reduction in refunds.
pub struct Eip3529;

impl Eip for Eip3529 {
    const NUMBER: u32 = 3529;
}
