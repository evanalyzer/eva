//! EIP-4399: Supplant DIFFICULTY opcode with PREVRANDAO.
//!
//! ## Abstract
//!
//! This EIP supplants the semantics of the return value of existing `DIFFICULTY (0x44)` opcode and renames the opcode to `PREVRANDAO (0x44)`.
//!
//! The return value of the `DIFFICULTY (0x44)` instruction after this change is the output of the randomness beacon provided by the beacon chain.
//!
//!
//! ## Motivation
//!
//! Applications may benefit from using the randomness accumulated by the beacon chain. Thus, randomness outputs produced by the beacon chain should be accessible in the EVM.
//!
//! At the point of `TRANSITION_BLOCK` of the Proof-of-Stake (`PoS`) upgrade described in [EIP-3675](./eip-3675.md), the `difficulty` block field **MUST** be `0` thereafter because there is no longer any Proof-of-Work (`PoW`) seal on the block. This means that the `DIFFICULTY (0x44)` instruction no longer has it's previous semantic meaning, nor a clear "correct" value to return.
//!
//! Given prior analysis on the usage of `DIFFICULTY`, the value returned by the instruction mixed with other values is a common pattern used by smart contracts to obtain randomness. The instruction with the same number as the `DIFFICULTY` opcode returning outputs of the beacon chain RANDAO implementation makes the upgrade to `PoS` backwards compatible for existing smart contracts obtaining randomness from the `DIFFICULTY` instruction.
//!
//! Additionally, changes proposed by this EIP allow for smart contracts to determine whether the upgrade to the `PoS` has already happened. This can be done by analyzing the return value of the `DIFFICULTY` instruction. A value greater than `2**64` indicates that the transaction is being executed in the `PoS` block. Decompilers and other similar tooling may also use this trick to discern the new semantics of the instruction if data of the block including the transaction in question is available.
//!
//!
//! ## Specification
//!
//! ### Definitions
//!
//! * **`TRANSITION_BLOCK`** The definition of this block can be found in the Definitions section of [EIP-3675](./eip-3675.md#definitions).
//!
//! ### Block structure
//!
//! Beginning with `TRANSITION_BLOCK`, client software **MUST** set the value of the `mixHash`, i.e. the field with the number `13` (0-indexed) in a block header, to the latest RANDAO mix of the post beacon state of the previous block.
//!
//! ### EVM
//!
//! Beginning with `TRANSITION_BLOCK`, the `DIFFICULTY (0x44)` instruction **MUST** return the value of the `mixHash` field.
//!
//! *Note*: The gas cost of the `DIFFICULTY (0x44)` opcode remains unchanged.
//!
//! ### Renaming
//!
//! The `mixHash` field **SHOULD** further be renamed to `prevRandao`.
//!
//! The `DIFFICULTY (0x44)` opcode **SHOULD** further be renamed to `PREVRANDAO (0x44)`.
//!
//!
//! ## Rationale
//!
//! ### Including RANDAO output in the block header
//!
//! Including a RANDAO output in the block header provides a straightforward method of accessing it from inside of the EVM as block header data is already available in the EVM context.
//!
//! Additionally, this ensures that the execution layer can be fully executed with the block alone rather than requiring extra inputs from the `PoS` consensus layer.
//!
//! Mixing the randomness into a block header may contribute to uniqueness of the block hash in the case when values of other fields of the block header match the corresponding values of the header of another block.
//!
//! ### Using `mixHash` field instead of `difficulty`
//!
//! The `mixHash` header field is used instead of `difficulty` to avoid a class of hidden forkchoice bugs after the `PoS` upgrade.
//!
//! Client software implementing pre-EIP-3675 logic heavily depends on the `difficulty` value as total difficulty computation is the basis of the `PoW` fork choice rule. Setting the `difficulty` field to `0` at the `PoS` upgrade aims to reduce the surface of bugs related to the total difficulty value growing after the upgrade.
//!
//! Additionally, any latent total difficulty computation after the `PoS` upgrade would become overflow prone if the randomness output supplanted the value of the `difficulty` field.
//!
//! ### Reusing existing field instead of appending a new one
//!
//! The `mixHash` field is deprecated at the `PoS` upgrade and set to zero bytes array thereafter. Reusing an existing field as a place for the randomness output saves 32 bytes per block and effectively removes the deprecation of one of the fields induced by the upgrade.
//!
//! ### Reusing the `DIFFICULTY` opcode instead of introducing a new one
//!
//! See the [Motivation](#motivation).
//!
//! ### Renaming the field and the opcode
//!
//! The renaming should be done to make the field and the opcode names semantically sound.
//!
//! ### Using `TRANSITION_BLOCK` rather than a block or slot number
//!
//! By utilizing `TRANSITION_BLOCK` to trigger the change in logic defined in this EIP rather than a block or slot number, this EIP is tightly coupled to the `PoS` upgrade defined by [EIP-3675](./eip-3675.md).
//!
//! By tightly coupling to the `PoS` upgrade, we ensure that there is no discontinuity for the usecase of this opcode for randomness -- the primary [motivation](#motivation) for re-using `DIFFICULTY` rather than creating a new opcode.
//!
//! ### Using `2**64` threshold to determine `PoS` blocks
//!
//! The probability of RANDAO value to fall into the range between `0` and `2**64` and, thus, to be mixed with `PoW` difficulty values, is drastically low. Though, proposed threshold might seem to have insufficient distance from difficulty values on Ethereum Mainnet (they are currently around `2**54`), it requires a thousand times increase of the hashrate to make this threshold insecure. Such an increase is considered impossible to occur before the upcoming consensus upgrade.
//!
//!
//! ## Backwards Compatibility
//!
//! This EIP introduces backward incompatible changes to the execution and validation of EVM state transitions. As written, this EIP utilizes `TRANSITION_BLOCK` and is thus tightly coupled with the `PoS` upgrade introduced in [EIP-3675](./eip-3675.md). If this EIP is to be adopted, it **MUST** be scheduled at the same time as EIP-3675.
//!
//! Additionally, the changes proposed might be backward incompatible for the following categories of applications:
//! * Applications that use the value returned by the `DIFFICULTY` opcode as the `PoW` `difficulty` parameter
//! * Applications with logic that depends on the `DIFFICULTY` opcode returning a relatively small number with respect to the full 256-bit size of the field.
//!
//! The first category is already affected by switching the consensus mechanism to `PoS` and no additional breaking changes are introduced by this specification.
//!
//! The second category is comprised of applications that use the return value of the `DIFFICULTY` opcode in operations that might cause either overflow or underflow errors. While it is theoretically possible to author an application where a change in the range of possible values this opcode may return could lead to a security vulnerability, the chances of that are negligible.
//!
//!
//! ## Test Cases
//!
//! * In one of ancestors of `TRANSITION_BLOCK` deploy a contract that stores return value of  `DIFFICULTY (0x44)` to the state
//! * Check that value returned by `DIFFICULTY (0x44)` in transaction executed within the parent of `TRANSITION_BLOCK` equals `difficulty` field value
//! * Check that value returned by `PREVRANDAO (0x44)` in transaction executed within `TRANSITION_BLOCK` equals `prevRandao` field value
//!
//!
//! ## Security Considerations
//!
//! The `PREVRANDAO (0x44)` opcode in `PoS` Ethereum (based on the beacon chain RANDAO implementation) is a source of randomness with different properties to the randomness supplied by `BLOCKHASH (0x40)` or `DIFFICULTY (0x44)` opcodes in the `PoW` network.
//!
//! ### Biasability
//!
//! The beacon chain RANDAO implementation gives every block proposer 1 bit of influence power per slot. Proposer may deliberately refuse to propose a block on the opportunity cost of proposer and transaction fees to prevent beacon chain randomness (a RANDAO mix) from being updated in a particular slot.
//!
//! An effect of proposer's influence power is limited in time and lasts until the first honest RANDAO reveal is made afterwards. This limitation does also exist in the case when proposers of `n` consecutive slots are colluding to get `n` bits of influence power. Simply speaking, one honest block proposal is enough to unbias the RANDAO even if it was biased during several slots in a row.
//!
//! Additionally, semantics of the `PREVRANDAO (0x44)` instruction gives proposers another way to gain 1 bit of influence power on applications. Biased proposer may censor a rolling the dice transaction to force it to be included into the next block, thus, force it to use a RANDAO mix that the proposer knows in advance. The opportunity cost in this case would be negligible.
//!
//! ### Predictability
//!
//! Obviously, historical randomness provided by any decentralized oracle is 100% predictable. On the contrary, the randomness that is revealed in the future is predictable up to a limited extent.
//!
//! A list of inputs influencing future randomness on the beacon chain consists of but is not limited to the following items:
//! * **Accumulated randomness.** A RANDAO mix produced by the beacon chain in the last slot of epoch `N` is the main input to the function defining block proposers in each slot of epoch `N + MIN_SEED_LOOKAHEAD + 1`, i.e. it is the main factor defining future RANDAO revealers.
//! * **Number of active validators.** A number of active validators throughout an epoch is another input to the block proposer function.
//! * **Effective balance.** All else being equal, the lower the effective balance of a validator the lower the chance this validator has to be designated as a proposer in a slot.
//! * **Accidentally missed proposals.** Network conditions and other factors that are resulting in accidentally missed proposals is a source of highly qualitative entropy that impacts RANDAO mixes. Usual rate of missed proposals on the Mainnet is about `1%`.
//!
//! These inputs may be predictable and malleable on a short range of slots but the longer the attempted lookahead the more entropy is accumulated by the beacon chain.
//!
//! ### Tips for application developers
//!
//! The following tips attempt to reduce predictability and biasability of randomness outputs returned by `PREVRANDAO (0x44)`:
//!
//! 1. Make your applications rely on the future randomness with a reasonably high lookahead. For example, an application stops accepting bids at the end of epoch `K` and uses a RANDAO mix produced in slot `K + N + ε` to roll the dice, where `N` is a lookahead in epochs and `ε` is a few slots into epoch `N + 1`.
//! 2. At least four epochs of lookahead results in the following outcome:
//!   * A proposer set of epoch `N + 1` isn't known at the end of epoch `K` breaking a direct link between bidders and dice rollers
//!   * A number of active validators is updated at the end of each epoch affecting a set of proposers of next epochs, thus, impacting a RANDAO mix used by the application to roll the dice
//!   * Due to Mainnet statistics, there is about a `100%` chance for the network to accidentally miss a proposal during this period of time which reduces predictability of a RANDAO mix used to roll the dice.
//! 3. Setting `ε` to a small number, e.g. 2 or 4 slots, gives a third party a little time to gain influence power on the future randomness that is being used to roll the dice. This amount of time is defined by `MIN_SEED_LOOKAHEAD` parameter and is about 6 minutes on the Mainnet.
//!
//! A reasonably high distance between bidding and rolling the dice attempts to leave low chance for bidders controlling a subset of validators to directly exploit their influence power. Ultimately, this chance depends on the type of the game and on a number of controlled validators. For instance, a chance of a single validator to affect a one-time game is negligible, and becomes bigger for multiple validators in a repeated game scenario.

use crate::eip::Eip;

/// EIP-4399: Supplant DIFFICULTY opcode with PREVRANDAO.
pub struct Eip4399;

impl Eip for Eip4399 {
    const NUMBER: u32 = 4399;
}
