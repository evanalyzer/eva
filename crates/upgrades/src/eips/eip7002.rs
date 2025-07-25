//! EIP-7002: Execution layer triggerable withdrawals.
//!
//! ## Abstract
//!
//! Adds a new mechanism to allow validators to trigger withdrawals and exits from their execution layer (0x01) withdrawal credentials.
//!
//! These new execution layer exit messages are appended to the execution layer block and then processed by the consensus layer.
//!
//! ## Motivation
//!
//! Validators have two keys -- an active key and a withdrawal credential. The active key takes the form of a BLS key, whereas the withdrawal credential can either be a BLS key (0x00) or an execution layer address (0x01). The active key is "hot", actively signing and performing validator duties, whereas the withdrawal credential can remain "cold", only performing limited operations in relation to withdrawing and ownership of the staked ETH. Due to this security relationship, the withdrawal credential ultimately is the key that owns the staked ETH and any rewards.
//!
//! As currently specified, only the active key can initiate a validator exit. This means that in any non-standard custody relationships (i.e. active key is separate entity from withdrawal credentials), that the ultimate owner of the funds -- the possessor of the withdrawal credentials -- cannot independently choose to exit and begin the withdrawal process. This leads to either trust issues (e.g. ETH can be "held hostage" by the active key owner) or insufficient work-arounds such as pre-signed exits. Additionally, in the event that active keys are lost, a user should still be able to recover their funds by using their cold withdrawal credentials.
//!
//! To ensure that the withdrawal credentials (owned by both EOAs and smart contracts) can trustlessly control the destiny of the staked ETH, this specification enables exits triggerable by 0x01 withdrawal credentials.
//!
//! Note, 0x00 withdrawal credentials can be changed into 0x01 withdrawal credentials with a one-time signed message. Thus any functionality enabled for 0x01 credentials is defacto enabled for 0x00 credentials.
//!
//! ## Specification
//!
//! ### Configuration
//!
//! | Name | Value | Comment |
//! | - | - | - |
//! | `WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS` | `0x00000961Ef480Eb55e80D19ad83579A64c007002` | Where to call and store relevant details about exit / partial withdrawal mechanism |
//! | `WITHDRAWAL_REQUEST_TYPE` | `0x01` | The [EIP-7685](./eip-7685.md) type prefix for withdrawal request |
//! | `SYSTEM_ADDRESS` | `0xfffffffffffffffffffffffffffffffffffffffe` | Address used to invoke system operation on contract
//! | `EXCESS_WITHDRAWAL_REQUESTS_STORAGE_SLOT` | 0 | |
//! | `WITHDRAWAL_REQUEST_COUNT_STORAGE_SLOT` | 1 | |
//! | `WITHDRAWAL_REQUEST_QUEUE_HEAD_STORAGE_SLOT` | 2 | Pointer to head of the withdrawal request message queue |
//! | `WITHDRAWAL_REQUEST_QUEUE_TAIL_STORAGE_SLOT` | 3 | Pointer to the tail of the withdrawal request message queue|
//! | `WITHDRAWAL_REQUEST_QUEUE_STORAGE_OFFSET` | 4 | The start memory slot of the in-state withdrawal request message queue|
//! | `MAX_WITHDRAWAL_REQUESTS_PER_BLOCK` | 16 | Maximum number of withdrawal requests that can be dequeued into a block |
//! | `TARGET_WITHDRAWAL_REQUESTS_PER_BLOCK` | 2 | |
//! | `MIN_WITHDRAWAL_REQUEST_FEE` | 1 | |
//! | `WITHDRAWAL_REQUEST_FEE_UPDATE_FRACTION` | 17 | |
//! | `EXCESS_INHIBITOR` | `2**256-1` | Excess value used to compute the fee before the first system call |
//!
//! ### Execution layer
//!
//! #### Definitions
//!
//! * **`FORK_BLOCK`** -- the first block in a blockchain after this EIP has been activated.
//!
//! #### Withdrawal request operation
//!
//! The new withdrawal request operation is an [EIP-7685](./eip-7685.md) request
//! with type `0x01` and consists of the following fields:
//!
//! 1. `source_address`: `Bytes20`
//! 2. `validator_pubkey`: `Bytes48`
//! 3. `amount:` `uint64`
//!
//! The [EIP-7685](./eip-7685.md) encoding of a withdrawal request is computed as follows.
//! Note that `amount` is returned by the contract little-endian, and must be encoded as such.
//!
//! ```python
//! request_type = WITHDRAWAL_REQUEST_TYPE
//! request_data = read_withdrawal_requests()
//! ```
//!
//! #### Withdrawal Request Contract
//!
//! The contract has three different code paths, which can be summarized at a high level as follows:
//!
//! 1. Add withdrawal request - requires a `56` byte input, the validator's public
//!    key concatenated with a big-endian `uint64` amount value.
//! 2. Fee getter - if the input length is zero, return the current fee required to add a withdrawal request.
//! 3. System process - if called by system address, pop off the withdrawal requests for the current block from the queue.
//!
//! ##### Add Withdrawal Request
//!
//! If call data input to the contract is exactly `56` bytes, perform the following:
//!
//! * Ensure enough ETH was sent to cover the current withdrawal request fee (`check_fee()`)
//! * Increase withdrawal request count by 1 for the current block (`increment_count()`)
//! * Insert a withdrawal request into the queue for the source address and validator pubkey (`insert_withdrawal_request_into_queue()`)
//!
//! Specifically, the functionality is defined in pseudocode as the function `add_withdrawal_request()`:
//!
//! ```python
//! def add_withdrawal_request(Bytes48: validator_pubkey, uint64: amount):
//!     """
//!     Add withdrawal request adds new request to the withdrawal request queue, so long as a sufficient fee is provided.
//!     """
//!
//!     # Verify sufficient fee was provided.
//!     fee = get_fee()
//!     require(msg.value >= fee, 'Insufficient value for fee')
//!
//!     # Increment withdrawal request count.
//!     count = sload(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, WITHDRAWAL_REQUEST_COUNT_STORAGE_SLOT)
//!     sstore(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, WITHDRAWAL_REQUEST_COUNT_STORAGE_SLOT, count + 1)
//!
//!     # Insert into queue.
//!     queue_tail_index = sload(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, WITHDRAWAL_REQUEST_QUEUE_TAIL_STORAGE_SLOT)
//!     queue_storage_slot = WITHDRAWAL_REQUEST_QUEUE_STORAGE_OFFSET + queue_tail_index * 3
//!     sstore(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, queue_storage_slot, msg.sender)
//!     sstore(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, queue_storage_slot + 1, validator_pubkey[0:32])
//!     sstore(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, queue_storage_slot + 2, validator_pubkey[32:48] ++ uint64_to_little_endian(amount))
//!     sstore(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, WITHDRAWAL_REQUEST_QUEUE_TAIL_STORAGE_SLOT, queue_tail_index + 1)
//! ```
//!
//! ###### Fee calculation
//!
//! The following pseudocode can compute the cost an individual withdrawal request, given a certain number of excess withdrawal requests.
//!
//! ```python
//! def get_fee() -> int:
//!     excess = sload(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, EXCESS_WITHDRAWAL_REQUESTS_STORAGE_SLOT)
//!     require(excess != EXCESS_INHIBITOR, 'Inhibitor still active')
//!     return fake_exponential(
//!         MIN_WITHDRAWAL_REQUEST_FEE,
//!         excess,
//!         WITHDRAWAL_REQUEST_FEE_UPDATE_FRACTION
//!     )
//!
//! def fake_exponential(factor: int, numerator: int, denominator: int) -> int:
//!     i = 1
//!     output = 0
//!     numerator_accum = factor * denominator
//!     while numerator_accum > 0:
//!         output += numerator_accum
//!         numerator_accum = (numerator_accum * numerator) // (denominator * i)
//!         i += 1
//!     return output // denominator
//! ```
//!
//! ##### Fee Getter
//!
//! When the input to the contract is length zero, interpret this as a get request for the current fee, i.e. the contract returns the result of `get_fee()`.
//!
//! ##### System Call
//!
//! At the end of processing any execution block starting from the `FORK_BLOCK` (i.e. after processing all transactions and after performing the block body withdrawal requests validations), call `WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS` as `SYSTEM_ADDRESS` with no calldata. The invocation triggers the following:
//!
//! * The contract's queue is updated based on withdrawal requests dequeued and the withdrawal requests queue head/tail are reset if the queue has been cleared (`dequeue_withdrawal_requests()`)
//! * The contract's excess withdrawal requests are updated based on usage in the current block (`update_excess_withdrawal_requests()`)
//! * The contract's withdrawal requests count is reset to 0 (`reset_withdrawal_requests_count()`)
//!
//! Each withdrawal request must appear in the EIP-7685 requests list in the exact order returned by `dequeue_withdrawal_requests()`.
//!
//! Additionally, the system call and the processing of that block must conform to the following:
//!
//! * The call has a dedicated gas limit of `30_000_000`.
//! * Gas consumed by this call does not count against the block’s overall gas usage.
//! * Both the gas limit assigned to the call and the gas consumed is excluded from any checks against the block’s gas limit.
//! * The call does not follow [EIP-1559](./eip-1559.md) fee burn semantics — no value should be transferred as part of this call.
//! * If there is no code at `WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS`, the corresponding block **MUST** be marked invalid.
//! * If the call to the contract fails or returns an error, the block **MUST** be invalidated.
//!
//! The functionality triggered by the system call is defined in pseudocode as the function `read_withdrawal_requests()`:
//!
//! ```python
//! ###################
//! # Public function #
//! ###################
//!
//! def read_withdrawal_requests():
//!     reqs = dequeue_withdrawal_requests()
//!     update_excess_withdrawal_requests()
//!     reset_withdrawal_requests_count()
//!     return ssz.serialize(reqs)
//!
//! ###########
//! # Helpers #
//! ###########
//!
//! def little_endian_to_uint64(data: bytes) -> uint64:
//!     return uint64(int.from_bytes(data, 'little'))
//!
//! def uint64_to_little_endian(num: uint64) -> bytes:
//!     return num.to_bytes(8, 'little')
//!
//! class ValidatorWithdrawalRequest(object):
//!     source_address: Bytes20
//!     validator_pubkey: Bytes48
//!     amount: uint64
//!
//! def dequeue_withdrawal_requests():
//!     queue_head_index = sload(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, WITHDRAWAL_REQUEST_QUEUE_HEAD_STORAGE_SLOT)
//!     queue_tail_index = sload(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, WITHDRAWAL_REQUEST_QUEUE_TAIL_STORAGE_SLOT)
//!     num_in_queue = queue_tail_index - queue_head_index
//!     num_dequeued = min(num_in_queue, MAX_WITHDRAWAL_REQUESTS_PER_BLOCK)
//!
//!     reqs = []
//!     for i in range(num_dequeued):
//!         queue_storage_slot = WITHDRAWAL_REQUEST_QUEUE_STORAGE_OFFSET + (queue_head_index + i) * 3
//!         source_address = address(sload(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, queue_storage_slot)[0:20])
//!         validator_pubkey = (
//!             sload(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, queue_storage_slot + 1)[0:32] + sload(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, queue_storage_slot + 2)[0:16]
//!         )
//!         amount = little_endian_to_uint64(sload(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, queue_storage_slot + 2)[16:24])
//!         req = ValidatorWithdrawalRequest(
//!             source_address=Bytes20(source_address),
//!             validator_pubkey=Bytes48(validator_pubkey),
//!             amount=uint64(amount)
//!         )
//!         reqs.append(req)
//!
//!     new_queue_head_index = queue_head_index + num_dequeued
//!     if new_queue_head_index == queue_tail_index:
//!         # Queue is empty, reset queue pointers
//!         sstore(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, WITHDRAWAL_REQUEST_QUEUE_HEAD_STORAGE_SLOT, 0)
//!         sstore(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, WITHDRAWAL_REQUEST_QUEUE_TAIL_STORAGE_SLOT, 0)
//!     else:
//!         sstore(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, WITHDRAWAL_REQUEST_QUEUE_HEAD_STORAGE_SLOT, new_queue_head_index)
//!
//!     return reqs
//!
//! def update_excess_withdrawal_requests():
//!     previous_excess = sload(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, EXCESS_WITHDRAWAL_REQUESTS_STORAGE_SLOT)
//!     if previous_excess == EXCESS_INHIBITOR:
//!         previous_excess = 0
//!
//!     count = sload(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, WITHDRAWAL_REQUEST_COUNT_STORAGE_SLOT)
//!
//!     new_excess = 0
//!     if previous_excess + count > TARGET_WITHDRAWAL_REQUESTS_PER_BLOCK:
//!         new_excess = previous_excess + count - TARGET_WITHDRAWAL_REQUESTS_PER_BLOCK
//!
//!     sstore(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, EXCESS_WITHDRAWAL_REQUESTS_STORAGE_SLOT, new_excess)
//!
//! def reset_withdrawal_requests_count():
//!     sstore(WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS, WITHDRAWAL_REQUEST_COUNT_STORAGE_SLOT, 0)
//! ```
//!
//! ##### Bytecode
//!
//! ```asm
//! caller
//! push20 0xfffffffffffffffffffffffffffffffffffffffe
//! eq
//! push1 0xcb
//! jumpi
//!
//! push1 0x11
//! push0
//! sload
//! dup1
//! push32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
//! eq
//! push2 0x01f4
//! jumpi
//!
//! push1 0x01
//! dup3
//! mul
//! push1 0x01
//! swap1
//! push0
//!
//! jumpdest
//! push0
//! dup3
//! gt
//! iszero
//! push1 0x68
//! jumpi
//!
//! dup2
//! add
//! swap1
//! dup4
//! mul
//! dup5
//! dup4
//! mul
//! swap1
//! div
//! swap2
//! push1 0x01
//! add
//! swap2
//! swap1
//! push1 0x4d
//! jump
//!
//! jumpdest
//! swap1
//! swap4
//! swap1
//! div
//! swap3
//! pop
//! pop
//! pop
//! calldatasize
//! push1 0x38
//! eq
//! push1 0x88
//! jumpi
//!
//! calldatasize
//! push2 0x01f4
//! jumpi
//!
//! callvalue
//! push2 0x01f4
//! jumpi
//!
//! push0
//! mstore
//! push1 0x20
//! push0
//! return
//!
//! jumpdest
//! callvalue
//! lt
//! push2 0x01f4
//! jumpi
//!
//! push1 0x01
//! sload
//! push1 0x01
//! add
//! push1 0x01
//! sstore
//! push1 0x03
//! sload
//! dup1
//! push1 0x03
//! mul
//! push1 0x04
//! add
//! caller
//! dup2
//! sstore
//! push1 0x01
//! add
//! push0
//! calldataload
//! dup2
//! sstore
//! push1 0x01
//! add
//! push1 0x20
//! calldataload
//! swap1
//! sstore
//! caller
//! push1 0x60
//! shl
//! push0
//! mstore
//! push1 0x38
//! push0
//! push1 0x14
//! calldatacopy
//! push1 0x4c
//! push0
//! log0
//! push1 0x01
//! add
//! push1 0x03
//! sstore
//! stop
//!
//! jumpdest
//! push1 0x03
//! sload
//! push1 0x02
//! sload
//! dup1
//! dup3
//! sub
//! dup1
//! push1 0x10
//! gt
//! push1 0xdf
//! jumpi
//!
//! pop
//! push1 0x10
//!
//! jumpdest
//! push0
//!
//! jumpdest
//! dup2
//! dup2
//! eq
//! push2 0x0183
//! jumpi
//!
//! dup3
//! dup2
//! add
//! push1 0x03
//! mul
//! push1 0x04
//! add
//! dup2
//! push1 0x4c
//! mul
//! dup2
//! sload
//! push1 0x60
//! shl
//! dup2
//! mstore
//! push1 0x14
//! add
//! dup2
//! push1 0x01
//! add
//! sload
//! dup2
//! mstore
//! push1 0x20
//! add
//! swap1
//! push1 0x02
//! add
//! sload
//! dup1
//! push32 0xffffffffffffffffffffffffffffffff00000000000000000000000000000000
//! and
//! dup3
//! mstore
//! swap1
//! push1 0x10
//! add
//! swap1
//! push1 0x40
//! shr
//! swap1
//! dup2
//! push1 0x38
//! shr
//! dup2
//! push1 0x07
//! add
//! mstore8
//! dup2
//! push1 0x30
//! shr
//! dup2
//! push1 0x06
//! add
//! mstore8
//! dup2
//! push1 0x28
//! shr
//! dup2
//! push1 0x05
//! add
//! mstore8
//! dup2
//! push1 0x20
//! shr
//! dup2
//! push1 0x04
//! add
//! mstore8
//! dup2
//! push1 0x18
//! shr
//! dup2
//! push1 0x03
//! add
//! mstore8
//! dup2
//! push1 0x10
//! shr
//! dup2
//! push1 0x02
//! add
//! mstore8
//! dup2
//! push1 0x08
//! shr
//! dup2
//! push1 0x01
//! add
//! mstore8
//! mstore8
//! push1 0x01
//! add
//! push1 0xe1
//! jump
//!
//! jumpdest
//! swap2
//! add
//! dup1
//! swap3
//! eq
//! push2 0x0195
//! jumpi
//!
//! swap1
//! push1 0x02
//! sstore
//! push2 0x01a0
//! jump
//!
//! jumpdest
//! swap1
//! pop
//! push0
//! push1 0x02
//! sstore
//! push0
//! push1 0x03
//! sstore
//!
//! jumpdest
//! push0
//! sload
//! dup1
//! push32 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
//! eq
//! iszero
//! push2 0x01cd
//! jumpi
//!
//! pop
//! push0
//!
//! jumpdest
//! push1 0x01
//! sload
//! push1 0x02
//! dup3
//! dup3
//! add
//! gt
//! push2 0x01e2
//! jumpi
//!
//! pop
//! pop
//! push0
//! push2 0x01e8
//! jump
//!
//! jumpdest
//! add
//! push1 0x02
//! swap1
//! sub
//!
//! jumpdest
//! push0
//! sstore
//! push0
//! push1 0x01
//! sstore
//! push1 0x4c
//! mul
//! push0
//! return
//!
//! jumpdest
//! push0
//! push0
//! revert
//! ```
//!
//! ##### Deployment
//!
//! The withdrawal requests contract is deployed like any other smart contract. A special synthetic address is generated by working backwards from the desired deployment transaction:
//!
//! ```json
//! {
//!   "type": "0x0",
//!   "nonce": "0x0",
//!   "to": null,
//!   "gas": "0x3d090",
//!   "gasPrice": "0xe8d4a51000",
//!   "maxPriorityFeePerGas": null,
//!   "maxFeePerGas": null,
//!   "value": "0x0",
//!   "input": "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff5f556101f880602d5f395ff33373fffffffffffffffffffffffffffffffffffffffe1460cb5760115f54807fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff146101f457600182026001905f5b5f82111560685781019083028483029004916001019190604d565b909390049250505036603814608857366101f457346101f4575f5260205ff35b34106101f457600154600101600155600354806003026004013381556001015f35815560010160203590553360601b5f5260385f601437604c5fa0600101600355005b6003546002548082038060101160df575060105b5f5b8181146101835782810160030260040181604c02815460601b8152601401816001015481526020019060020154807fffffffffffffffffffffffffffffffff00000000000000000000000000000000168252906010019060401c908160381c81600701538160301c81600601538160281c81600501538160201c81600401538160181c81600301538160101c81600201538160081c81600101535360010160e1565b910180921461019557906002556101a0565b90505f6002555f6003555b5f54807fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff14156101cd57505f5b6001546002828201116101e25750505f6101e8565b01600290035b5f555f600155604c025ff35b5f5ffd",
//!   "v": "0x1b",
//!   "r": "0x539",
//!   "s": "0x5feeb084551e4e03a3581e269bc2ea2f8d0008",
//!   "hash": "0x8ded54be89448d78d4bc97782c0187b099e45380ab681742f9d3754e405c2572"
//! }
//! ```
//!
//! ```python
//! Sender: 0x8646861A7cF453dDD086874d622b0696dE5b9674
//! Address: 0x00000961Ef480Eb55e80D19ad83579A64c007002
//! ```
//!
//! ### Consensus layer
//!
//! [Full specification](https://github.com/ethereum/consensus-specs/blob/7bf43d1bc4fdb91059f0e6f4f7f0f3349b144950/specs/electra/beacon-chain.md)
//!
//! Sketch of spec:
//!
//! * New operation `ExecutionLayerWithdrawalRequest`
//! * Will show up in `ExecutionPayload` as an SSZ List bound by length `MAX_WITHDRAWAL_REQUESTS_PER_BLOCK`
//! * New function that has similar functionality to `process_voluntary_exit` but can fail validations (e.g. validator is already exited) without the block failing (similar to deposit coming from EL)
//! * This function is called in `process_operations` for each `ExecutionLayerWithdrawalRequest` found in the `ExecutionPayload`
//!
//! ## Rationale
//!
//! ### `validator_pubkey` field
//!
//! Multiple validators can utilize the same execution layer withdrawal credential, thus the `validator_pubkey` field is utilized to disambiguate which validator is being exited.
//!
//! Note, `validator_index` also disambiguates validators.
//! The problem is that smart contracts of some staking pools are not aware of the indices, because the index becomes known only after validator has been created on the beacon chain, while the pubkey is available in advance.
//!
//! ### Message queue
//!
//! The contract maintains an in-state queue of withdrawal request messages to be dequeued each block into the block and thus into the execution layer.
//!
//! The number of withdrawal requests that can be passed into the consensus layer are bound by `MAX_WITHDRAWAL_REQUESTS_PER_BLOCK` to bound the load both on the block size as well as on the consensus layer processing. `16` has been chosen for `MAX_WITHDRAWAL_REQUESTS_PER_BLOCK` to be in line with the bounds of similar operations on the beacon chain -- e.g. `VoluntaryExit` and `Deposit`.
//!
//! Although there is a maximum number of withdrawal requests that can passed to the consensus layer each block, the execution layer gas limit can provide for far more calls to the withdrawal request predeploy contract at each block. The queue then allows for these calls to successfully be made while still maintaining a system rate limit.
//!
//! The alternative design considered was to have calls to the contract fail after `MAX_WITHDRAWAL_REQUESTS_PER_BLOCK` successful calls were made within the context of a single block. This would eliminate the need for the message queue, but would come at the cost of a bad UX of contract call failures in times of high exiting. The complexity to mitigate this bad UX is relatively low and is currently favored.
//!
//! ### Rate limiting using a fee
//!
//! Transactions are naturally rate-limited in the execution layer via the gas limit, but an adversary willing to pay market-rate gas fees (and potentially utilize builder markets to pay for front-of-block transaction inclusion) can fill up the exit operation limits for relatively cheap, thus griefing honest validators that want to make a withdrawal request.
//!
//! There are two general approaches to combat this griefing -- (a) only allow validators to send such messages and with a limit per time period or (b) utilize an economic method to make such griefing increasingly costly.
//!
//! Method (a) (not used in this EIP) would require [EIP-4788](./eip-4788.md) (the `BEACON_ROOT` opcode) against which to prove withdrawal credentials in relation to validator pubkeys as well as a data-structure to track requests per-unit-time (e.g. 4 months) to ensure that a validator cannot grief the mechanism by submitting many requests. The downsides of this method are that it requires another cross-layer EIP and that it is of higher cross-layer complexity (e.g. care that might need to be taken in future upgrades if, for example, the shape of the merkle tree of `BEACON_ROOT` changes, then the contract and proof structure might need to be updated).
//!
//! Method (b) has been utilized in this EIP to eliminate additional EIP requirements and to reduce cross-layer complexity to allow for correctness of this EIP (now and in the future) to be easier to analyze. The [EIP-1559](./eip-1559.md)-style mechanism with a dynamically adjusting fee mechanism allows for users to pay `MIN_WITHDRAWAL_REQUEST_FEE` for withdrawal requests in the normal case (fewer than 2 per block on average), but scales the fee up exponentially in response to high usage (i.e. potential abuse).
//!
//! ### `TARGET_WITHDRAWAL_REQUESTS_PER_BLOCK` configuration value
//!
//! `TARGET_WITHDRAWAL_REQUESTS_PER_BLOCK` has been selected as `2` such that the growth of the partial withdrawal queue in the beacon state is negligible under extreme scenarios of the exit churn congestion.
//!
//! ### Fee update rule
//!
//! The fee update rule is intended to approximate the formula `fee = MIN_WITHDRAWAL_REQUEST_FEE * e**(excess / WITHDRAWAL_REQUEST_FEE_UPDATE_FRACTION)`, where `excess` is the total "extra" amount of withdrawal requests that the chain has processed relative to the "targeted" number (`TARGET_WITHDRAWAL_REQUESTS_PER_BLOCK` per block).
//!
//! Like EIP-1559, it’s a self-correcting formula: as the excess goes higher, the `fee` increases exponentially, reducing usage and eventually forcing the excess back down.
//!
//! The block-by-block behavior is roughly as follows. If block `N` processes `X` requests, then at the end of block `N` `excess` increases by `X - TARGET_WITHDRAWAL_REQUESTS_PER_BLOCK`, and so the `fee` in block `N+1` increases by a factor of `e**((X - TARGET_WITHDRAWAL_REQUESTS_PER_BLOCK) / WITHDRAWAL_REQUEST_FEE_UPDATE_FRACTION)`. Hence, it has a similar effect to the existing EIP-1559, but is more "stable" in the sense that it responds in the same way to the same total withdrawal requests regardless of how they are distributed over time.
//!
//! The parameter `WITHDRAWAL_REQUEST_FEE_UPDATE_FRACTION` controls the maximum downwards rate of change of the blob gas price. It is chosen to target a maximum downwards change rate of `e(TARGET_WITHDRAWAL_REQUESTS_PER_BLOCK / WITHDRAWAL_REQUEST_FEE_UPDATE_FRACTION) ≈ 1.125` per block.
//!
//! More detailed analysis of the fee mechanism is available [here](../assets/eip-7002/fee_analysis.md).
//!
//! ### Withdrawal requests inside of the block
//!
//! Withdrawal requests are placed into the actual body of the beacon block.
//!
//! There is a strong design requirement that the consensus layer and execution layer can execute independently of each other. This means, in this case, that the consensus layer cannot rely upon a synchronous call to the execution layer to get the required withdrawal requests for the current block. Instead, the requests must be embedded in the beacon block such that if the execution layer is offline, the consensus layer still has the requisite data to fully execute the consensus portion of the state transition function.
//!
//! ### Submitting requests via execution layer
//!
//! Verifying `secp256k1` signatures from the consensus layer via Engine API is one of the alternatives to the proposed requests mechanism which engineering complexity is much lower.
//! However, this approach would limit usage of withdrawal requests to a large extent by making it impossible for smart contracts owning validator withdrawal credentials to benefit from this functionality.
//!
//! ## Backwards Compatibility
//!
//! This EIP introduces backwards incompatible changes to the block structure and block validation rule set. But neither of these changes break anything related to current user activity and experience.
//!
//! ## Security Considerations
//!
//! ### Impact on existing custody relationships
//!
//! There might be existing custody relationships and/or products that rely upon the assumption that the withdrawal credentials *cannot* trigger a withdrawal request. We are currently confident that the additional withdrawal credentials feature does not impact the security of existing validators because:
//!
//! 1. The withdrawal credentials ultimately own the funds so allowing them to exit staking is natural with respect to ownership.
//! 2. We are currently not aware of any such custody relationships and/or products that do rely on the lack of this feature.
//!
//! In the event that existing validators/custodians rely on this, then the validators can be exited and restaked utilizing 0x01 withdrawal credentials pointing to a smart contract that simulates this behaviour.
//!
//! ### Fee Overpayment
//!
//! Calls to the system contract require a fee payment defined by the current contract state. Overpaid fees are not returned to the caller. It is not generally possible to compute the exact required fee amount ahead of time. When adding a withdrawal request from a contract, the contract can perform a read operation to check for the current fee and then pay exactly the required amount. Here is an example in Solidity:
//!
//! ```solidity
//! function addWithdrawal(bytes memory pubkey, uint64 amount, uint64 requestFeeLimit) private {
//!     assert(pubkey.length == 48);
//!
//!     // Read current fee from the contract.
//!     (bool readOK, bytes memory feeData) = WithdrawalsContract.staticcall('');
//!     if (!readOK) {
//!         revert('reading fee failed');
//!     }
//!     uint256 fee = uint256(bytes32(feeData));
//!
//!     // Check the fee is not too high.
//!     if (fee > requestFeeLimit) {
//!         revert('fee is too high');
//!     }
//!
//!     // Add the request.
//!     bytes memory callData = abi.encodePacked(pubkey, amount);
//!     (bool writeOK,) = WithdrawalsContract.call{value: fee}(callData);
//!     if (!writeOK) {
//!         revert('adding request failed');
//!     }
//! }
//! ```
//!
//! Note: the system contract uses the EVM `CALLER` operation (Solidity: `msg.sender`) as the target address for withdrawals, i.e. the address that calls the system contract must match the 0x01 withdrawal credential recorded in the beacon state.
//!
//! Note: the above code reverts if the fee is too high, the fee can change significantly between creation of a withdrawal request transaction and its inclusion into a block, thus, this check is very important to avoid overpayments.
//!
//! Using an EOA to request withdrawals will always result in overpayment of fees. There is no way for an EOA to use a wrapper contract to request a withdrawal. And even if a way existed, the gas cost of returning the overage would likely be higher than the overage itself. If requesting withdrawals to an EOA through the system contract is desired, we recommend that users perform transaction simulations to estimate a reasonable fee amount to send.
//!
//! ### System Call failure
//!
//! Although the likelihood of a failed system call to `WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS` is extremely low, the behavior in such cases is well-defined: the block is marked as invalid. However, if the failure results from processing a transaction within the block, the public mempool may still retain the transaction even after the block is invalidated. This can result in the offending transaction being included again, potentially causing one or more subsequent slots to go without valid blocks. To mitigate this, we recommend that the block producer implementation shuffle their transaction set to increase the chances of producing a valid block, without the offending transaction(s). The block producer implementation and/or the mempool should be aware of system call failure scenarios to enable this behavior.
//!
//! ### Empty Code failure
//!
//! This EIP should not have been activated if there is no code present at `WITHDRAWAL_REQUEST_PREDEPLOY_ADDRESS` (i.e., if the chain is not "ready"). Doing so would cause the first and all subsequent blocks after `FORK_BLOCK` to be marked invalid.
//!
//! If this situation occurs on a live chain, the following are two potential recovery strategies:
//!
//! * Deploy the contract code via a transaction and include it in a block. This block would become the first valid block, provided the system call to the contract does not fail. This works because the empty code validation occurs after block-transactions execution.
//! * Postpone the `FORK_BLOCK` activation point by updating the consensual fork timestamp or block number in the client implementation(s), then deploy the contract before the fork activates.
//!
//! Danny Ryan (@djrtwo), Mikhail Kalinin (@mkalinin), Ansgar Dietrichs (@adietrichs), Hsiao-Wei Wang (@hwwhww), lightclient (@lightclient), Felix Lange (@fjl), "EIP-7002: Execution layer triggerable withdrawals," Ethereum Improvement Proposals, no. 7002, May 2023. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-7002>.

use crate::eip::Eip;

/// EIP-7002: Execution layer triggerable withdrawals.
pub struct Eip7002;

impl Eip for Eip7002 {
    const NUMBER: u32 = 7002;
}
