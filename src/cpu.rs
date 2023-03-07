use crate::memory::Memory;

pub struct Cpu {
    pub registers: Registers,
}

#[derive(Clone, Copy)]
pub struct Registers {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,
    ime: u8,
}
//read_register_8 from register
impl Registers {
    pub fn read_8(&self, register: char) -> u8 {
        //switch case for each 8 bit register a, b, c, d, e, h, l, f,
        let tmp = *self;
        (match register {
            'a' => tmp.af >> 8,
            'b' => tmp.bc >> 8,
            'c' => tmp.bc & 0xFF,
            'd' => tmp.de >> 8,
            'e' => tmp.de & 0xFF,
            'h' => tmp.hl >> 8,
            'l' => tmp.hl & 0xFF,
            'f' => tmp.af & 0xFF,
            'i' => tmp.ime as u16,
            _ => 0,
        }) as u8
    }

    pub fn read_16(&self, register: &str) -> u16 {
        //switch case for each 16 bit register af, bc, de, hl, sp, pc
        let tmp = *self;
        match register {
            "af" => tmp.af,
            "bc" => tmp.bc,
            "de" => tmp.de,
            "hl" => tmp.hl,
            "sp" => tmp.sp,
            "pc" => tmp.pc,
            _ => 0,
        }
    }

    pub fn write_8(&mut self, register: char, value: u8) {
        //switch case for each 8 bit register a, b, c, d, e, h, l, f,
        let tmp = *self;
        match register {
            'a' => self.af = (tmp.af & 0xFF) | ((value as u16) << 8),
            'b' => self.bc = (tmp.bc & 0xFF) | ((value as u16) << 8),
            'c' => self.bc = (tmp.bc & 0xFF00) | value as u16,
            'd' => self.de = (tmp.de & 0xFF) | ((value as u16) << 8),
            'e' => self.de = (tmp.de & 0xFF00) | value as u16,
            'h' => self.hl = (tmp.hl & 0xFF) | ((value as u16) << 8),
            'l' => self.hl = (tmp.hl & 0xFF00) | value as u16,
            'f' => self.af = (tmp.af & 0xFF00) | value as u16,
            'i' => self.ime = value,
            _ => (),
        }
    }

    pub(crate) fn write_16(&mut self, register: &str, value: u16) {
        //switch case for each 16 bit register _af, bc, de, hl, sp, pc

        match register {
            "af" => self.af = value,
            "bc" => self.bc = value,
            "de" => self.de = value,
            "hl" => self.hl = value,
            "sp" => self.sp = value,
            "pc" => self.pc = value,
            _ => (),
        }
    }
}

// array for opcodes duration
const OPCODE_DURATION: [u8; 256] = [
    4, 12, 8, 8, 4, 4, 8, 4, 20, 8, 8, 8, 4, 4, 8, 4, 4, 12, 8, 8, 4, 4, 8, 4, 12, 8, 8, 8, 4, 4,
    8, 4, 12, 12, 8, 8, 4, 4, 8, 4, 12, 8, 8, 8, 4, 4, 8, 4, 12, 12, 8, 8, 12, 12, 12, 4, 12, 8, 8,
    8, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4,
    4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 8, 8, 8, 8, 8, 8, 4, 8, 4, 4, 4,
    4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4,
    4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4,
    4, 4, 4, 8, 4, 20, 12, 16, 16, 24, 16, 8, 16, 20, 16, 16, 4, 24, 24, 8, 16, 20, 12, 16, 0, 24,
    16, 8, 16, 20, 16, 16, 0, 24, 0, 8, 16, 12, 12, 8, 0, 0, 16, 8, 16, 16, 4, 16, 0, 0, 0, 8, 16,
    12, 12, 8, 4, 0, 16, 8, 16, 12, 8, 16, 4, 0, 0, 8, 16,
];
const OPCODE_DURATION_CB: [u8; 256] = [
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8,
    16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8,
    8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8,
    8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8,
    16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8,
    8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8,
    8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
];
const OPCODE_LENGTHS: [u8; 256] = [
    1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1, 1, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, 2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 1, 3, 3, 2, 1, 1, 1, 3, 0, 3, 1, 2, 1, 1, 1, 3, 0, 3, 0, 2, 1,
    2, 1, 1, 0, 0, 1, 2, 1, 2, 1, 3, 0, 0, 0, 2, 1, 2, 1, 1, 1, 0, 1, 2, 1, 2, 1, 3, 1, 0, 0, 2, 1,
];
const OPCODE_LENGTHS_CB: [u8; 256] = [
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
];

const OPCODE_REGISTERS: [char; 8] = ['b', 'c', 'd', 'e', 'h', 'l', 'n', 'a'];
const ZERO_FLAG: u8 = 0b10000000;
const SUBTRACT_FLAG: u8 = 0b01000000;
const HALF_CARRY_FLAG: u8 = 0b00100000;
const CARRY_FLAG: u8 = 0b00010000;

impl Cpu {
    pub(crate) fn new() -> Cpu {
        Cpu {
            registers: Registers {
                af: 0,
                bc: 0,
                de: 0,
                hl: 0,
                sp: 0,
                pc: 0,
                ime: 0,
            },
        }
    }
    //load instructions
    fn ld_nn(&mut self, mem: &mut Memory, reg: &str) {
        let value = mem.read_16(self.registers.read_16("pc") + 1);
        self.registers.write_16(reg, value);
    }

    fn ld_a_nn(&mut self, mem: &Memory) {
        let value = mem.read_8(self.registers.read_16("pc") + 1);
        self.registers.write_8('a', value)
    }

    fn ld_n(&mut self, mem: &mut Memory, reg: &str) {
        let value = mem.read_8(self.registers.read_16("pc") + 1);
        self.registers.write_8(reg.chars().next().unwrap(), value);
    }

    fn ld_r1_r2(&mut self, mem: &mut Memory, dest: &str, source: &str) {
        //if dest contains (, it's a memory address change it to the dest withouth the () and set dstmem to true, same for source
        let mut dstmem = false;
        let mut srcmem = false;
        let mut dest = dest;
        let mut source = source;
        let mut temp = String::new();
        let mut temp2 = String::new();
        if dest.contains("(") {
            dstmem = true;

            temp = dest.replace("(", "").replace(")", "");
            dest = &temp;
        }
        if source.contains("(") {
            srcmem = true;
            temp2 = source.replace("(", "").replace(")", "");
            source = &temp2;
        }

        //load from memory(r1) to r2
        //if source have 1 char, it's a 8 bit register if dest have 1 char, it's a 8 bit register, if scrmem is true, it's a memory address(only for 16 bit Registers) same for dstmem
        if source.len() == 1 {
            //source is 8 bit register can't be memory address
            if dest.len() == 1 {
                //dest is 8 bit register
                let value = self.registers.read_8(source.chars().next().unwrap());
                self.registers.write_8(dest.chars().next().unwrap(), value);
            } else {
                //dest is 16 bit register
                let value = self.registers.read_8(source.chars().next().unwrap());
                if dstmem {
                    mem.write_8(self.registers.read_16(dest), value);
                } else {
                    self.registers.write_16(dest, value as u16);
                }
            }
        } else {
            //source is 16 bit register could be memory address
            let value: u16 = if srcmem {
                mem.read_16(self.registers.read_16(source))
            } else {
                self.registers.read_16(source)
            };
            if dest.len() == 1 {
                self.registers
                    .write_8(dest.chars().next().unwrap(), value as u8);
            } else {
                self.registers.write_16(dest, value);
            }
        }
    }

    fn ld_nn_a(&mut self, mem: &mut Memory) {
        let value = self.registers.read_8('a');
        mem.write_8(mem.read_16(self.registers.read_16("pc") + 1), value);
    }

    fn ld_m_n(&mut self, mem: &mut Memory) {
        let value = mem.read_8(self.registers.read_16("pc") + 1);
        mem.write_8(self.registers.read_16("hl"), value);
    }

    fn ld_sp_e(&mut self, mem: &mut Memory) {
        let sp = mem.read_16(self.registers.read_16("af"));
        self.registers.write_16(
            "hl",
            sp + mem.read_8(self.registers.read_16("PC") + 1) as u16,
        );
    }

    fn ld_sp_hl(&mut self, mem: &mut Memory) {
        let value = self.registers.read_16("hl");
        mem.write_16(self.registers.read_16("sp"), value);
    }

    fn ldh_n_a(&mut self, mem: &mut Memory) {
        let value = self.registers.read_8('a');
        mem.write_8(
            0xFF00 + mem.read_8(self.registers.read_16("pc") + 1) as u16,
            value,
        );
    }

    fn ldh_a_n(&mut self, mem: &mut Memory) {}

    fn ldh_c_a(&mut self, mem: &mut Memory) {
        let value = self.registers.read_8('a');
        mem.write_8(0xFF00 + self.registers.read_8('c') as u16, value);
    }

    fn ldh_a_c(&mut self, mem: &mut Memory) {
        self.registers.write_8(
            'a',
            mem.read_8(0xFF00 + (self.registers.read_8('c') as u16)),
        );
    }

    fn pop(&mut self, mem: &mut Memory, reg: &str) {
        let value = mem.read_16(self.registers.read_16("sp"));
        self.registers.write_16(reg, value);
        self.registers
            .write_16("sp", self.registers.clone().read_16("sp") + 2);
    }

    fn push(&mut self, mem: &mut Memory, reg: &str) {
        let mut value = self.registers.read_16(reg);
        //if regs is af then the last 4 bits are 0
        if reg == "af" {
            value = value ^ 0b0000000000001111;
        }
        self.registers
            .write_16("sp", self.registers.clone().read_16("sp") - 2);
        mem.write_16(self.registers.read_16("sp"), value);
    }

    // incr and decr
    fn inc_r(&mut self, mem: &mut Memory, reg: &str) {
        //if reg have 1 char, it's a 8 bit register
        if reg.len() == 1 {
            let value = self.registers.read_8(reg.chars().next().unwrap());
            self.registers
                .write_8(reg.chars().next().unwrap(), value + 1);
        } else {
            let value = self.registers.read_16(reg);
            self.registers.write_16(reg, value + 1);
        }
    }

    fn dec_r(&mut self, mem: &mut Memory, reg: &str) {
        //if reg have 1 char, it's a 8 bit register
        if reg.len() == 1 {
            let value = self.registers.read_8(reg.chars().next().unwrap());
            self.registers
                .write_8(reg.chars().next().unwrap(), value - 1);
        } else {
            let value = self.registers.read_16(reg);
            self.registers.write_16(reg, value - 1);
        }
    }

    fn inc_m(&mut self, mem: &mut Memory, reg: &str) {
        let value = mem.read_8(self.registers.read_16(reg));
        mem.write_8(self.registers.read_16(reg), value + 1);
    }

    fn dec_m(&mut self, mem: &mut Memory, reg: &str) {
        let value = mem.read_8(self.registers.read_16(reg));
        mem.write_8(self.registers.read_16(reg), value - 1);
    }

    //rotate and shift
    fn rlca(&mut self) {
        let value = self.registers.read_8('a').clone();
        let msb = value & 0x80;
        let new_value = (value << 1) | msb;
        self.registers.write_8('a', new_value);
        let mut flags = self.registers.read_8('f');
        //reset N and H flags bits 5 and 6 and Z flag
        flags &= !(HALF_CARRY_FLAG | SUBTRACT_FLAG | ZERO_FLAG);
        //set carry flag if carry from bit 7
        if msb != 0 {
            flags |= CARRY_FLAG;
        }

        self.registers.write_8('f', flags);
    }

    fn rla(&mut self) {
        let value = self.registers.read_8('a');
        let msb = value & 0x80;
        let new_value = (value << 1) | ((self.registers.read_8('f') & CARRY_FLAG) >> 4);
        self.registers.write_8('a', new_value);
        let mut flags = self.registers.read_8('f');
        //reset N and H flags bits 5 and 6 and Z flag
        flags &= !(HALF_CARRY_FLAG | SUBTRACT_FLAG | ZERO_FLAG);
        //set carry flag if carry from bit 7
        if msb != 0 {
            flags |= CARRY_FLAG;
        }

        self.registers.write_8('f', flags);
    }
    fn rrca(&mut self) {
        let value = self.registers.read_8('a');
        let lsb = value & 0x01;
        let new_value = (value >> 1) | (lsb << 7);
        self.registers.write_8('a', new_value);
        let mut flags = self.registers.read_8('f');
        //reset N and H flags bits 5 and 6 and Z flag
        flags &= !(HALF_CARRY_FLAG | SUBTRACT_FLAG | ZERO_FLAG);
        //set carry flag if carry from bit 0
        if lsb != 0 {
            flags |= CARRY_FLAG;
        }

        self.registers.write_8('f', flags);
    }

    fn rra(&mut self, mem: &mut Memory) {
        let value = self.registers.read_8('a');
        let lsb = value & 0x01;
        let new_value = (value >> 1) | ((self.registers.read_8('f') & CARRY_FLAG) << 3);
        self.registers.write_8('a', new_value);
        let mut flags = self.registers.read_8('f');
        //reset N and H flags bits 5 and 6 and Z flag
        flags &= !(HALF_CARRY_FLAG | SUBTRACT_FLAG | ZERO_FLAG);
        //set carry flag if carry from bit 0
        if lsb != 0 {
            flags |= CARRY_FLAG;
        }

        self.registers.write_8('f', flags);
    }

    //arithmetic and logic
    fn add_hl(&mut self, mem: &mut Memory, reg: &str) {
        let value = self.registers.read_16(reg);
        let hl = self.registers.read_16("hl");
        let result: u32 = value as u32 + hl as u32;
        self.registers.write_16("hl", result as u16);
        let mut flags = self.registers.read_8('f');
        //reset N
        flags &= !SUBTRACT_FLAG;
        //set carry flag if carry from bit 15
        if result > 0xFFFF {
            flags |= CARRY_FLAG;
        }
        //set H flag if carry from bit 11
        if (value & 0xFFF) + (hl & 0xFFF) > 0xFFF {
            flags |= HALF_CARRY_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn add_a_r(&mut self, mem: &mut Memory, reg: &str) {
        //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
        let value = if reg.len() > 1 {
            mem.read_8(
                self.registers
                    .read_16(&reg.replace("(", "").replace(")", "")),
            )
        } else {
            self.registers.read_8(reg.chars().next().unwrap())
        };
        let a = self.registers.read_8('a');
        let result: u16 = value as u16 + a as u16;
        self.registers.write_8('a', result as u8);
        let mut flags = self.registers.read_8('f');
        //reset N
        flags &= !SUBTRACT_FLAG;
        //set carry flag if carry from bit 7
        if result > 0xFF {
            flags |= CARRY_FLAG;
        }
        //set H flag if carry from bit 3
        if (value & 0xF) + (a & 0xF) > 0xF {
            flags |= HALF_CARRY_FLAG;
        }
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result as u8 == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn add_a_n(&mut self, mem: &mut Memory) {
        let value = mem.read_8(self.registers.read_16("pc") + 1);
        let a = self.registers.read_8('a');
        let result: u16 = value as u16 + a as u16;
        self.registers.write_8('a', result as u8);
        let mut flags = self.registers.read_8('f');
        //reset N
        flags &= !SUBTRACT_FLAG;
        //set carry flag if carry from bit 7
        if result > 0xFF {
            flags |= CARRY_FLAG;
        }
        //set H flag if carry from bit 3
        if (value & 0xF) + (a & 0xF) > 0xF {
            flags |= HALF_CARRY_FLAG;
        }
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result as u8 == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn add_sp_e(&mut self, mem: &mut Memory) {
        let value = mem.read_8(self.registers.read_16("pc") + 1) as i16;
        let sp = self.registers.read_16("sp");
        let result: i32 = value as i32 + sp as i32;
        self.registers.write_16("sp", result as u16);
        let mut flags = self.registers.read_8('f');
        //reset N and Z
        flags &= !SUBTRACT_FLAG;
        flags &= !ZERO_FLAG;
        //set carry flag if carry from bit 15
        flags &= !CARRY_FLAG;
        if result > 0xFFFF {
            flags |= CARRY_FLAG;
        }
        //set H flag if carry from bit 11
        if (value as u16 & 0xFFF) + (sp & 0xFFF) > 0xFFF {
            flags |= HALF_CARRY_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn adc_a_r(&mut self, mem: &mut Memory, reg: &str) {
        //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
        let value = if reg.len() > 1 {
            mem.read_8(
                self.registers
                    .read_16(&reg.replace("(", "").replace(")", "")),
            )
        } else {
            self.registers.read_8(reg.chars().next().unwrap())
        };
        let a = self.registers.read_8('a');
        let carry = (self.registers.read_8('f') & CARRY_FLAG) >> 4;

        let result: u16 = value as u16 + a as u16 + carry as u16;
        self.registers.write_8('a', result as u8);
        let mut flags = self.registers.read_8('f');
        //reset N
        flags &= !SUBTRACT_FLAG;
        //set carry flag if carry from bit 7
        if result > 0xFF {
            flags |= CARRY_FLAG;
        }
        //set H flag if carry from bit 3
        if (value & 0xF) + (a & 0xF) + carry > 0xF {
            flags |= HALF_CARRY_FLAG;
        }
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result as u8 == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn adc_a_n(&mut self, mem: &mut Memory) {
        let value = mem.read_8(self.registers.read_16("pc") + 1);
        let a = self.registers.read_8('a');
        let carry = (self.registers.read_8('f') & CARRY_FLAG) >> 4;

        let result: u16 = value as u16 + a as u16 + carry as u16;
        self.registers.write_8('a', result as u8);
        let mut flags = self.registers.read_8('f');
        //reset N
        flags &= !SUBTRACT_FLAG;
        //set carry flag if carry from bit 7
        if result > 0xFF {
            flags |= CARRY_FLAG;
        }
        //set H flag if carry from bit 3
        if (value & 0xF) + (a & 0xF) + carry > 0xF {
            flags |= HALF_CARRY_FLAG;
        }
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result & 0xFF == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn sub_a_r(&mut self, mem: &mut Memory, reg: &str) {
        //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
        let value = if reg.len() > 1 {
            mem.read_8(
                self.registers
                    .read_16(&reg.replace("(", "").replace(")", "")),
            )
        } else {
            self.registers.read_8(reg.chars().next().unwrap())
        };
        let a = self.registers.read_8('a');
        let result: u16 = a as u16 - value as u16;
        self.registers.write_8('a', result as u8);
        let mut flags = self.registers.read_8('f');
        //set N
        flags |= SUBTRACT_FLAG;
        //set carry flag if carry from bit 7
        if result > 0xFF {
            flags |= CARRY_FLAG;
        }
        //set H flag if carry from bit 3
        if (value & 0xF) > (a & 0xF) {
            flags |= HALF_CARRY_FLAG;
        }
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result as u8 == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn sub_a_n(&mut self, mem: &mut Memory) {
        let value = mem.read_8(self.registers.read_16("pc") + 1);
        let a = self.registers.read_8('a');
        let result: u16 = a as u16 - value as u16;
        self.registers.write_8('a', result as u8);
        let mut flags = self.registers.read_8('f');
        //set N
        flags |= SUBTRACT_FLAG;
        //set carry flag if carry from bit 7
        if result > 0xFF {
            flags |= CARRY_FLAG;
        }
        //set H flag if carry from bit 3
        if (value & 0xF) > (a & 0xF) {
            flags |= HALF_CARRY_FLAG;
        }
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result as u8 == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn sbc_a_r(&mut self, mem: &mut Memory, reg: &str) {
        //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
        let value = self.register_or_memory(mem, &reg);
        let a = self.registers.read_8('a');
        let carry = (self.registers.read_8('f') & CARRY_FLAG) >> 4;

        let result: u16 = a as u16 - value as u16 - carry as u16;
        self.registers.write_8('a', result as u8);
        let mut flags = self.registers.read_8('f');
        //set N
        flags |= SUBTRACT_FLAG;
        //set carry flag if carry from bit 7
        if result > 0xFF {
            flags |= CARRY_FLAG;
        }
        //set H flag if carry from bit 3
        if (value & 0xF) + carry > (a & 0xF) {
            flags |= HALF_CARRY_FLAG;
        }
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result as u8 == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn sbc_a_n(&mut self, mem: &mut Memory) {
        let value = mem.read_8(self.registers.read_16("pc") + 1);
        let a = self.registers.read_8('a');
        let carry = (self.registers.read_8('f') & CARRY_FLAG) >> 4;

        let result: u16 = a as u16 - value as u16 - carry as u16;
        self.registers.write_8('a', result as u8);
        let mut flags = self.registers.read_8('f');
        //set N
        flags |= SUBTRACT_FLAG;
        //set carry flag if carry from bit 7
        if result > 0xFF {
            flags |= CARRY_FLAG;
        }
        //set H flag if carry from bit 3
        if (value & 0xF) + carry > (a & 0xF) {
            flags |= HALF_CARRY_FLAG;
        }
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result as u8 == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn daa(&mut self, mem: &mut Memory) {
        let mut value = self.registers.read_8('a');
        let mut flags = self.registers.read_8('f');
        let mut carry = flags & CARRY_FLAG;
        let half_carry = flags & HALF_CARRY_FLAG;
        let subtract = flags & SUBTRACT_FLAG;
        let mut correction: u16 = 0x00;

        if !subtract != 0 {
            if half_carry != 0 || (value & 0x0F) > 0x09 {
                correction += 0x06;
            }
            if carry != 0 || value > 0x99 {
                correction += 0x60;
            }
            value = value.wrapping_add(correction as u8);
        } else {
            if carry != 0 {
                correction += 0x60;
            }
            if half_carry != 0 {
                correction += 0x06;
            }
            value = value.wrapping_sub(correction as u8);
        }
        if value > 0x99 {
            carry = CARRY_FLAG;
        }
        flags &= !ZERO_FLAG;
        if value == 0 {
            flags |= ZERO_FLAG;
        }
        flags &= !HALF_CARRY_FLAG;

        if carry != 0 {
            flags |= CARRY_FLAG;
        }
        self.registers.write_8('f', flags);
        self.registers.write_8('a', value);
    }

    fn cpl(&mut self) {
        let value = self.registers.read_8('a');
        self.registers.write_8('a', !value);
    }

    fn and_a_r(&mut self, mem: &mut Memory, reg: &str) {
        //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
        let value = self.register_or_memory(mem, &reg);
        let a = self.registers.read_8('a');
        let result = a & value;
        self.registers.write_8('a', result);
        let mut flags = self.registers.read_8('f');
        //set H
        flags |= HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set C
        flags &= !CARRY_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result as u8 == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn and_a_n(&mut self, mem: &mut Memory) {
        let value = mem.read_8(self.registers.read_16("pc") + 1);
        let a = self.registers.read_8('a');
        let result = a & value;
        self.registers.write_8('a', result);
        let mut flags = self.registers.read_8('f');
        //set H
        flags |= HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set C
        flags &= !CARRY_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn xor_a_r(&mut self, mem: &mut Memory, reg: &str) {
        //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
        let value = self.register_or_memory(mem, &reg);
        let a = self.registers.read_8('a');
        let result = a ^ value;
        self.registers.write_8('a', result);
        let mut flags = self.registers.read_8('f');
        //set H
        flags &= !HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set C
        flags &= !CARRY_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result as u8 == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn xor_a_n(&mut self, mem: &mut Memory) {
        let value = mem.read_8(self.registers.read_16("pc") + 1);
        let a = self.registers.read_8('a');
        let result = a ^ value;
        self.registers.write_8('a', result);
        let mut flags = self.registers.read_8('f');
        //set H
        flags &= !HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set C
        flags &= !CARRY_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn or_a_r(&mut self, mem: &mut Memory, reg: &str) {
        //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
        let value = self.register_or_memory(mem, &reg);
        let a = self.registers.read_8('a');
        let result = a | value;
        self.registers.write_8('a', result);
        let mut flags = self.registers.read_8('f');
        //set H
        flags &= !HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set C
        flags &= !CARRY_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result as u8 == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn or_a_n(&mut self, mem: &mut Memory) {
        let value = mem.read_8(self.registers.read_16("pc") + 1);
        let a = self.registers.read_8('a');
        let result = a | value;
        self.registers.write_8('a', result);
        let mut flags = self.registers.read_8('f');
        //set H
        flags |= HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set C
        flags &= !CARRY_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn cp_a_r(&mut self, mem: &mut Memory, reg: &str) {
        //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
        let value = self.register_or_memory(mem, &reg);
        let a = self.registers.read_8('a');
        let result: u16 = a as u16 - value as u16;
        let mut flags = self.registers.read_8('f');
        //set N
        flags |= SUBTRACT_FLAG;
        //set carry flag if carry from bit 7
        if result > 0xFF {
            flags |= CARRY_FLAG;
        }
        //set H flag if carry from bit 3
        if (value & 0xF) > (a & 0xF) {
            flags |= HALF_CARRY_FLAG;
        }
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result as u8 == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn cp_a_n(&mut self, mem: &mut Memory) {
        let value = mem.read_8(self.registers.read_16("pc") + 1);
        let a = self.registers.read_8('a');
        let result: u16 = a as u16 - value as u16;
        let mut flags = self.registers.read_8('f');
        //set N
        flags |= SUBTRACT_FLAG;
        //set carry flag if carry from bit 7
        if result > 0xFF {
            flags |= CARRY_FLAG;
        }
        //set H flag if carry from bit 3
        if (value & 0xF) > (a & 0xF) {
            flags |= HALF_CARRY_FLAG;
        }
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if result as u8 == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }
    //utils

    fn register_or_memory(&mut self, mem: &mut Memory, reg: &&str) -> u8 {
        let value = if reg.len() > 1 {
            mem.read_8(self.registers.read_16(&reg.replace(['(', ')'], "")))
        } else {
            self.registers.read_8(reg.chars().next().unwrap())
        };
        value
    }

    //misc
    fn nop(&mut self) {
        //do nothing
    }

    fn stop(&mut self) {
        //stop Cpu until button pressed
    }

    fn scf(&mut self) {
        let mut flags = self.registers.read_8('f');
        flags |= CARRY_FLAG;
        flags &= !HALF_CARRY_FLAG;
        flags &= !SUBTRACT_FLAG;
        self.registers.write_8('f', flags);
    }

    fn ccf(&mut self) {
        let mut flags = self.registers.read_8('f');
        flags ^= CARRY_FLAG;
        flags &= !HALF_CARRY_FLAG;
        flags &= !SUBTRACT_FLAG;
        self.registers.write_8('f', flags);
    }

    //cb instructions
    fn write_mem_or_regs(&mut self, mem: &mut Memory, reg: &&str, result: u8) {
        if reg.len() > 1 {
            mem.write_8(self.registers.read_16(&reg.replace(['(', ')'], "")), result);
        } else {
            self.registers.write_8(reg.chars().next().unwrap(), result);
        }
    }

    fn rlc_r(&mut self, mem: &mut Memory, reg: &str) {
        let value = self.register_or_memory(mem, &reg);
        let mut flags = self.registers.read_8('f');
        //set carry flag if bit 7 is set
        flags &= !CARRY_FLAG;
        if value & 0x80 != 0 {
            flags |= CARRY_FLAG;
        }
        //set H
        flags &= !HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        let result = value.rotate_left(1);
        if result == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
        self.write_mem_or_regs(mem, &reg, result);
    }

    fn rrc_r(&mut self, mem: &mut Memory, reg: &str) {
        let value = self.register_or_memory(mem, &reg);
        let mut flags = self.registers.read_8('f');
        //set carry flag if bit 0 is set
        flags &= !CARRY_FLAG;
        if value & 0x01 != 0 {
            flags |= CARRY_FLAG;
        }
        //set H
        flags &= !HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        let result = value.rotate_right(1);
        if result == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
        self.write_mem_or_regs(mem, &reg, result);
    }

    fn rl_r(&mut self, mem: &mut Memory, reg: &str) {
        let value = self.register_or_memory(mem, &reg);
        let mut flags = self.registers.read_8('f');
        //set carry flag if bit 7 is set
        let mut carry = 0;
        if flags & CARRY_FLAG != 0 {
            carry = 1;
        }
        flags &= !CARRY_FLAG;
        if value & 0x80 != 0 {
            flags |= CARRY_FLAG;
        }
        //set H
        flags &= !HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        let result = (value << 1) | carry;
        if result == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
        self.write_mem_or_regs(mem, &reg, result);
    }

    fn rr_r(&mut self, mem: &mut Memory, reg: &str) {
        let value = self.register_or_memory(mem, &reg);
        let mut flags = self.registers.read_8('f');
        //set carry flag if bit 0 is set
        let mut carry = 0;
        if flags & CARRY_FLAG != 0 {
            carry = 1;
        }
        flags &= !CARRY_FLAG;
        if value & 0x01 != 0 {
            flags |= CARRY_FLAG;
        }
        //set H
        flags &= !HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        let result = (value >> 1) | (carry << 7);
        if result == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
        self.write_mem_or_regs(mem, &reg, result);
    }

    fn sla_r(&mut self, mem: &mut Memory, reg: &str) {
        let value = self.register_or_memory(mem, &reg) as i8;
        let mut flags = self.registers.read_8('f');
        //set carry flag if bit 7 is set
        flags &= !CARRY_FLAG;
        if value as u8 & 0x80 != 0 {
            flags |= CARRY_FLAG;
        }
        //set H
        flags &= !HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        let result = value << 1;
        if result == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
        self.write_mem_or_regs(mem, &reg, result as u8);
    }

    fn sra_r(&mut self, mem: &mut Memory, reg: &str) {
        let value = self.register_or_memory(mem, &reg) as i8;
        let mut flags = self.registers.read_8('f');
        //set carry flag if bit 0 is set
        flags &= !CARRY_FLAG;
        if value & 0x01 != 0 {
            flags |= CARRY_FLAG;
        }
        //set H
        flags &= !HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        let result = value >> 1;
        if result == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
        self.write_mem_or_regs(mem, &reg, result as u8);
    }

    fn swap_r(&mut self, mem: &mut Memory, reg: &str) {
        let value = self.register_or_memory(mem, &reg);
        let mut flags = self.registers.read_8('f');
        //set H
        flags &= !HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        let result = (value >> 4) | (value << 4);
        if result == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
        self.write_mem_or_regs(mem, &reg, result);
    }

    fn bit_n_r(&mut self, mem: &mut Memory, reg: &str, n: u8) {
        let value = self.register_or_memory(mem, &reg);
        let mut flags = self.registers.read_8('f');
        //set H
        flags |= HALF_CARRY_FLAG;
        //reset N
        flags &= !SUBTRACT_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        if value & (1 << n) == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
    }

    fn srl_r(&mut self, mem: &mut Memory, reg: &str) {
        let value = self.register_or_memory(mem, &reg);
        let mut flags = self.registers.read_8('f');
        //set carry flag if bit 0 is set
        flags &= !CARRY_FLAG;
        if value & 0x01 != 0 {
            flags |= CARRY_FLAG;
        }
        //set H
        flags &= !HALF_CARRY_FLAG;
        //set N
        flags &= !SUBTRACT_FLAG;
        //set Z flag if result is 0
        flags &= !ZERO_FLAG;
        let result = value >> 1;
        if result == 0 {
            flags |= ZERO_FLAG;
        }
        self.registers.write_8('f', flags);
        self.write_mem_or_regs(mem, &reg, result);
    }

    fn res_n_r(&mut self, mem: &mut Memory, reg: &str, n: u8) {
        let value = self.register_or_memory(mem, &reg);
        let result = value & !(1 << n);
        self.write_mem_or_regs(mem, &reg, result);
    }

    fn set_n_r(&mut self, mem: &mut Memory, reg: &str, n: u8) {
        let value = self.register_or_memory(mem, &reg);
        let result = value | (1 << n);
        self.write_mem_or_regs(mem, &reg, result);
    }

    fn call_CB(&mut self, mem: &mut Memory) {
        //TODO change name
        const REG_NAMES: [&str; 8] = ["b", "c", "d", "e", "h", "l", "(hl)", "a"];
        //TODO implement CB instructions
        let cb_opcode = mem.read_8(self.registers.read_16("pc") + 1);
        match cb_opcode {
            0x00..0x07 => self.rlc_r(mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
            0x08..0x0F => self.rrc_r(mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
            0x10..0x17 => self.rl_r(mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
            0x18..0x1F => self.rr_r(mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
            0x20..0x27 => self.sla_r(mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
            0x28..0x2F => self.sra_r(mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
            0x30..0x37 => self.swap_r(mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
            0x38..0x3F => self.srl_r(mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
            0x40..0x7F => self.bit_n_r(
                mem,
                REG_NAMES[(cb_opcode & 0x0f) as usize],
                (cb_opcode >> 3) & 0x07,
            ),
            0x80..0xBF => self.res_n_r(
                mem,
                REG_NAMES[(cb_opcode & 0x0f) as usize],
                (cb_opcode >> 4) & 0x07,
            ),
            0xC0..0xFF => self.set_n_r(
                mem,
                REG_NAMES[(cb_opcode & 0x0f) as usize],
                (cb_opcode >> 5) & 0x07,
            ),
            _ => panic!("Unknown CB opcode: {:x}", cb_opcode),
        }
    }

    fn di(&mut self) {
        self.registers.ime = 0;
    }

    fn ei(&mut self) {
        self.registers.ime = 1;
    }

    //flow
    fn jr_e(&mut self, mem: &mut Memory) {
        //temp register
        let tmp = self.registers.read_16("pc").clone() as i16;
        //read next byte
        let value = mem.read_8(tmp.clone() as u16 + 1) as i8;
        self.registers.write_16("pc", (tmp + value as i16) as u16);
    }

    fn jr_f_e(&mut self, mem: &mut Memory, cflag: char, z: bool) {
        //get flag
        let flag = match cflag {
            'c' => CARRY_FLAG,
            'z' => ZERO_FLAG,
            _ => panic!("Invalid flag"),
        };
        let shift = match cflag {
            'c' => 4,
            'z' => 7,
            _ => panic!("Invalid flag"),
        };
        //temp register
        let tmp = self.registers.read_16("pc") as i16;
        //read next byte
        let value = mem.read_8(tmp as u16 + 1) as i8;
        let cond = if z { 1 } else { 0 };
        if (self.registers.read_8('f') & flag) >> shift == cond {
            self.registers.write_16("pc", (tmp + value as i16) as u16);
        }
    }

    fn jp_nn(&mut self, mem: &mut Memory) {
        let value = mem.read_16(self.registers.read_16("pc") + 1) - 3;
        //correct for the 2 bytes read
        self.registers.write_16("pc", value);
    }

    fn jp_f_nn(&mut self, mem: &mut Memory, cflag: char, condition: bool) {
        //get flag
        let flag = match cflag {
            'c' => CARRY_FLAG,
            'z' => ZERO_FLAG,
            _ => panic!("Invalid flag"),
        };

        //get shift value
        let shift = match cflag {
            'c' => 4,
            'z' => 7,
            _ => panic!("Invalid flag"),
        };

        let cond = if condition { 1 } else { 0 };

        if (self.registers.read_8('f') & flag) >> shift == cond {
            let value = mem.read_16(self.registers.read_16("pc") + 1) - 3;
            //correct for the 2 bytes read
            self.registers.write_16("pc", value);
        }
    }

    fn jp_hl(&mut self) {
        self.registers.write_16("pc", self.registers.read_16("hl"));
    }

    fn call_nn(&mut self, mem: &mut Memory) {
        let value = mem.read_16(self.registers.read_16("pc") + 1) - 3;
        //correct for the 2 bytes read
        self.registers
            .write_16("sp", self.registers.clone().read_16("sp") - 2);
        mem.write_16(
            self.registers.clone().read_16("sp"),
            self.registers.clone().read_16("pc"),
        );
        self.registers.write_16("pc", value);
    }

    fn call_f_nn(&mut self, mem: &mut Memory, cflag: char, z: bool) {
        //get flag
        let flag = match cflag {
            'c' => CARRY_FLAG,
            'z' => ZERO_FLAG,
            _ => panic!("Invalid flag"),
        };

        //get shift value
        let shift = match cflag {
            'c' => 4,
            'z' => 7,
            _ => panic!("Invalid flag"),
        };

        let cond = if z { 1 } else { 0 };

        if (self.registers.read_8('f') & flag) >> shift == cond {
            let value = mem.read_16(self.registers.clone().read_16("pc") + 1) - 3;
            //correct for the 2 bytes read
            self.registers
                .write_16("sp", self.registers.clone().read_16("sp") - 2);
            mem.write_16(
                self.registers.clone().read_16("sp"),
                self.registers.clone().read_16("pc") + 3,
            );
            self.registers.write_16("pc", value);
        }
    }

    fn rst(&mut self, mem: &mut Memory, value: u16) {
        self.registers
            .write_16("sp", self.registers.clone().read_16("sp") - 2);
        mem.write_16(
            self.registers.clone().read_16("sp"),
            self.registers.clone().read_16("pc") + 1,
        );
        self.registers.write_16("pc", value - 1);
    }

    fn ret(&mut self, mem: &mut Memory) {
        let value = mem.read_16(self.registers.clone().read_16("sp"));
        self.registers
            .write_16("sp", self.registers.clone().read_16("sp") + 2);
        self.registers.write_16("pc", value - 1);
    }

    fn ret_f(&mut self, mem: &mut Memory, cflag: char, z: bool) {
        //get flag
        let flag = match cflag {
            'c' => CARRY_FLAG,
            'z' => ZERO_FLAG,
            _ => panic!("Invalid flag"),
        };

        //get shift value
        let shift = match cflag {
            'c' => 4,
            'z' => 7,
            _ => panic!("Invalid flag"),
        };

        let cond = if z { 1 } else { 0 };

        if (self.registers.read_8('f') & flag) >> shift == cond {
            let value = mem.read_16(self.registers.clone().read_16("sp"));
            self.registers
                .write_16("sp", self.registers.clone().read_16("sp") + 2);
            self.registers.write_16("pc", value - 1);
        }
    }

    fn reti(&mut self, mem: &mut Memory) {
        let value = mem.read_16(self.registers.clone().read_16("sp"));
        self.registers
            .write_16("sp", self.registers.clone().read_16("sp") + 2);
        self.registers.write_16("pc", value - 1);
        self.registers.write_8('i', 1);
    }

    //end of Cpu
    fn execute(&mut self, opcode: u8, mem: &mut Memory) {
        const REG_NAMES: [&str; 8] = ["b", "c", "d", "e", "h", "l", "(hl)", "a"];
        match opcode {
            0x00 => self.nop(),
            0x01 => self.ld_nn(mem, "bc"),
            0x02 => self.ld_r1_r2(mem, "(bc)", "a"), //load (bc) into a, bc is the memory address
            0x03 => self.inc_r(mem, "bc"),
            0x04 => self.inc_r(mem, "b"),
            0x05 => self.dec_r(mem, "b"),
            0x06 => self.ld_n(mem, "b"),
            0x07 => self.rlca(),
            0x08 => self.ld_nn(mem, "sp"),
            0x09 => self.add_hl(mem, "bc"),
            0x0A => self.ld_r1_r2(mem, "a", "(bc)"), //load a into (bc), bc is the memory address
            0x0B => self.dec_r(mem, "bc"),
            0x0C => self.inc_r(mem, "c"),
            0x0D => self.dec_r(mem, "c"),
            0x0E => self.ld_n(mem, "c"),
            0x0F => self.rrca(),
            0x10 => self.stop(),
            0x11 => self.ld_nn(mem, "de"),
            0x12 => self.ld_r1_r2(mem, "(de)", "a"), //load (de) into a, de is the memory address
            0x13 => self.inc_r(mem, "de"),
            0x14 => self.inc_r(mem, "d"),
            0x15 => self.dec_r(mem, "d"),
            0x16 => self.ld_n(mem, "d"),
            0x17 => self.rla(),
            0x18 => self.jr_e(mem),
            0x19 => self.add_hl(mem, "de"),
            0x1A => self.ld_r1_r2(mem, "a", "(de)"), //load a into (de), de is the memory address
            0x1B => self.dec_r(mem, "de"),
            0x1C => self.inc_r(mem, "e"),
            0x1D => self.dec_r(mem, "e"),
            0x1E => self.ld_n(mem, "e"),
            0x1F => self.rra(mem),
            0x20 => self.jr_f_e(mem, 'z', false),
            0x21 => self.ld_nn(mem, "hl"),
            0x22 => {
                self.ld_r1_r2(mem, "(hl)", "a");
                self.inc_r(mem, "hl");
            } //load a into (hl), hl is the memory address, then increment hl
            0x23 => self.inc_r(mem, "hl"),
            0x24 => self.inc_r(mem, "h"),
            0x25 => self.dec_r(mem, "h"),
            0x26 => self.ld_n(mem, "h"),
            0x27 => self.daa(mem),
            0x28 => self.jr_f_e(mem, 'z', true),
            0x29 => self.add_hl(mem, "hl"),
            0x2A => {
                self.ld_r1_r2(mem, "a", "(hl)");
                self.inc_r(mem, "hl");
            } //load (hl) into a, hl is the memory address, then increment hl
            0x2B => self.dec_r(mem, "hl"),
            0x2C => self.inc_r(mem, "l"),
            0x2D => self.dec_r(mem, "l"),
            0x2E => self.ld_n(mem, "l"),
            0x2F => self.cpl(),
            0x30 => self.jr_f_e(mem, 'c', false),
            0x31 => self.ld_nn(mem, "sp"),
            0x32 => {
                self.ld_r1_r2(mem, "(hl)", "a");
                self.dec_r(mem, "hl");
            } //load a into (hl), hl is the memory address, then decrement hl
            0x33 => self.inc_r(mem, "sp"),
            0x34 => self.inc_m(mem, "hl"),
            0x35 => self.dec_m(mem, "hl"),
            0x36 => self.ld_m_n(mem),
            0x37 => self.scf(),
            0x38 => self.jr_f_e(mem, 'c', true),
            0x39 => self.add_hl(mem, "sp"),
            0x3A => {
                self.ld_r1_r2(mem, "a", "(hl)");
                self.dec_r(mem, "hl");
            } //load (hl) into a, hl is the memory address, then decrement hl
            0x3B => self.dec_r(mem, "sp"),
            0x3C => self.inc_r(mem, "a"),
            0x3D => self.dec_r(mem, "a"),
            0x3E => self.ld_n(mem, "a"),
            0x3F => self.ccf(),
            0x40..0x7F => self.ld_r1_r2(
                mem,
                REG_NAMES[((opcode >> 3) & 0b111) as usize],
                REG_NAMES[(opcode & 0x0f) as usize],
            ),
            0x80..0x87 => self.add_a_r(mem, REG_NAMES[(opcode & 0x0f) as usize]),
            0x88..0x8F => self.adc_a_r(mem, REG_NAMES[(opcode & 0x0f) as usize]),
            0x90..0x97 => self.sub_a_r(mem, REG_NAMES[(opcode & 0x0f) as usize]),
            0x98..0x9F => self.sbc_a_r(mem, REG_NAMES[(opcode & 0x0f) as usize]),
            0xA0..0xA7 => self.and_a_r(mem, REG_NAMES[(opcode & 0x0f) as usize]),
            0xA8..0xAF => self.xor_a_r(mem, REG_NAMES[(opcode & 0x0f) as usize]),
            0xB0..0xB7 => self.or_a_r(mem, REG_NAMES[(opcode & 0x0f) as usize]),
            0xB8..0xBF => self.cp_a_r(mem, REG_NAMES[(opcode & 0x0f) as usize]),
            0xC0 => self.ret_f(mem, 'z', false), //return if z flag is false
            0xC1 => self.pop(mem, "bc"),
            0xC2 => self.jp_f_nn(mem, 'z', false),
            0xC3 => self.jp_nn(mem),
            0xC4 => self.call_f_nn(mem, 'z', false),
            0xC5 => self.push(mem, "bc"),
            0xC6 => self.add_a_n(mem),
            0xC7 => self.rst(mem, 0x00),
            0xC8 => self.ret_f(mem, 'z', true), //return if z flag is true
            0xC9 => self.ret(mem),
            0xCA => self.jp_f_nn(mem, 'z', true),
            0xCB => self.call_CB(mem),
            0xCC => self.call_f_nn(mem, 'z', true),
            0xCD => self.call_nn(mem),
            0xCE => self.adc_a_n(mem),
            0xCF => self.rst(mem, 0x08),
            0xD0 => self.ret_f(mem, 'c', false), //return if c flag is false
            0xD1 => self.pop(mem, "de"),
            0xD2 => self.jp_f_nn(mem, 'c', false),
            0xD3 => unimplemented!(),
            0xD4 => self.call_f_nn(mem, 'c', false),
            0xD5 => self.push(mem, "de"),
            0xD6 => self.sub_a_n(mem),
            0xD7 => self.rst(mem, 0x10),
            0xD8 => self.ret_f(mem, 'c', true), //return if c flag is true
            0xD9 => self.reti(mem),
            0xDA => self.jp_f_nn(mem, 'c', true),
            0xDB => unimplemented!(),
            0xDC => self.call_f_nn(mem, 'c', true),
            0xDD => unimplemented!(),
            0xDE => self.sbc_a_n(mem),
            0xDF => self.rst(mem, 0x18),
            0xE0 => self.ldh_n_a(mem),
            0xE1 => self.pop(mem, "hl"),
            0xE2 => self.ldh_c_a(mem),
            0xE3..0xE4 => unimplemented!(),
            0xE5 => self.push(mem, "hl"),
            0xE6 => self.and_a_n(mem),
            0xE7 => self.rst(mem, 0x20),
            0xE8 => self.add_sp_e(mem),
            0xE9 => self.jp_hl(),
            0xEA => self.ld_nn_a(mem),
            0xEB..0xED => unimplemented!(),
            0xEE => self.xor_a_n(mem),
            0xEF => self.rst(mem, 0x28),
            0xF0 => self.ldh_a_n(mem),
            0xF1 => self.pop(mem, "af"),
            0xF2 => self.ldh_a_c(mem),
            0xF3 => self.di(),
            0xF4 => unimplemented!(),
            0xF5 => self.push(mem, "af"),
            0xF6 => self.or_a_n(mem),
            0xF7 => self.rst(mem, 0x30),
            0xF8 => self.ld_sp_e(mem),
            0xF9 => self.ld_sp_hl(mem),
            0xFA => self.ld_a_nn(mem),
            0xFB => self.ei(),
            0xFC..0xFD => unimplemented!(),
            0xFE => self.cp_a_n(mem),
            0xFF => self.rst(mem, 0x38),
            _ => println!("Opcode not implemented: {:2X}", opcode),
        }
    }

    fn handle_post_instruction(&mut self, mem: &mut Memory, opcode: u8, mut lenght: u64) {
        //increment pc
        let mut delta = 0;
        let pc = self.registers.read_16("pc").clone();
        self.registers
            .write_16("pc", pc + OPCODE_LENGTHS[opcode.clone() as usize] as u16);
        lenght += OPCODE_LENGTHS[opcode.clone() as usize] as u64;
        delta += OPCODE_LENGTHS[opcode as usize] as u64;
        if opcode == 0xCB {
            let pc = self.registers.read_16("pc").clone();
            let opcode = mem.read_8(pc);
            self.registers
                .write_16("pc", pc + OPCODE_LENGTHS_CB[opcode.clone() as usize] as u16);
            lenght += OPCODE_LENGTHS_CB[opcode.clone() as usize] as u64;
            delta += OPCODE_LENGTHS_CB[opcode as usize] as u64;
        }

        //handle interrupts
        //draw delta pixels if vblank
        //handle stuff
    }
}
