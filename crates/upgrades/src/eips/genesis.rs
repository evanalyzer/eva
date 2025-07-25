//! Genesis state of Ethereum.

use asm::instruction::{
    Add, AddMod, Address, And, Balance, BlockHash, Byte, Call, CallCode, CallDataCopy,
    CallDataLoad, CallDataSize, CallValue, Caller, CodeCopy, CodeSize, CoinBase, Create, Div, Dup,
    Eq, Exp, ExtCodeCopy, ExtCodeSize, Gas, GasLimit, GasPrice, Gt, Invalid, IsZero, Jump,
    JumpDest, JumpI, Keccak256, Log, Lt, MLoad, MSize, MStore, MStore8, Mod, Mul, MulMod, Not,
    Number, Or, Origin, Pc, Pop, PrevRandao, Push, Return, SDiv, SGt, SLoad, SLt, SMod, SStore,
    SelfDestruct, SignExtend, Stop, Sub, Swap, Timestamp, Unknown, Xor,
};

use crate::eip::{Eip, macros::introduces_instructions};

/// Genesis state of Ethereum.
pub struct Genesis;

impl Eip for Genesis {
    const NUMBER: u32 = 0;
}

introduces_instructions!(
    Genesis,
    Stop,
    Add,
    Mul,
    Sub,
    Div,
    SDiv,
    Mod,
    SMod,
    AddMod,
    MulMod,
    Exp,
    SignExtend,
    Lt,
    Gt,
    SLt,
    SGt,
    Eq,
    IsZero,
    And,
    Or,
    Xor,
    Not,
    Byte,
    Keccak256,
    Address,
    Balance,
    Origin,
    Caller,
    CallValue,
    CallDataLoad,
    CallDataSize,
    CallDataCopy,
    CodeSize,
    CodeCopy,
    GasPrice,
    ExtCodeSize,
    ExtCodeCopy,
    BlockHash,
    CoinBase,
    Timestamp,
    Number,
    PrevRandao,
    GasLimit,
    Pop,
    MLoad,
    MStore,
    MStore8,
    SLoad,
    SStore,
    Jump,
    JumpI,
    Pc,
    MSize,
    Gas,
    JumpDest,
    Push<1>,
    Push<2>,
    Push<3>,
    Push<4>,
    Push<5>,
    Push<6>,
    Push<7>,
    Push<8>,
    Push<9>,
    Push<10>,
    Push<11>,
    Push<12>,
    Push<13>,
    Push<14>,
    Push<15>,
    Push<16>,
    Push<17>,
    Push<18>,
    Push<19>,
    Push<20>,
    Push<21>,
    Push<22>,
    Push<23>,
    Push<24>,
    Push<25>,
    Push<26>,
    Push<27>,
    Push<28>,
    Push<29>,
    Push<30>,
    Push<31>,
    Push<32>,
    Dup<1>,
    Dup<2>,
    Dup<3>,
    Dup<4>,
    Dup<5>,
    Dup<6>,
    Dup<7>,
    Dup<8>,
    Dup<9>,
    Dup<10>,
    Dup<11>,
    Dup<12>,
    Dup<13>,
    Dup<14>,
    Dup<15>,
    Dup<16>,
    Swap<1>,
    Swap<2>,
    Swap<3>,
    Swap<4>,
    Swap<5>,
    Swap<6>,
    Swap<7>,
    Swap<8>,
    Swap<9>,
    Swap<10>,
    Swap<11>,
    Swap<12>,
    Swap<13>,
    Swap<14>,
    Swap<15>,
    Swap<16>,
    Log<0>,
    Log<1>,
    Log<2>,
    Log<3>,
    Log<4>,
    Create,
    Call,
    CallCode,
    Return,
    Invalid,
    SelfDestruct,
    Unknown
);
