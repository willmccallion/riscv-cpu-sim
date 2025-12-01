#[derive(Clone, Copy, Debug, Default)]
pub struct Decoded {
    pub raw: u32,
    pub opcode: u32,
    pub rd: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub funct3: u32,
    pub funct7: u32,
    pub imm: i64,
}
