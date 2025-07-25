//! EIP-152: Add BLAKE2 compression function `F` precompile.
//! ## Simple Summary
//!
//! This EIP will enable the `BLAKE2b` hash function and other higher-round 64-bit BLAKE2 variants to run cheaply on the EVM, allowing easier interoperability between Ethereum and Zcash as well as other Equihash-based `PoW` coins.
//!
//! ## Abstract
//!
//! This EIP introduces a new precompiled contract which implements the compression function `F` used in the BLAKE2 cryptographic hashing algorithm, for the purpose of allowing interoperability between the EVM and Zcash, as well as introducing more flexible cryptographic hash primitives to the EVM.
//!
//! ## Motivation
//!
//! Besides being a useful cryptographic hash function and SHA3 finalist, BLAKE2 allows for efficient verification of the Equihash `PoW` used in Zcash, making a BTC Relay - style SPV client possible on Ethereum. A single verification of an Equihash `PoW` verification requires 512 iterations of the hash function, making verification of Zcash block headers prohibitively expensive if a Solidity implementation of BLAKE2 is used.
//!
//! `BLAKE2b`, the common 64-bit BLAKE2 variant, is highly optimized and faster than MD5 on modern processors.
//!
//! Interoperability with Zcash could enable contracts like trustless atomic swaps between the chains, which could provide a much needed aspect of privacy to the very public Ethereum blockchain.
//!
//! ## Specification
//!
//! We propose adding a precompiled contract at address `0x09` wrapping the [BLAKE2 `F` compression function](https://tools.ietf.org/html/rfc7693#section-3.2).
//!
//! The precompile requires 6 inputs tightly encoded, taking exactly 213 bytes, as explained below. The encoded inputs are corresponding to the ones specified in the [BLAKE2 RFC Section 3.2](https://tools.ietf.org/html/rfc7693#section-3.2):
//!
//! - `rounds` - the number of rounds - 32-bit unsigned big-endian word
//! - `h` - the state vector - 8 unsigned 64-bit little-endian words
//! - `m` - the message block vector - 16 unsigned 64-bit little-endian words
//! - `t_0, t_1` - offset counters - 2 unsigned 64-bit little-endian words
//! - `f` - the final block indicator flag - 8-bit word
//!
//! ```python
//! [4 bytes for rounds][64 bytes for h][128 bytes for m][8 bytes for t_0][8 bytes for t_1][1 byte for f]
//! ```
//!
//! The boolean `f` parameter is considered as `true` if set to `1`.
//! The boolean `f` parameter is considered as `false` if set to `0`.
//! All other values yield an invalid encoding of `f` error.
//!
//! The precompile should compute the `F` function as [specified in the RFC](https://tools.ietf.org/html/rfc7693#section-3.2) and return the updated state vector `h` with unchanged encoding (little-endian).
//!
//! ### Example Usage in Solidity
//!
//! The precompile can be wrapped easily in Solidity to provide a more development-friendly interface to `F`.
//!
//! ```solidity
//! function F(uint32 rounds, bytes32[2] memory h, bytes32[4] memory m, bytes8[2] memory t, bool f) public view returns (bytes32[2] memory) {
//!   bytes32[2] memory output;
//!
//!   bytes memory args = abi.encodePacked(rounds, h[0], h[1], m[0], m[1], m[2], m[3], t[0], t[1], f);
//!
//!   assembly {
//!     if iszero(staticcall(not(0), 0x09, add(args, 32), 0xd5, output, 0x40)) {
//!       revert(0, 0)
//!     }
//!   }
//!
//!   return output;
//! }
//!
//! function callF() public view returns (bytes32[2] memory) {
//!   uint32 rounds = 12;
//!
//!   bytes32[2] memory h;
//!   h[0] = hex"48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5";
//!   h[1] = hex"d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b";
//!
//!   bytes32[4] memory m;
//!   m[0] = hex"6162630000000000000000000000000000000000000000000000000000000000";
//!   m[1] = hex"0000000000000000000000000000000000000000000000000000000000000000";
//!   m[2] = hex"0000000000000000000000000000000000000000000000000000000000000000";
//!   m[3] = hex"0000000000000000000000000000000000000000000000000000000000000000";
//!
//!   bytes8[2] memory t;
//!   t[0] = hex"0300000000000000";
//!   t[1] = hex"0000000000000000";
//!
//!   bool f = true;
//!
//!   // Expected output:
//!   // ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d1
//!   // 7d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923
//!   return F(rounds, h, m, t, f);
//! }
//! ```
//!
//! ### Gas costs and benchmarks
//!
//! Each operation will cost `GFROUND * rounds` gas, where `GFROUND = 1`. Detailed benchmarks are presented in the benchmarks appendix section.
//!
//! ## Rationale
//!
//! BLAKE2 is an excellent candidate for precompilation. BLAKE2 is heavily optimized for modern 64-bit CPUs, specifically utilizing 24 and 63-bit rotations to allow parallelism through SIMD instructions and little-endian arithmetic. These characteristics provide exceptional speed on native CPUs: 3.08 cycles per byte, or 1 gibibyte per second on an Intel i5.
//!
//! In contrast, the big-endian 32 byte semantics of the EVM are not conducive to efficient implementation of BLAKE2, and thus the gas cost associated with computing the hash on the EVM is disproportionate to the true cost of computing the function natively.
//!
//! An obvious implementation would be a direct `BLAKE2b` hash function precompile. At first glance, a `BLAKE2b` precompile satisfies most hashing and interoperability requirements on the EVM. Once we started digging in, however, it became clear that any `BLAKE2b` implementation would need specific features and internal modifications based on different projects' requirements and libraries.
//!
//! A [thread with the Zcash team](https://github.com/ethereum/EIPs/issues/152#issuecomment-499240310) makes the issue clear.
//!
//! > The minimal thing that is necessary for a working ZEC-ETH relay is an implementation of BLAKE2b Compression F in a precompile.
//!
//! > A BLAKE2b Compression Function F precompile would also suffice for the Filecoin and Handshake interop goals.
//!
//! > A full BLAKE2b precompile would suffice for a ZEC-ETH relay, provided that the implementation provided the parts of the BLAKE2 API that we need (personalization, maybe something else—I'm not sure).
//!
//! > I'm not 100% certain if a full BLAKE2b precompile would also suffice for the Filecoin and Handshake goals. It almost certainly could, provided that it supports all the API that they need.
//!
//! > BLAKE2s — whether the Compression Function F or the full hash — is only a nice-to-have for the purposes of a ZEC-ETH relay.
//!
//! From this and other conversations with teams in the space, we believe we should focus first on the `F` precompile as a strictly necessary piece for interoperability projects. A `BLAKE2b` precompile is a nice-to-have, and we support any efforts to add one-- but it's unclear whether complete requirements and a flexible API can be found in time for Istanbul.
//!
//! Implementation of only the core F compression function also allows substantial flexibility and extensibility while keeping changes at the protocol level to a minimum. This will allow functions like tree hashing, incremental hashing, and keyed, salted, and personalized hashing as well as variable length digests, none of which are currently available on the EVM.
//!
//! ## Backwards Compatibility
//!
//! There is very little risk of breaking backwards-compatibility with this EIP, the sole issue being if someone were to build a contract relying on the address at `0x09` being empty. The likelihood of this is low, and should specific instances arise, the address could be chosen to be any arbitrary value with negligible risk of collision.
//!
//! ## Test Cases
//!
//! #### Test vector 0
//! * input: (empty)
//! * output: error "input length for BLAKE2 F precompile should be exactly 213 bytes"
//!
//! #### Test vector 1
//! * input:
//!   `00000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001`
//! * output: error "input length for BLAKE2 F precompile should be exactly 213 bytes"
//!
//! #### Test vector 2
//! * input:
//!   `000000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001`
//! * output: error "input length for BLAKE2 F precompile should be exactly 213 bytes"
//!
//! #### Test vector 3
//! * input:
//!   `0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000002`
//! * output: error "incorrect final block indicator flag"
//!
//! #### Test vector 4
//! * input:
//!   `0000000048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001`
//! * output:
//!   `08c9bcf367e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d282e6ad7f520e511f6c3e2b8c68059b9442be0454267ce079217e1319cde05b`
//!
//! #### Test vector 5
//! * input:
//!   `0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001`
//! * output: `ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923`
//!
//! #### Test vector 6
//! * input:
//!   `0000000c48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000`
//! * output:
//!   `75ab69d3190a562c51aef8d88f1c2775876944407270c42c9844252c26d2875298743e7f6d5ea2f2d3e8d226039cd31b4e426ac4f2d3d666a610c2116fde4735`
//!
//! #### Test vector 7
//! * input:
//!   `0000000148c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001`
//! * output:
//!   `b63a380cb2897d521994a85234ee2c181b5f844d2c624c002677e9703449d2fba551b3a8333bcdf5f2f7e08993d53923de3d64fcc68c034e717b9293fed7a421`
//!
//! #### Test vector 8
//! * input:
//!   `ffffffff48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001`
//! * output:
//!   `fc59093aafa9ab43daae0e914c57635c5402d8e3d2130eb9b3cc181de7f0ecf9b22bf99a7815ce16419e200e01846e6b5df8cc7703041bbceb571de6631d2615`
//!
//! ## Implementation
//!
//! An initial implementation of the `F` function in Go, adapted from the standard library, can be found in our [Golang BLAKE2 library fork](https://github.com/keep-network/blake2-f). There's also an implementation of the precompile in our fork of [go-ethereum](https://github.com/keep-network/go-ethereum/pull/4).
//!
//! ## References
//!
//! For reference, further discussion on this EIP also occurred in the following PRs and issues
//!
//!  * [Original Issue](https://github.com/ethereum/EIPs/issues/152)
//!  * [Ethereum Magicians](https://ethereum-magicians.org/t/blake2b-f-precompile/3157)
//!  * [PR 2129](https://github.com/ethereum/EIPs/pull/2129)
//!
//! ## Appendix - benchmarks
//!
//! Assuming ecRecover precompile is perfectly priced, we executed a set of benchmarks comparing Blake2b F compression function precompile with ecRecover precompile. For benchmarks, we used 3.1 GHz Intel Core i7 64-bit machine.
//!
//! ```sh
//! $ sysctl -n machdep.cpu.brand_string
//! Intel(R) Core(TM) i7-7920HQ CPU @ 3.10GHz
//! ```
//!
//! ### 12 rounds
//!
//! An average gas price of F precompile call with 12 rounds compared to ecRecover should have been `6.74153` and it gives `0.5618` gas per round.
//!
//! ```python
//! Name                                         Gascost         Time (ns)     MGas/S    Gasprice for 10MGas/S    Gasprice for ECDSA eq
//! -----------------------------------------  ---------  ----------------  ---------  -----------------------  -----------------------
//! PrecompiledEcrecover/                           3000  152636              19.6546                 1526.36               3000
//! PrecompiledBlake2F/testVectors2bX_0               12     338              35.503                     3.38                  6.64326
//! PrecompiledBlake2F/testVectors2bX_3               12     336              35.7143                    3.36                  6.60395
//! PrecompiledBlake2F/testVectors2bX_70              12     362              33.1492                    3.62                  7.11497
//! PrecompiledBlake2F/testVectors2bX_140             12     339              35.3982                    3.39                  6.66291
//! PrecompiledBlake2F/testVectors2bX_230             12     339              35.3982                    3.39                  6.66291
//! PrecompiledBlake2F/testVectors2bX_300             12     343              34.9854                    3.43                  6.74153
//! PrecompiledBlake2F/testVectors2bX_370             12     336              35.7143                    3.36                  6.60395
//! PrecompiledBlake2F/testVectors2bX_440             12     337              35.6083                    3.37                  6.6236
//! PrecompiledBlake2F/testVectors2bX_510             12     345              34.7826                    3.45                  6.78084
//! PrecompiledBlake2F/testVectors2bX_580             12     355              33.8028                    3.55                  6.97738
//! ```
//!
//! Columns
//!
//! * `MGas/S` - Shows what `MGas` per second was measured on that machine at that time
//! * `Gasprice for 10MGas/S` shows what the gasprice should have been, in order to reach 10 MGas/second
//! * `Gasprice for ECDSA eq` shows what the gasprice should have been, in order to have the same cost/cycle as ecRecover
//!
//! ### 1200 rounds
//!
//! An average gas price of F precompile call with 1200 rounds compared to ecRecover should have been `436.1288` and it gives `0.3634` gas per round.
//!
//! ```python
//! Name                                         Gascost         Time (ns)     MGas/S    Gasprice for 10MGas/S    Gasprice for ECDSA eq
//! -----------------------------------------  ---------  ----------------  ---------  -----------------------  -----------------------
//! PrecompiledEcrecover/                           3000  156152              19.212                  1561.52               3000
//! PrecompiledBlake2F/testVectors2bX_0             1200   22642              52.9989                  226.42                434.999
//! PrecompiledBlake2F/testVectors2bX_3             1200   22885              52.4361                  228.85                439.668
//! PrecompiledBlake2F/testVectors2bX_70            1200   22737              52.7774                  227.37                436.824
//! PrecompiledBlake2F/testVectors2bX_140           1200   22602              53.0926                  226.02                434.231
//! PrecompiledBlake2F/testVectors2bX_230           1200   22501              53.331                   225.01                432.29
//! PrecompiledBlake2F/testVectors2bX_300           1200   22435              53.4879                  224.35                431.022
//! PrecompiledBlake2F/testVectors2bX_370           1200   22901              52.3995                  229.01                439.975
//! PrecompiledBlake2F/testVectors2bX_440           1200   23134              51.8717                  231.34                444.452
//! PrecompiledBlake2F/testVectors2bX_510           1200   22608              53.0786                  226.08                434.346
//! PrecompiledBlake2F/testVectors2bX_580           1200   22563              53.1844                  225.63                433.481
//! ```
//!
//! ### 1 round
//!
//! An average gas price of F precompile call with 1 round compared to ecRecover should have been `2.431701`. However, in this scenario the call cost would totally overshadow the dynamic cost anyway.
//!
//! ```python
//! Name                                         Gascost         Time (ns)      MGas/S    Gasprice for 10MGas/S    Gasprice for ECDSA eq
//! -----------------------------------------  ---------  ----------------  ----------  -----------------------  -----------------------
//! PrecompiledEcrecover/                           3000  157544              19.0423                  1575.44               3000
//! PrecompiledBlake2F/testVectors2bX_0                1     126               7.93651                    1.26                  2.39933
//! PrecompiledBlake2F/testVectors2bX_3                1     127               7.87402                    1.27                  2.41837
//! PrecompiledBlake2F/testVectors2bX_70               1     128               7.8125                     1.28                  2.43741
//! PrecompiledBlake2F/testVectors2bX_140              1     125               8                          1.25                  2.38029
//! PrecompiledBlake2F/testVectors2bX_230              1     128               7.8125                     1.28                  2.43741
//! PrecompiledBlake2F/testVectors2bX_300              1     127               7.87402                    1.27                  2.41837
//! PrecompiledBlake2F/testVectors2bX_370              1     131               7.63359                    1.31                  2.49454
//! PrecompiledBlake2F/testVectors2bX_440              1     129               7.75194                    1.29                  2.45646
//! PrecompiledBlake2F/testVectors2bX_510              1     125               8                          1.25                  2.38029
//! PrecompiledBlake2F/testVectors2bX_580              1     131               7.63359                    1.31                  2.49454
//! ```
//!
//! Tjaden Hess <tah83@cornell.edu>, Matt Luongo (@mhluongo), Piotr Dyraga (@pdyraga), James Hancock (@`MadeOfTin`), "EIP-152: Add BLAKE2 compression function `F` precompile," Ethereum Improvement Proposals, no. 152, October 2016. [Online serial]. Available: <https://eips.ethereum.org/EIPS/eip-152>.

use crate::eip::Eip;

/// EIP-152: Add BLAKE2 compression function `F` precompile.
pub struct Eip152;

impl Eip for Eip152 {
    const NUMBER: u32 = 152;
}
