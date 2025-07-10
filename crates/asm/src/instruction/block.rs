//! Block information.

use crate::{
    instruction::InstructionMeta,
    opcode::{Mnemonic, OpCode},
};

/// Get the hash of one of the 256 most recent complete blocks.
pub struct BlockHash;

impl InstructionMeta for BlockHash {
    fn opcode(&self) -> OpCode {
        OpCode::Known(Mnemonic::BLOCKHASH)
    }
}

/// Get the block’s beneficiary address.
pub struct CoinBase;

impl InstructionMeta for CoinBase {
    fn opcode(&self) -> OpCode {
        OpCode::Known(Mnemonic::COINBASE)
    }
}

/// Get the block’s timestamp.
pub struct Timestamp;

impl InstructionMeta for Timestamp {
    fn opcode(&self) -> OpCode {
        OpCode::Known(Mnemonic::TIMESTAMP)
    }
}

/// Get the block’s number.
pub struct Number;

impl InstructionMeta for Number {
    fn opcode(&self) -> OpCode {
        OpCode::Known(Mnemonic::NUMBER)
    }
}

/// Get the block’s difficulty.
pub struct PrevRandao;

impl InstructionMeta for PrevRandao {
    fn opcode(&self) -> OpCode {
        OpCode::Known(Mnemonic::PREVRANDAO)
    }
}

/// Get the block’s gas limit.
pub struct GasLimit;

impl InstructionMeta for GasLimit {
    fn opcode(&self) -> OpCode {
        OpCode::Known(Mnemonic::GASLIMIT)
    }
}

/// Get the chain ID.
pub struct ChainId;

impl InstructionMeta for ChainId {
    fn opcode(&self) -> OpCode {
        OpCode::Known(Mnemonic::CHAINID)
    }
}

/// Get balance of currently executing account.
pub struct SelfBalance;

impl InstructionMeta for SelfBalance {
    fn opcode(&self) -> OpCode {
        OpCode::Known(Mnemonic::SELFBALANCE)
    }
}

/// Get the base fee.
pub struct BaseFee;

impl InstructionMeta for BaseFee {
    fn opcode(&self) -> OpCode {
        OpCode::Known(Mnemonic::BASEFEE)
    }
}

/// Get versioned hashes.
pub struct BlobHash;

impl InstructionMeta for BlobHash {
    fn opcode(&self) -> OpCode {
        OpCode::Known(Mnemonic::BLOBHASH)
    }
}

/// Returns the value of the blob base-fee of the current block.
pub struct BlobBaseFee;

impl InstructionMeta for BlobBaseFee {
    fn opcode(&self) -> OpCode {
        OpCode::Known(Mnemonic::BLOBBASEFEE)
    }
}
