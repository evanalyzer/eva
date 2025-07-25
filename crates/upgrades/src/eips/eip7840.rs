//! EIP-7840: Add blob schedule to EL config files.
//!
//! ## Abstract
//!
//! Add a new object to client configuration files `blobSchedule` which lists the
//! target blob count per block and max blob count per block for each fork.
//!
//! ## Motivation
//!
//! - ensure there is a way to dynamically adjust the target and max blob counts per
//!   block
//! - ensure there is a way to dynamically adjust the blob base fee update fraction
//! - avoid complex handshake over engine API
//!
//! ## Specification
//!
//! Extend client configuration files with the object `blobSchedule`, which has the
//! following shape:
//!
//! ```json
//! "blobSchedule": {
//!   "cancun": {
//!     "target": 3,
//!     "max": 6,
//!     "baseFeeUpdateFraction": 3338477
//!   },
//!   "prague": {
//!     "target": 6,
//!     "max": 9,
//!     "baseFeeUpdateFraction": 5007716
//!   }
//! }
//! ```
//!
//! Clients must configure the target, max and baseFeeUpdateFraction per-fork. The behavior
//! when the configuration is missing or incomplete for a fork is undefined. Clients
//! are free to choose how to handle this situation.
//!
//! ## Rationale
//!
//! Although maintaining the target and max blob only in the consensus client is
//! desirable, we acknowledge the reality that execution clients need these values
//! for various activities. For example, the `eth_feeHistory` RPC method returns a
//! field `blobGasUsedRatio` that does require the max, even though the core
//! protocol doesn't specifically need such value. Passing this value over the
//! engine API every block seem overkill so we believe a configuration value is a
//! good middle ground. Additionally, the `baseFeeUpdateFraction` parameter was added to adjust the responsiveness of blob gas pricing per fork.
//!
//! ## Backwards Compatibility
//!
//! No backward compatibility issues found.
//!
//! ## Security Considerations
//!
//! No security considerations found.
//!
//! lightclient (@lightclient), "EIP-7840: Add blob schedule to EL config files," Ethereum Improvement Proposals, no. 7840, December 2024. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-7840>.

use crate::eip::Eip;

/// EIP-7840: Add blob schedule to EL config files.
pub struct Eip7840;

impl Eip for Eip7840 {
    const NUMBER: u32 = 7840;
}
