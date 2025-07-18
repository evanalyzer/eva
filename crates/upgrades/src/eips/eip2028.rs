//! EIP-2028: Transaction data gas cost reduction.
//! ## Simple Summary
//! We propose to reduce the gas cost of Calldata (`GTXDATANONZERO`) from its current value of 68 gas per byte to 16 gas per byte, to be backed by mathematical modeling and empirical estimates. The mathematical model is the one used in the works of Sompolinsky and Zohar [1] and Pass, Seeman and Shelat [2], which relates network security to network delay. We shall (1) evaluate the theoretical impact of lower Calldata gas cost on network delay using this model, (2) validate the model empirically, and (3) base the proposed gas cost on our findings.
//!
//! ## Motivation
//! There are a couple of main benefits to accepting this proposal and lowering gas cost of Calldata
//! On-Chain Scalability: Generally speaking, higher bandwidth of Calldata improves scalability, as more data can fit within a single block.
//! * Layer two scalability: Layer two scaling solutions can improve scalability by moving storage and computation off-chain, but often introduce data transmission instead.
//! 	- Proof systems such as STARKs and SNARKs use a single proof that attests to the computational integrity of a large computation, say, one that processes a large batch of transactions.
//! 	- Some solutions use fraud proofs which requires a transmission of merkle proofs.
//! 	- Moreover, one optional data availability solution to layer two is to place data on the main chain, via Calldata.
//! * Stateless clients: The same model will be used to determine the price of the state access for the stateless client regime, which will be proposed in the State Rent (from version 4). There, it is expected that the gas cost of state accessing operation will increase roughly proportional to the extra bandwidth required to transmit the “block proofs” as well as extra processing required to verify those block proofs.
//!
//! ## Specification
//! The gas per non-zero byte is reduced from 68 to 16. Gas cost of zero bytes is unchanged.
//!
//! ## Rationale
//! Roughly speaking, reducing the gas cost of Calldata leads to potentially larger blocks, which increases the network delay associated with data transmission over the network. This is only part of the full network delay, other factors are block processing time (and storage access, as part of it). Increasing network delay affects security by lowering the cost of attacking the network, because at any given point in time fewer nodes are updated on the latest state of the blockchain.
//!
//! Yonatan Sompolinsky and Aviv Zohar suggested in [1] an elegant model to relate network delay to network security, and this model is also used in the work of Rafael Pass, Lior Seeman and Abhi Shelat [2]. We briefly explain this model below, because we shall study it theoretically and validate it by empirical measurements to reach the suggested lower gas cost for Calldata.
//!
//! The model uses the following natural parameters:
//! * _lambda_  denotes the block creation rate [1/s]: We treat the process of finding a PoW
//! solution as a poisson process with rate _lambda_.
//! * _beta_ - chain growth rate [1/s]: the rate at which new blocks are added to
//! the heaviest chain.
//! * _D_ - block delay [s]: The time that elapses between the mining of a new block and its acceptance by all the miners (all miners switched to mining on top of that block).
//!
//! ### _Beta_ Lower Bound
//! Notice that _lambda_ => _beta_, because not all blocks that are found will enter the main chain (as is the case with uncles). In [1] it was shown that for a blockchain using the longest chain rule, one may bound _beta_ from below by _lambda_/ (1+ D * _lambda_). This lower bound holds in the extremal case where the topology of the network is a clique in which the delay between each pair of nodes is D, the maximal possible delay. Recording both the lower and upper bounds on _beta_ we get
//! ```python
//! 	_lambda_ >= _beta_ >= _lambda_ / (1 + D * _lambda_)               (*)
//! ```
//! Notice, as a sanity check, that when there is no delay (D=0) then _beta_ equals _lambda_, as expected.
//!
//! ### Security of the network
//! An attacker attempting to reorganize the main chain needs to generate blocks at a rate that is greater than _beta_.
//! Fixing the difficulty level of the PoW puzzle, the total hash rate in the system is correlated to _lambda_. Thus, _beta_ / _lambda_ is defined as the *efficiency* of the system, as it measures the fraction of total hash power that is used to generate the main chain of the network.
//!
//! Rearranging (*) gives the following lower bound on efficiency in terms of delay:
//! ```python
//! 	_beta_ / _lambda_ >= 1 / (1 + D * _lambda_)                 (**)
//! ```
//! ### The _delay_ parameter D
//! The network delay depends on the location of the mining node within the network and on the current network topology (which changes dynamically), and consequently is somewhat difficult to measure directly.
//! Previously, Christian Decker and Roger Wattenhofer [3] showed that propagation time scales with blocksize,  and Vitalik Buterin showed that uncle rate, which is tightly related to efficiency (**) measure, also scales with block size [4].
//!
//! However, the delay function can be decomposed into two parts D = *D_t* + *`D_p`*, where _D_t_ is the delay caused by the transmission of the block and _`D_p`_ is the delay caused by the processing of the block by the node. Our model and tests will examine the effect of Calldata on each of _`D_t`_ and _`D_p`_, postulating that their effect is different. This may be particularly relevant for Layer 2 Scalability and for Stateless Clients (Rationales 2, 3 above) because most of the Calldata associated with these goals are Merkle authentication paths that have a large _D_t_ component but relatively small _`D_p`_ values.
//!
//! ## Test Cases
//! To suggest the gas cost of calldata we shall conduct two types of tests:
//! 1. Network tests, conducted on the Ethereum mainnet, used to estimate the effect on increasing block size on _D_p_ and _D_t_, on the overall network delay D and the efficiency ratio (**), as well as delays between different mining pools. Those tests will include regression tests on existing data, and stress tests to introduce extreme scenarios.
//! 2. Local tests, conducted on a single node and measuring the processing time as a function of Calldata amount and general computation limits.
//!
//! Alexey Akhunov (`@AlexeyAkhunov`), Eli Ben Sasson <eli@starkware.co>, Tom Brand <tom@starkware.co>, Louis Guthmann <louis@starkware.co>, Avihu Levy <avihu@starkware.co>, "EIP-2028: Transaction data gas cost reduction," Ethereum Improvement Proposals, no. 2028, May 2019. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-2028>.

use crate::eip::Eip;

/// EIP-2028: Transaction data gas cost reduction.
pub struct Eip2028;

impl Eip for Eip2028 {
    const NUMBER: u32 = 2028;
}
