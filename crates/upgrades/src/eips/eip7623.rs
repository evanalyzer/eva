//! EIP-7623: Increase calldata cost.
//!
//! ## Abstract
//!
//! The current calldata pricing permits EL payloads of up to 7.15 MB, while the average size is much smaller at around 100 KB.
//! This EIP proposes adjusting the calldata cost to reduce the maximum possible block size and its variance without negatively impacting regular users.
//! This is achieved by increasing calldata costs for transactions that predominantly post data.
//!
//! ## Motivation
//!
//! The block gas limit has not been increased since [EIP-1559](./eip-1559.md), while the average size of blocks has continuously increased due to the growing number of rollups posting data to Ethereum. Moreover, calldata costs have remained unchanged since [EIP-2028](./eip-2028).
//! [EIP-4844](./eip-4844.md) introduces blobs as a preferred method for data availability (DA).
//! This transition demands a reevaluation of calldata pricing, especially in order to address the disparity between average and maximum block sizes.
//! By introducing a floor cost dependent on the ratio of gas spent on EVM operations to calldata, this proposal aims to reduce the maximum block size to make room for additional blobs or potential block gas limit increases.
//!
//! ## Specification
//!
//! | Parameter                    | Value |
//! | ---------------------------- | ----- |
//! | `STANDARD_TOKEN_COST`        | `4`   |
//! | `TOTAL_COST_FLOOR_PER_TOKEN` | `10`  |
//!
//! Let `tokens_in_calldata = zero_bytes_in_calldata + nonzero_bytes_in_calldata * 4`.
//!
//! Let `isContractCreation` be a boolean indicating the respective event.
//!
//! Let `execution_gas_used` be the gas used for EVM execution with the gas refund subtracted.
//!
//! Let `INITCODE_WORD_COST` be 2 as defined in [EIP-3860](./eip-3860.md).
//!
//! The current formula for determining the total gas used per transaction (`tx.gasUsed`) is equivalent to:
//!
//! ```python
//! tx.gasUsed = (
//!     21000
//!     + STANDARD_TOKEN_COST * tokens_in_calldata
//!     + execution_gas_used
//!     + isContractCreation * (32000 + INITCODE_WORD_COST * words(calldata))
//! )
//! ```
//!
//! The formula for determining the gas used per transaction changes to:
//!
//! ```python
//! tx.gasUsed = (
//!     21000
//!     +
//!     max(
//!         STANDARD_TOKEN_COST * tokens_in_calldata
//!         + execution_gas_used
//!         + isContractCreation * (32000 + INITCODE_WORD_COST * words(calldata)),
//!         TOTAL_COST_FLOOR_PER_TOKEN * tokens_in_calldata
//!     )
//! )
//! ```
//!
//! Any transaction with a gas limit below `21000 + TOTAL_COST_FLOOR_PER_TOKEN * tokens_in_calldata` or below its intrinsic gas cost (take the maximum of these two calculations) is considered invalid. This limitation exists because transactions must cover the floor price of their calldata without relying on the execution of the transaction. There are valid cases where `gasUsed` will be below this floor price, but the floor price needs to be reserved in the transaction gas limit.
//!
//! ## Rationale
//!
//! The current maximum EL payload size is approximately 1.79 MB (`30_000_000/16`). It is possible to create payloads filled with zero bytes that expand to 7.15 MB. However, since blocks are typically compressed with Snappy at the P2P layer, zero-byte-heavy EL payloads generally compress to under 1.79 MB. The implementation of [EIP-4844](./eip-4844.md) increased the maximum possible compressed block size to approximately 2.54 MB.
//!
//! This proposal aims to increase the cost of calldata to 10/40 gas for transactions that do not exceed a certain threshold of gas spent on EVM operations relative to gas spent on calldata. This change will significantly reduce the maximum block size by limiting the size of data-heavy transactions that can fit into a single block. By increasing calldata costs from 4/16 to 10/40 gas per byte, for data-heavy transactions this EIP aims to reduce the possible EL payload size to approximately 0.72 MB (`30_000_000/40`) without affecting the majority of users. Other adversarial block constructions can have a non-compressible EL payload size of approximately 1.26MiB.
//!
//! Notably, regular users (e.g. sending ETH/Tokens/NFTs, engaging in `DeFi`, social media, restaking, bridging, etc.), who do not use calldata predominantly for DA, may remain unaffected.
//! The calldata cost for transactions involving significant EVM computation remains at 4/16 gas per byte, so those transactions are unaffected.
//!
//! ## Backwards Compatibility
//!
//! This is a backwards incompatible gas repricing that requires a scheduled network upgrade.
//!
//! Wallet developers and node operators MUST update gas estimation handling to accommodate the new calldata cost rules. Specifically:
//!
//! 1. **Wallets**: Wallets using `eth_estimateGas` MUST be updated to ensure that they correctly account for the `TOTAL_COST_FLOOR_PER_TOKEN` parameter. Failure to do so could result in underestimating gas, leading to failed transactions.
//!
//! 2. **Node Software**: RPC methods such as `eth_estimateGas` MUST incorporate the updated formula for gas calculation. Node developers MUST ensure compatibility with the updated calldata pricing logic.
//!
//! Users can maintain their usual workflows without modification, as wallet and RPC updates will handle these changes.
//!
//! ## Security Considerations
//!
//! As the maximum possible block size is reduced, no security concerns have been raised.
//!
//! In some cases, it might seem advantageous to combine two transactions into one to reduce costs. For example, bundling a transaction that relies heavily on calldata but minimally on EVM resources with another that does the opposite. However, this is not a significant concern for several reasons:
//!
//! 1. This type of bundling is already possible today. Merging multiple transactions can save the 21,000 gas cost for each additional transaction beyond the first, a feature explicitly supported in [ERC-4337](./eip-4337.md).
//! 2. Such bundling does not compromise the block size reduction objectives of this EIP.
//! 3. In practice, transaction bundling is often impractical due to challenges such as trust and coordination requirements.
//!
//! These factors ensure that transaction bundling does not pose a significant issue.
//!
//! Toni Wahrstätter (@nerolation), Vitalik Buterin (@vbuterin), "EIP-7623: Increase calldata cost," Ethereum Improvement Proposals, no. 7623, February 2024. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-7623>.

use crate::eip::Eip;

/// EIP-7623: Increase calldata cost.
pub struct Eip7623;

impl Eip for Eip7623 {
    const NUMBER: u32 = 7623;
}
