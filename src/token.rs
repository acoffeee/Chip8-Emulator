enum Registers = {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8
    V9,
    Va,
    Vb,
    Vc,
    Vd,
    Ve,
    Vf
}
#[derive(Debug, PartialEq)]
enum tokens = {
    //instructions
    CLS,
    RET,
    JP,
    CALL,
    SE,
    SNE,
    LD,
    ADD,
    OR,
    AND,
    XOR,
    SUB,
    SHR,
    SUBN,
    SHL,
    RND,
    DRW,
    SKP,
    SKNP,
    DELAY,
    SOUND,
    HEX,
    BCD,
    STOR,
    RSTR,
    STOP,
    WAIT,
    NOPE,
    SP,
    SPG,
    JPB,
    JPF,
    OUT,
    HLT,
    HLTREAD,
    READ,
    //types ig like Vx or DT
    REGISTER(0x0),
    DT, //delay timer
    ADDRESS,
    INDEX,
    BYTE,
    ST, // Sount timer
    COMMA,
    SEMICOLON,
    ALIAS

}
impl tokens {
    pub fn from_String(s: &str) -> Option<Self> {
        let s = s.to_lowercase();

        Some(match s.as_str() {
            "cls" => tokens::CLS,
            "ret" => tokens::RET,
            "jp" => tokens::JP,
            "call" => tokens::CALL,
            "se" => tokens::SE,
            "sne" => tokens::SNE,
            "ld" => tokens::LD,
            "add" => tokens::ADD,
            "or" => tokens::OR,
            "and" => tokens::AND,
            "xor" => tokens::XOR,
            "sub" => tokens::SUB,
            "shr" => tokens::SHR,
            "subn" => tokens::SUBN,
            "shl" => tokens::SHL,
            "rnd" => tokens::RND,
            "drw" => tokens::DRW,
            "skp" => tokens::SKP,
            "sknp" => tokens::SKNP,
            "delay" => tokens::DELAY,
            "sound" => tokens::SOUND,
            "hex" => tokens::HEX,
            "bcd" => tokens::BCD,
            "stor" => tokens::STOR,
            "rstr" => tokens::RSTR,
            "stop" => tokens::STOP,
            "wait" => tokens::WAIT,
            "nope" => tokens::NOPE,
            "sp" => tokens::SP,
            "spg" => tokens::SPG,
            "jpb" => tokens::JPB,
            "jpf" => tokens::JPF,
            "out" => tokens::OUT,
            "hlt" => tokens::HLT,
            "hltread" => tokens::HLTREAD,
            "read" => tokens::READ,

            "dt" => tokens::DT,
            "address" => tokens::ADDRESS,
            "index" => tokens::INDEX,
            "byte" => tokens::BYTE,
            "st" => tokens::ST,
            "," => tokens::COMMA,
            "=" => tokens::ASSIGNMENT,
            ";" => tokens::SEMICOLON,
            "alias" => tokens::ALIAS,
            s if matches!(
                s,
                "v0" | "v1" | "v2" | "v3" |
                "v4" | "v5" | "v6" | "v7" |
                "v8" | "v9" | "va" | "vb" |
                "vc" | "vd" | "ve" | "vf"
            ) => tokens::REGISTER,

            _ => return None,
        })
    }
}

impl PartialEq<&Str> for tokens {
    fn eq(&self, other: &Str) -> bool {
        let other = other.to_lowercase();
        match self {
            tokens::CLS => other == "cls",
            tokens::RET => other == "ret",
            tokens::JP => other == "jp",
            tokens::CALL => other == "call",
            tokens::SE => other == "se",
            tokens::SNE => other == "sne",
            tokens::LD => other == "ld",
            tokens::ADD => other == "add",
            tokens::OR => other == "or",
            tokens::AND => other == "and",
            tokens::XOR => other == "xor",
            tokens::SUB => other == "sub",
            tokens::SHR => other == "shr",
            tokens::SUBN => other == "subn",
            tokens::SHL => other == "shl",
            tokens::RND => other == "rnd",
            tokens::DRW => other == "drw",
            tokens::SKP => other == "skp",
            tokens::SKNP => other == "sknp",
            tokens::DELAY => other == "delay",
            tokens::SOUND => other == "sound",
            tokens::HEX => other == "hex",
            tokens::BCD => other == "bcd",
            tokens::STOR => other == "stor",
            tokens::RSTR => other == "rstr",
            tokens::STOP => other == "stop",
            tokens::WAIT => other == "wait",
            tokens::NOPE => other == "nope",
            tokens::SP => other == "sp",
            tokens::SPG => other == "spg",
            tokens::JPB => other == "jpb",
            tokens::JPF => other == "jpf",
            tokens::OUT => other == "out",
            tokens::HLT => other == "hlt",
            tokens::HLTREAD => other == "hltread",
            tokens::READ => other == "read",

            tokens::REGISTER => {
                matches!(
                    other.as_str(),
                    "v0" | "v1" | "v2" | "v3" |
                    "v4" | "v5" | "v6" | "v7" |
                    "v8" | "v9" | "va" | "vb" |
                    "vc" | "vd" | "ve" | "vf"
                )
            }

            tokens::DT => other == "dt",
            tokens::ADDRESS => other == "address",
            tokens::INDEX => other == "index",
            tokens::BYTE => other == "byte",
            tokens::ST => other == "st",
            tokens::COMMA => other == ",",
            tokens::ASSIGNMENT => other == "=",
            tokens::SEMICOLON => other == ";",
            tokens::ALIAS => other == "alias"
        }
    }
}