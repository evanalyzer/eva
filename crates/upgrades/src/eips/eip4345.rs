//! EIP-4345: Diffictly Bomb Delay to June 2022.
//!
//! ## Abstract
//! Starting with `FORK_BLOCK_NUMBER` the client will calculate the difficulty based on a fake block number suggesting to the client that the difficulty bomb is adjusting 10,700,000 blocks later than the actual block number.
//!
//! ## Motivation
//! Targeting for The Merge to occur before June 2022. If it is not ready by then, the bomb can be delayed further.
//!
//! ## Specification
//! #### Relax Difficulty with Fake Block Number
//! For the purposes of `calc_difficulty`, simply replace the use of `block.number`, as used in the exponential ice age component, with the formula:
//! ```py
//!     fake_block_number = max(0, block.number - 10_700_000) if block.number >= FORK_BLOCK_NUMBER else block.number
//! ```
//! ## Rationale
//!
//! The following script predicts a ~0.1 second delay to block time by June 2022 and a ~0.5 second delay by July 2022. This gives reason to address because the effect will be seen, but not so much urgency we don't have space to work around if needed.
//!
//! ```python
//! def predict_diff_bomb_effect(current_blknum, current_difficulty, block_adjustment, months):
//!     '''
//!     Predicts the effect on block time (as a ratio) in a specified amount of months in the future.
//!     Vars used for predictions:
//!     current_blknum = 13423376 # Oct 15, 2021
//!     current_difficulty = 9545154427582720
//!     block adjustment = 10700000
//!     months = 7.5 # June 2022
//!     months = 8.5 # July 2022
//!     '''
//!     blocks_per_month = (86400 * 30) // 13.3
//!     future_blknum = current_blknum + blocks_per_month * months
//!     diff_adjustment = 2 ** ((future_blknum - block_adjustment) // 100000 - 2)
//!     diff_adjust_coeff = diff_adjustment / current_difficulty * 2048
//!     return diff_adjust_coeff
//!
//!
//! diff_adjust_coeff = predict_diff_bomb_effect(13423376,9545154427582720,10700000,7.5)
//! diff_adjust_coeff = predict_diff_bomb_effect(13423376,9545154427582720,10700000,8.5)
//! ```
//!
//! ## Backwards Compatibility
//! No known backward compatibility issues.
//!
//! ## Security Considerations
//! Misjudging the effects of the difficulty can mean longer blocktimes than anticipated until a hardfork is released. Wild shifts in difficulty can affect this number severely. Also, gradual changes in blocktimes due to longer-term adjustments in difficulty can affect the timing of difficulty bomb epochs. This affects the usability of the network but unlikely to have security ramifications.
//!
//! In this specific instance, it is possible that the network hashrate drops considerably before The Merge, which could accelerate the timeline by which the bomb is felt in block times. The offset value chosen aimed to take this into account.
//!
//! Tim Beiko (@timbeiko), James Hancock (`@MadeOfTin`), Thomas Jay Rush (@tjayrush), "EIP-4345: Difficulty Bomb Delay to June 2022," Ethereum Improvement Proposals, no. 4345, October 2021. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-4345>.

use crate::eip::Eip;

/// EIP-4345: Diffictly Bomb Delay to June 2022.
pub struct Eip4345;

impl Eip for Eip4345 {
    const NUMBER: u32 = 4345;
}
