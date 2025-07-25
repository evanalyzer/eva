//! EIP-6110: Supply validator deposits on chain.
//!
//! ## Abstract
//!
//! Appends validator deposits to the Execution Layer block structure. This shifts responsibility of deposit inclusion and validation to the Execution Layer and removes the need for deposit (or `eth1data`) voting from the Consensus Layer.
//!
//! Validator deposits list supplied in a block is obtained by parsing deposit contract log events emitted by each deposit transaction included in a given block.
//!
//! ## Motivation
//!
//! Validator deposits are a core component of the proof-of-stake consensus mechanism. This EIP allows for an in-protocol mechanism of deposit processing on the Consensus Layer and eliminates the proposer voting mechanism utilized currently. This proposed mechanism relaxes safety assumptions and reduces complexity of client software design, contributing to the security of the deposits flow. It also improves validator UX.
//!
//! Advantages of in-protocol deposit processing consist of but are not limit to the following:
//!
//! * Significant increase of deposits security by supplanting proposer voting. With the proposed in-protocol mechanism, an honest online node can't be convinced to process fake deposits even when more than 2/3 portion of stake is adversarial.
//! * Decrease of delay between submitting deposit transaction on Execution Layer and its processing on Consensus Layer. That is, ~13 minutes with in-protocol deposit processing compared to ~12 hours with the existing mechanism.
//! * Eliminate beacon block proposal dependency on JSON-RPC API data polling that suffers from failures caused by inconsistencies between JSON-RPC API implementations and dependency of API calls processing on the inner state (e.g. syncing) of client software.
//! * Eliminate requirement to maintain and distribute deposit contract snapshots ([EIP-4881](./eip-4881.md)).
//! * Reduction of design and engineering complexity of Consensus Layer client software on a component that has proven to be brittle.
//!
//! ## Specification
//!
//! ### Execution Layer
//!
//! #### Constants
//!
//! | Name | Value | Comment |
//! | - | - | - |
//! |`DEPOSIT_REQUEST_TYPE` | `b'0'` | The [EIP-7685](./eip-7685.md) request type byte for deposit operation |
//!
//! #### Configuration
//!
//! | Name | Value | Comment |
//! | - | - | - |
//! |`DEPOSIT_CONTRACT_ADDRESS` | `0x00000000219ab540356cbb839cbe05303d7705fa` | Mainnet |
//! |`DEPOSIT_EVENT_SIGNATURE_HASH` | `0x649bbc62d0e31342afea4e5cd82d4049e7e1ee912fc0889aa790803be39038c5` | |
//!
//! `DEPOSIT_CONTRACT_ADDRESS`, `DEPOSIT_EVENT_SIGNATURE_HASH` parameters **MUST** be included into client software binary distribution.
//!
//! #### Definitions
//!
//! * **`FORK_BLOCK`** -- the first block in a blockchain after this EIP has been activated.
//!
//! #### Deposit request
//!
//! The structure denoting the new deposit request consists of the following fields:
//!
//! 1. `pubkey: Bytes48`
//! 2. `withdrawal_credentials: Bytes32`
//! 3. `amount: uint64`
//! 4. `signature: Bytes96`
//! 5. `index: uint64`
//!
//! Deposits are a type of [EIP-7685](./eip-7685.md) request, with the following encoding:
//!
//! ```python
//! request_type = DEPOSIT_REQUEST_TYPE
//! request_data = get_deposit_request_data(block.receipts)
//! ```
//!
//! #### Block validity
//!
//! Beginning with the `FORK_BLOCK`, each deposit accumulated in the block **MUST** appear in the EIP-7685 requests list
//! in the order they appear in the logs. To illustrate:
//!
//! ```python
//! def parse_deposit_data(deposit_event_data) -> bytes[]:
//!   """
//!   Parses deposit data from DepositContract.DepositEvent data
//!   """
//!   pass
//!
//! def is_valid_deposit_event_data(deposit_event_data: bytes) -> bool:
//!     """
//!     Verifies the layout of the DepositEvent. Returns `False` if the layout is unsupported,
//!     `True` if the layout is of the expected format.
//!     """
//!     if len(deposit_event_data) != 576:
//!         return False
//!
//!     pubkey_offset = int.from_bytes(deposit_event_data[0:32], byteorder='big', signed=False)
//!     withdrawal_credentials_offset = int.from_bytes(deposit_event_data[32:64], byteorder='big', signed=False)
//!     amount_offset = int.from_bytes(deposit_event_data[64:96], byteorder='big', signed=False)
//!     signature_offset = int.from_bytes(deposit_event_data[96:128], byteorder='big', signed=False)
//!     index_offset = int.from_bytes(deposit_event_data[128:160], byteorder='big', signed=False)
//!
//!     if (
//!         pubkey_offset != 160
//!         or withdrawal_credentials_offset != 256
//!         or amount_offset != 320
//!         or signature_offset != 384
//!         or index_offset != 512
//!     ):
//!         return False
//!
//!     # These sizes are the sizes of the relevant data
//!     pubkey_size = int.from_bytes(deposit_event_data[pubkey_offset:pubkey_offset+32], byteorder='big', signed=False)
//!     withdrawal_credentials_size = int.from_bytes(deposit_event_data[withdrawal_credentials_offset:withdrawal_credentials_offset+32], byteorder='big', signed=False)
//!     amount_size = int.from_bytes(deposit_event_data[amount_offset:amount_offset+32], byteorder='big', signed=False)
//!     signature_size = int.from_bytes(deposit_event_data[signature_offset:signature_offset+32], byteorder='big', signed=False)
//!     index_size = int.from_bytes(deposit_event_data[index_offset:index_offset+32], byteorder='big', signed=False)
//!
//!     return (
//!         pubkey_size == 48
//!         and withdrawal_credentials_size == 32
//!         and amount_size == 8
//!         and signature_size == 96
//!         and index_size == 8
//!     )
//!
//! def event_data_to_deposit_request(deposit_event_data) -> bytes:
//!     deposit_data = parse_deposit_data(deposit_event_data)
//!     pubkey = Bytes48(deposit_data[0])
//!     withdrawal_credentials = Bytes32(deposit_data[1])
//!     amount = deposit_data[2]   # 8 bytes uint64 LE
//!     signature = Bytes96(deposit_data[3])
//!     index = deposit_data[4]    # 8 bytes uint64 LE
//!
//!     return pubkey + withdrawal_credentials + amount + signature + index
//!
//! def get_deposit_request_data(receipts)
//!     # Retrieve all deposits made in the block
//!     deposit_requests = []
//!     for receipt in receipts:
//!         for log in receipt.logs:
//!             if log.address == DEPOSIT_CONTRACT_ADDRESS:
//!                 if len(log.topics) > 0 and log.topics[0] == DEPOSIT_EVENT_SIGNATURE_HASH:
//!                     assert is_valid_deposit_event_data(log.data), 'invalid deposit log: unsupported data layout'
//!                     deposit_request = event_data_to_deposit_request(log.data)
//!                     deposit_requests.append(deposit_request)
//!
//!     # Concatenate list of deposit request data
//!     return b''.join(deposit_requests)
//! ```
//!
//! ### Consensus layer
//!
//! Consensus layer changes can be summarized into the following list:
//!
//! 1. `ExecutionRequests` is extended with a new `deposit_requests` field to accommodate deposit requests list.
//! 2. `BeaconState` is appended with `deposit_requests_start_index` used to switch from the former deposit mechanism to the new one.
//! 3. As a part of transition logic a new beacon block validity condition is added to constrain the usage of `Eth1Data` poll.
//! 4. A new `process_deposit_request` function is added to the block processing routine to handle `deposit_requests` processing.
//! 5. Validator guide provides a logic switching off `Eth1Data` poll once transition is completed.
//!
//! Detailed consensus layer specification can be found in following documents:
//!
//! * [`electra/beacon-chain.md`](https://github.com/ethereum/consensus-specs/blob/eba62dbf00132dfdc97fbfab663a99cb23b9e8f1/specs/electra/beacon-chain.md) -- state transition.
//! * [`electra/validator.md`](https://github.com/ethereum/consensus-specs/blob/eba62dbf00132dfdc97fbfab663a99cb23b9e8f1/specs/electra/validator.md) -- validator guide.
//! * [`electra/fork.md`](https://github.com/ethereum/consensus-specs/blob/eba62dbf00132dfdc97fbfab663a99cb23b9e8f1/specs/electra/fork.md) -- EIP activation.
//!
//! #### Validator index invariant
//!
//! Due to the large follow distance of `Eth1Data` poll an index of a new validator assigned during deposit processing remains the same across different branches of a block tree, i.e. with existing mechanism `(pubkey, index)` cache utilized by consensus layer clients is re-org resilient. The new deposit machinery breaks this invariant and consensus layer clients will have to deal with a fact that a validator index becomes fork dependent, i.e. a validator with the same `pubkey` can have different indexes in different block tree branches.
//!
//! Detailed [analysis](../assets/eip-6110/pubkey_to_index_cache_analysis.md) shows that `process_deposit` function is *the only* place requiring a fork dependent `(pubkey, index)` cache.
//!
//! #### `Eth1Data` poll deprecation
//!
//! Consensus layer clients will be able to remove `Eth1Data` poll mechanism in an uncoordinated fashion once transition period is finished. The transition period is considered as finished when a network reaches the point where `state.eth1_deposit_index == state.deposit_requests_start_index`.
//!
//! ## Rationale
//!
//! ### `index` field
//!
//! Deposit `index` is used to deterministically initialize `deposit_requests_start_index` in the `BeaconState`, this prevents same deposit from being applied twice during `Eth1Data` poll deprecation.
//!
//! ### Not limiting the size of deposit operations list
//!
//! The list is unbounded because of negligible data complexity and absence of potential `DoS` vectors. See [Security Considerations](#security-considerations) for more details.
//!
//! ### Filtering events by `DEPOSIT_CONTRACT_ADDRESS` and `DEPOSIT_EVENT_SIGNATURE_HASH`
//!
//! Depending on the design, Deposit smart contract can emit different type of events when deposit is being processed. For instance, Deposit smart contract on Sepolia emits `Transfer` in addition to `DepositEvent`. Thus it is important to filter out irrelevant events.
//!
//! ## Backwards Compatibility
//!
//! This EIP introduces backwards incompatible changes to the block structure and block validation rule set. But neither of these changes break anything related to user activity and experience.
//!
//! ## Security Considerations
//!
//! ### Data complexity
//!
//! At the time of the latest update of this document, the total number of submitted deposits is 1,899,120 which is 348MB of deposit data. Assuming frequency of deposit transactions remains the same, historic chain data complexity induced by this EIP can be estimated as 84MB per year which is negligible in comparison to other historical data.
//!
//! After the beacon chain launch in December 2020, the biggest observed spike in a number of submitted deposits was on June 1, 2023. More than 12,000 deposit transactions were submitted during 24 hours which on average is less than 2 deposit, or 384 bytes of data, per block.
//!
//! Considering the above, we conclude that data complexity introduced by this proposal is negligible.
//!
//! ### `DoS` vectors
//!
//! The code in the deposit contract costs 15,650 gas to run in the cheapest case (when all storage slots are hot and only a single leaf has to be modified). Some deposits in a batch deposit are more expensive, but those costs, when amortized over a large number of deposits, are small at around ~1,000 gas per deposit. Under current gas pricing rules an extra 6,900 gas is charged to make a `CALL` that transfers ETH, this is a case of inefficient gas pricing and may be reduced in the future. For future robustness the beacon chain needs to be able to withstand 1,916 deposits in a 30M gas block (15,650 gas per deposit). The limit under current rules is less than 1,271 deposits in a 30M gas block.
//!
//! #### Execution layer
//!
//! With 1 ETH as a minimum deposit amount, the lowest cost of a byte of deposit data is 1 ETH/192 ~ 5,208,333 Gwei. This is several orders of magnitude higher than the cost of a byte of transaction's calldata, thus adding deposit operations to a block does not increase `DoS` attack surface of the execution layer.
//!
//! #### Consensus layer
//!
//! The most consuming computation of deposit processing is signature verification. Its complexity is bounded by a maximum number of deposits per block which is around 1,271 with 30M gas block at the moment. So, it is ~1,271 signature verifications which is roughly ~1.2 seconds of processing (without optimisations like batched signatures verification). An attacker would need to spend 1,000 ETH to slow down block processing by a second which isn't sustainable and viable attack long term.
//!
//! An optimistically syncing node may be susceptible to a more severe attack scenario. Such a node can't validate a list of deposits provided in a payload which makes it possible for attacker to include as many deposits as the limitation allows to. Currently, it is 8,192 deposits (1.5MB of data) with rough processing time of 8s. Considering an attacker would need to sign off on this block with its crypto economically viable signature (which requires building an alternative chain and feeding it to a syncing node), this attack vector is not considered as viable as it can't result in a significant slow down of a sync process.
//!
//! ### Optimistic sync
//!
//! An optimistically syncing node have to rely on the honest majority assumption. That is, if adversary is powerful enough to finalize a deposit sequence, a syncing node will have to apply these deposits disregarding the validity of deposit requests with respect to the execution of a given block. Thus, an adversary that can finalize an invalid chain can also convince an honest node to accept fake deposits. The same is applicable to the validity of execution layer world state today and a new deposit processing design is within boundaries of the existing security model in that regard.
//!
//! Online nodes can't be tricked into this situation because their execution layer validates supplied deposits with respect to the block execution.
//!
//! ### Weak subjectivity period
//!
//! This EIP removes a hard limit on a number of deposits per epoch and makes a block gas limit the only limitation on this number. That is, the limit on deposits per epoch shifts from `MAX_DEPOSITS * SLOTS_PER_EPOCH = 512` to `max_deposits_per_30m_gas_block * SLOTS_PER_EPOCH ~ 32,768` at 30M gas block (we consider `max_deposits_per_30m_gas_block = 1,024` for simplicity).
//!
//! This change affects a number of top ups per epoch which is one of the inputs to the weak subjectivity period computation. One can top up own validators to instantly increase a portion of stake it owns with respect to those validators that are leaking. [The analysis](../assets/eip-6110/ws_period_analysis.md) does not demonstrate significant reduction of a weak subjectivity period sizes. Moreover, such an attack is not considered as viable because it requires a decent portion of stake to be burned as one of preliminaries.
//!
//! Mikhail Kalinin (@mkalinin), Danny Ryan (@djrtwo), Peter Davies (@petertdavies), "EIP-6110: Supply validator deposits on chain," Ethereum Improvement Proposals, no. 6110, December 2022. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-6110>.

use crate::eip::Eip;

/// EIP-6110: Supply validator deposits on chain.
pub struct Eip6110;

impl Eip for Eip6110 {
    const NUMBER: u32 = 6110;
}
