//! EIP-3554: Difficulty Bomb Delay to December 2021.
//!
//! ## Simple Summary
//! Delays the difficulty bomb to show effect the first week of December 2021.
//!
//! ## Abstract
//! Starting with `FORK_BLOCK_NUMBER` the client will calculate the difficulty based on a fake block number suggesting to the client that the difficulty bomb is adjusting 9,700,000 blocks later than the actual block number.
//!
//! ## Motivation
//! Targeting for the Shanghai upgrade and/or the Merge to occur before December 2021. Either the bomb can be readjusted at that time, or removed all together.
//!
//! ## Specification
//! #### Relax Difficulty with Fake Block Number
//! For the purposes of `calc_difficulty`, simply replace the use of `block.number`, as used in the exponential ice age component, with the formula:
//! ```py
//!     fake_block_number = max(0, block.number - 9_700_000) if block.number >= FORK_BLOCK_NUMBER else block.number
//! ```
//! ## Rationale
//!
//! The following script predicts a .1 second delay to blocktime the first week of december and a 1 second delay by the end of the month. This gives reason to address because the effect will be seen, but not so much urgency we don't have space to work around if needed.
//!
//! ```python
//! def predict_diff_bomb_effect(current_blknum, current_difficulty, block_adjustment, months):
//!     '''
//!     Predicts the effect on block time (as a ratio) in a specified amount of months in the future.
//!     Vars used in last prediction:
//!     current_blknum = 12382958
//!     current_difficulty = 7393633000000000
//!     block adjustment = 9700000
//!     months = 6
//!     '''
//!     blocks_per_month = (86400 * 30) // 13.3
//!     future_blknum = current_blknum + blocks_per_month * months
//!     diff_adjustment = 2 ** ((future_blknum - block_adjustment) // 100000 - 2)
//!     diff_adjust_coeff = diff_adjustment / current_difficulty * 2048
//!     return diff_adjust_coeff
//!
//!
//! diff_adjust_coeff = predict_diff_bomb_effect(12382958,7393633000000000,9700000,6)
//! ```
//!
//! ## Backwards Compatibility
//! No known backward compatibility issues.
//!
//! ## Security Considerations
//! Misjudging the effects of the difficulty can mean longer blocktimes than anticipated until a hardfork is released. Wild shifts in difficulty can affect this number severely. Also, gradual changes in blocktimes due to longer-term adjustments in difficulty can affect the timing of difficulty bomb epochs. This affects the usability of the network but unlikely to have security ramifications.
//!
//! James Hancock (@madeoftin), "EIP-3554: Difficulty Bomb Delay to December 2021," Ethereum Improvement Proposals, no. 3554, May 2021. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-3554>.

use crate::eip::Eip;

/// EIP-3554: Difficulty Bomb Delay to December 2021.
pub struct Eip3554;

impl Eip for Eip3554 {
    const NUMBER: u32 = 3554;
}
