//! EIP-5133: Delaying Difficulty Bomb to mid-September 2022.
//!
//! ## Abstract
//! Starting with `FORK_BLOCK_NUMBER` the client will calculate the difficulty based on a fake block number suggesting to the client that the difficulty bomb is adjusting 11,400,000 blocks later than the actual block number.
//!
//! ## Motivation
//! To avoid network degradation due to a premature activation of the difficulty bomb.
//!
//! ## Specification
//! #### Relax Difficulty with Fake Block Number
//! For the purposes of `calc_difficulty`, simply replace the use of `block.number`, as used in the exponential ice age component, with the formula:
//! ```py
//! fake_block_number = max(0, block.number - 11_400_000) if block.number >= FORK_BLOCK_NUMBER else block.number
//! ```
//! ## Rationale
//!
//! The following script predicts the bomb will go off at block 15530314, which is expected to be mined around mid-September.
//!
//! ```python
//! import math
//! def predict_bomb_block(current_difficulty, diff_adjust_coeff, block_adjustment):
//!     '''
//!     Predicts the block number at which the difficulty bomb will become noticeable.
//!
//!     current_difficulty: the current difficulty
//!     diff_adjust_coeff: intuitively, the percent increase in work that miners have to exert to find a PoW
//!     block_adjustment: the number of blocks to delay the bomb by
//!     '''
//!     return round(block_adjustment + 100000 * (2 + math.log2(diff_adjust_coeff * current_difficulty // 2048)))
//!
//! # current_difficulty = 13891609586928851 (Jun 01, 2022)
//! # diff_adjust_coeff = 0.1 (historically, the bomb is noticeable when the coefficient is >= 0.1)
//! # block_adjustment = 11400000
//! print(predict_bomb_block(13891609586928851, 0.1, 11400000))
//! ```
//!
//! Precise increases in block times are very difficult to predict (especially after the bomb is noticeable).
//! However, based on past manifestations of the bomb, we can anticipate 0.1s delays by mid-September and 0.6-1.2s delays by early October.
//!
//! ## Backwards Compatibility
//! No known backward compatibility issues.
//!
//! ## Security Considerations
//! Misjudging the effects of the difficulty can mean longer blocktimes than anticipated until a hardfork is released. Wild shifts in difficulty can affect this number severely. Also, gradual changes in blocktimes due to longer-term adjustments in difficulty can affect the timing of difficulty bomb epochs. This affects the usability of the network but unlikely to have security ramifications.
//!
//! In this specific instance, it is possible that the network hashrate drops considerably before The Merge, which could accelerate the timeline by which the bomb is felt in block times. The offset value chosen aims to take this into account.
//!
//! Tomasz Kajetan Stanczak (@tkstanczak), Eric Marti Haynes (@ericmartihaynes), Josh Klopfenstein (@joshklop), Abhimanyu Nag (`@AbhiMan1601`), "EIP-5133: Delaying Difficulty Bomb to mid-September 2022," Ethereum Improvement Proposals, no. 5133, June 2022. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-5133>.

use crate::eip::Eip;

/// EIP-5133: Delaying Difficulty Bomb to mid-September 2022.
pub struct Eip5133;

impl Eip for Eip5133 {
    const NUMBER: u32 = 5133;
}
