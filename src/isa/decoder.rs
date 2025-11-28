use super::instruction::Decoded;
use super::opcodes;

#[inline]
pub fn decode(word: u32) -> Decoded {
    let opcode = word & 0x7f;
    let rd = ((word >> 7) & 0x1f) as usize;
    let funct3 = (word >> 12) & 0x7;
    let rs1 = ((word >> 15) & 0x1f) as usize;
    let rs2 = ((word >> 20) & 0x1f) as usize;
    let funct7 = (word >> 25) & 0x7f;

    let imm = match opcode {
        opcodes::OP_IMM | opcodes::OP_LOAD | opcodes::OP_JALR | opcodes::OP_IMM_32 => {
            ((word as i32) >> 20) as i64
        }
        opcodes::OP_STORE => {
            let imm = (((word >> 25) & 0x7f) << 5) | ((word >> 7) & 0x1f);
            ((imm as i32) << 20 >> 20) as i64
        }
        opcodes::OP_BRANCH => {
            let imm = (((word >> 31) & 1) << 12)
                | (((word >> 7) & 1) << 11)
                | (((word >> 25) & 0x3f) << 5)
                | (((word >> 8) & 0xf) << 1);
            (((imm as i32) << 19) >> 19) as i64
        }
        opcodes::OP_LUI | opcodes::OP_AUIPC => ((word & 0xfffff000) as i32) as i64,
        opcodes::OP_JAL => {
            let imm = (((word >> 31) & 1) << 20)
                | (((word >> 12) & 0xff) << 12)
                | (((word >> 20) & 1) << 11)
                | (((word >> 21) & 0x3ff) << 1);
            (((imm as i32) << 11) >> 11) as i64
        }
        _ => 0,
    };

    Decoded {
        raw: word,
        opcode,
        rd,
        rs1,
        rs2,
        funct3,
        funct7,
        imm,
    }
}
