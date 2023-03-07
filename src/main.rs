#![feature(exclusive_range_pattern)]

mod cpu;
mod memory;

use crate::memory::Memory;
use std::fs::File;
use std::io::Read;
use std::ptr::null_mut;

struct CartridgeHeader {
    title: [char; 16],
    logo: [char; 48],
    c_type: u8,
    rom_size: u8,
    ram_size: u8,
    destination: u8,
    old_licensee: u8,
    mask_rom_version: u8,
    header_checksum: u8,
    global_checksum: [u8; 2],
}

const LCDC_REGISTER: u16 = 0xFF40;
//define bitmask for each flag to access them through and operation
const LCD_ENABLE: u8 = 0b10000000;
const WINDOW_TILE_MAP: u8 = 0b01000000;
const WINDOW_ENABLE: u8 = 0b00100000;
const BG_AND_WINDOW_TILE_DATA: u8 = 0b00010000;
const BG_TILE_MAP: u8 = 0b00001000;
const OBJ_ENABLE: u8 = 0b00000100;
const OBJ_SIZE: u8 = 0b00000010;
const BG_AND_WINDOW_TILE_MAP: u8 = 0b00000001;

const STAT: u16 = 0xFF41;
const MODE_FLAG: u8 = 0b00000011;
const LYC_FLAG: u8 = 0b00000100;
const HBLANK_FLAG: u8 = 0b00001000;
const VBLANK_FLAG: u8 = 0b00010000;
const OAM_FLAG: u8 = 0b00100000;
const LYC_INTERRUPT: u8 = 0b01000000;
const PIXEL_WIDTH: u16 = 160;
const PIXEL_HEIGHT: u16 = 144;
const PIXEL_SIZE: u16 = PIXEL_WIDTH * PIXEL_HEIGHT;

const INTERRUPT_ENABLE: u16 = 0xFFFF;
const INTERRUPT_FLAG: u16 = 0xFF0F;
const VBLANK_INTERRUPT: u8 = 0b00000001;
const LCD_INTERRUPT: u8 = 0b00000010;
const TIMER_INTERRUPT: u8 = 0b00000100;
const SERIAL_INTERRUPT: u8 = 0b00001000;
const JOYPAD_INTERRUPT: u8 = 0b00010000;

type Sprite = [u8; 4];

const ZERO_FLAG: u8 = 0b10000000;
const SUBTRACT_FLAG: u8 = 0b01000000;
const HALF_CARRY_FLAG: u8 = 0b00100000;
const CARRY_FLAG: u8 = 0b00010000;

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

    fn write_16(&mut self, register: &str, value: u16) {
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
const OPCODE_LENGHTS: [u8; 256] = [
    1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1, 1, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, 2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 1, 3, 3, 2, 1, 1, 1, 3, 0, 3, 1, 2, 1, 1, 1, 3, 0, 3, 0, 2, 1,
    2, 1, 1, 0, 0, 1, 2, 1, 2, 1, 3, 0, 0, 0, 2, 1, 2, 1, 1, 1, 0, 1, 2, 1, 2, 1, 3, 1, 0, 0, 2, 1,
];
const OPCODE_LENGHTS_CB: [u8; 256] = [
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

//load instructions
fn ld_nn(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = mem.read_16(regs.read_16("pc") + 1);
    regs.write_16(reg, value);
}

fn ld_a_nn(regs: &mut Registers, mem: &Memory) {
    let value = mem.read_8(regs.read_16("pc") + 1);
    regs.write_8('a', value)
}

fn ld_n(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = mem.read_8(regs.read_16("pc") + 1);
    regs.write_8(reg.chars().next().unwrap(), value);
}

fn ld_r1_r2(regs: &mut Registers, mem: &mut Memory, dest: &str, source: &str) {
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
            let value = regs.read_8(source.chars().next().unwrap());
            regs.write_8(dest.chars().next().unwrap(), value);
        } else {
            //dest is 16 bit register
            let value = regs.read_8(source.chars().next().unwrap());
            if dstmem {
                mem.write_8(regs.read_16(dest), value);
            } else {
                regs.write_16(dest, value as u16);
            }
        }
    } else {
        //source is 16 bit register could be memory address
        let value: u16 = if srcmem {
            mem.read_16(regs.read_16(source))
        } else {
            regs.read_16(source)
        };
        if dest.len() == 1 {
            regs.write_8(dest.chars().next().unwrap(), value as u8);
        } else {
            regs.write_16(dest, value);
        }
    }
}

fn ld_nn_a(regs: &mut Registers, mem: &mut Memory) {
    let value = regs.read_8('a');
    mem.write_8(mem.read_16(regs.read_16("pc") + 1), value);
}

fn ld_m_n(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_8(regs.read_16("pc") + 1);
    mem.write_8(regs.read_16("hl"), value);
}

fn ld_sp_e(regs: &mut Registers, mem: &mut Memory) {
    let sp = mem.read_16(regs.read_16("af"));
    regs.write_16("hl", sp + mem.read_8(regs.read_16("PC") + 1) as u16);
}

fn ld_sp_hl(regs: &mut Registers, mem: &mut Memory) {
    let value = regs.read_16("hl");
    mem.write_16(regs.read_16("sp"), value);
}

fn ldh_n_a(regs: &mut Registers, mem: &mut Memory) {
    let value = regs.read_8('a');
    write_memory_8(
        mem,
        rom,
        0xFF00 + mem.read_8(regs.read_16("pc") + 1) as u16,
        value,
    );
}

fn ldh_a_n(regs: &mut Registers, mem: &mut Memory) {}

fn ldh_c_a(regs: &mut Registers, mem: &mut Memory) {
    let value = regs.read_8('a');
    mem.write_8(0xFF00 + regs.read_8('c') as u16, value);
}

fn ldh_a_c(regs: &mut Registers, mem: &mut Memory) {
    regs.write_8('a', mem.read_8(0xFF00 + (regs.read_8('c') as u16)));
}

fn pop(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = mem.read_16(regs.read_16("sp"));
    regs.write_16(reg, value);
    regs.write_16("sp", regs.clone().read_16("sp") + 2);
}

fn push(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let mut value = regs.read_16(reg);
    //if regs is af then the last 4 bits are 0
    if reg == "af" {
        value = value ^ 0b0000000000001111;
    }
    regs.write_16("sp", regs.clone().read_16("sp") - 2);
    mem.write_16(regs.read_16("sp"), value);
}

// incr and decr
fn inc_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    //if reg have 1 char, it's a 8 bit register
    if reg.len() == 1 {
        let value = regs.read_8(reg.chars().next().unwrap());
        regs.write_8(reg.chars().next().unwrap(), value + 1);
    } else {
        let value = regs.read_16(reg);
        regs.write_16(reg, value + 1);
    }
}

fn dec_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    //if reg have 1 char, it's a 8 bit register
    if reg.len() == 1 {
        let value = regs.read_8(reg.chars().next().unwrap());
        regs.write_8(reg.chars().next().unwrap(), value - 1);
    } else {
        let value = regs.read_16(reg);
        regs.write_16(reg, value - 1);
    }
}

fn inc_m(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = mem.read_8(regs.read_16(reg));
    mem.write_8(regs.read_16(reg), value + 1);
}

fn dec_m(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = mem.read_8(regs.read_16(reg));
    mem.write_8(regs.read_16(reg), value - 1);
}

//rotate and shift
fn rlca(regs: &mut Registers) {
    let value = regs.read_8('a').clone();
    let msb = value & 0x80;
    let new_value = (value << 1) | msb;
    regs.write_8('a', new_value);
    let mut flags = regs.read_8('f');
    //reset N and H flags bits 5 and 6 and Z flag
    flags &= !(HALF_CARRY_FLAG | SUBTRACT_FLAG | ZERO_FLAG);
    //set carry flag if carry from bit 7
    if msb != 0 {
        flags |= CARRY_FLAG;
    }

    regs.write_8('f', flags);
}

fn rla(regs: &mut Registers) {
    let value = regs.read_8('a');
    let msb = value & 0x80;
    let new_value = (value << 1) | ((regs.read_8('f') & CARRY_FLAG) >> 4);
    regs.write_8('a', new_value);
    let mut flags = regs.read_8('f');
    //reset N and H flags bits 5 and 6 and Z flag
    flags &= !(HALF_CARRY_FLAG | SUBTRACT_FLAG | ZERO_FLAG);
    //set carry flag if carry from bit 7
    if msb != 0 {
        flags |= CARRY_FLAG;
    }

    regs.write_8('f', flags);
}
fn rrca(regs: &mut Registers) {
    let value = regs.read_8('a');
    let lsb = value & 0x01;
    let new_value = (value >> 1) | (lsb << 7);
    regs.write_8('a', new_value);
    let mut flags = regs.read_8('f');
    //reset N and H flags bits 5 and 6 and Z flag
    flags &= !(HALF_CARRY_FLAG | SUBTRACT_FLAG | ZERO_FLAG);
    //set carry flag if carry from bit 0
    if lsb != 0 {
        flags |= CARRY_FLAG;
    }

    regs.write_8('f', flags);
}

fn rra(regs: &mut Registers, mem: &mut Memory) {
    let value = regs.read_8('a');
    let lsb = value & 0x01;
    let new_value = (value >> 1) | ((regs.read_8('f') & CARRY_FLAG) << 3);
    regs.write_8('a', new_value);
    let mut flags = regs.read_8('f');
    //reset N and H flags bits 5 and 6 and Z flag
    flags &= !(HALF_CARRY_FLAG | SUBTRACT_FLAG | ZERO_FLAG);
    //set carry flag if carry from bit 0
    if lsb != 0 {
        flags |= CARRY_FLAG;
    }

    regs.write_8('f', flags);
}

//arithmetic and logic
fn add_hl(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = regs.read_16(reg);
    let hl = regs.read_16("hl");
    let result: u32 = value as u32 + hl as u32;
    regs.write_16("hl", result as u16);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn add_a_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = if reg.len() > 1 {
        read_memory_8(
            mem,
            rom,
            regs.read_16(&reg.replace("(", "").replace(")", "")),
        )
    } else {
        regs.read_8(reg.chars().next().unwrap())
    };
    let a = regs.read_8('a');
    let result: u16 = value as u16 + a as u16;
    regs.write_8('a', result as u8);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn add_a_n(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_8(regs.read_16("pc") + 1);
    let a = regs.read_8('a');
    let result: u16 = value as u16 + a as u16;
    regs.write_8('a', result as u8);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn add_sp_e(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_8(regs.read_16("pc") + 1) as i16;
    let sp = regs.read_16("sp");
    let result: i32 = value as i32 + sp as i32;
    regs.write_16("sp", result as u16);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn adc_a_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = if reg.len() > 1 {
        read_memory_8(
            mem,
            rom,
            regs.read_16(&reg.replace("(", "").replace(")", "")),
        )
    } else {
        regs.read_8(reg.chars().next().unwrap())
    };
    let a = regs.read_8('a');
    let carry = (regs.read_8('f') & CARRY_FLAG) >> 4;

    let result: u16 = value as u16 + a as u16 + carry as u16;
    regs.write_8('a', result as u8);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn adc_a_n(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_8(regs.read_16("pc") + 1);
    let a = regs.read_8('a');
    let carry = (regs.read_8('f') & CARRY_FLAG) >> 4;

    let result: u16 = value as u16 + a as u16 + carry as u16;
    regs.write_8('a', result as u8);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn sub_a_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = if reg.len() > 1 {
        read_memory_8(
            mem,
            rom,
            regs.read_16(&reg.replace("(", "").replace(")", "")),
        )
    } else {
        regs.read_8(reg.chars().next().unwrap())
    };
    let a = regs.read_8('a');
    let result: u16 = a as u16 - value as u16;
    regs.write_8('a', result as u8);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn sub_a_n(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_8(regs.read_16("pc") + 1);
    let a = regs.read_8('a');
    let result: u16 = a as u16 - value as u16;
    regs.write_8('a', result as u8);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn sbc_a_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = register_or_memory(regs, mem, &reg);
    let a = regs.read_8('a');
    let carry = (regs.read_8('f') & CARRY_FLAG) >> 4;

    let result: u16 = a as u16 - value as u16 - carry as u16;
    regs.write_8('a', result as u8);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn sbc_a_n(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_8(regs.read_16("pc") + 1);
    let a = regs.read_8('a');
    let carry = (regs.read_8('f') & CARRY_FLAG) >> 4;

    let result: u16 = a as u16 - value as u16 - carry as u16;
    regs.write_8('a', result as u8);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn daa(regs: &mut Registers, mem: &mut Memory) {
    let mut value = regs.read_8('a');
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
    regs.write_8('a', value);
}

fn cpl(regs: &mut Registers) {
    let value = regs.read_8('a');
    regs.write_8('a', !value);
}

fn and_a_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = register_or_memory(regs, mem, &reg);
    let a = regs.read_8('a');
    let result = a & value;
    regs.write_8('a', result);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn and_a_n(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_8(regs.read_16("pc") + 1);
    let a = regs.read_8('a');
    let result = a & value;
    regs.write_8('a', result);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn xor_a_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = register_or_memory(regs, mem, &reg);
    let a = regs.read_8('a');
    let result = a ^ value;
    regs.write_8('a', result);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn xor_a_n(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_8(regs.read_16("pc") + 1);
    let a = regs.read_8('a');
    let result = a ^ value;
    regs.write_8('a', result);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn or_a_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = register_or_memory(regs, mem, &reg);
    let a = regs.read_8('a');
    let result = a | value;
    regs.write_8('a', result);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn or_a_n(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_8(regs.read_16("pc") + 1);
    let a = regs.read_8('a');
    let result = a | value;
    regs.write_8('a', result);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn cp_a_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = register_or_memory(regs, mem, &reg);
    let a = regs.read_8('a');
    let result: u16 = a as u16 - value as u16;
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}

fn cp_a_n(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_8(regs.read_16("pc") + 1);
    let a = regs.read_8('a');
    let result: u16 = a as u16 - value as u16;
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
}
//utils

fn register_or_memory(regs: &mut Registers, mem: &mut Memory, reg: &&str) -> u8 {
    let value = if reg.len() > 1 {
        mem.read_8(regs.read_16(&reg.replace(['(', ')'], "")))
    } else {
        regs.read_8(reg.chars().next().unwrap())
    };
    value
}

//misc
fn nop() {
    //do nothing
}

fn stop() {
    //stop cpu until button pressed
}

fn scf(regs: &mut Registers) {
    let mut flags = regs.read_8('f');
    flags |= CARRY_FLAG;
    flags &= !HALF_CARRY_FLAG;
    flags &= !SUBTRACT_FLAG;
    regs.write_8('f', flags);
}

fn ccf(regs: &mut Registers) {
    let mut flags = regs.read_8('f');
    flags ^= CARRY_FLAG;
    flags &= !HALF_CARRY_FLAG;
    flags &= !SUBTRACT_FLAG;
    regs.write_8('f', flags);
}

//cb instructions
fn write_mem_or_regs(regs: &mut Registers, mem: &mut Memory, reg: &&str, result: u8) {
    if reg.len() > 1 {
        mem.write_8(regs.read_16(&reg.replace(['(', ')'], "")), result);
    } else {
        regs.write_8(reg.chars().next().unwrap(), result);
    }
}

fn rlc_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = register_or_memory(regs, mem, &reg);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
    write_mem_or_regs(regs, mem, &reg, result);
}

fn rrc_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = register_or_memory(regs, mem, &reg);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
    write_mem_or_regs(regs, mem, &reg, result);
}

fn rl_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = register_or_memory(regs, mem, &reg);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
    write_mem_or_regs(regs, mem, &reg, result);
}

fn rr_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = register_or_memory(regs, mem, &reg);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
    write_mem_or_regs(regs, mem, &reg, result);
}

fn sla_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = register_or_memory(regs, mem, &reg) as i8;
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
    write_mem_or_regs(regs, mem, &reg, result as u8);
}

fn sra_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = register_or_memory(regs, mem, &reg) as i8;
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
    write_mem_or_regs(regs, mem, &reg, result as u8);
}

fn swap_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = register_or_memory(regs, mem, &reg);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
    write_mem_or_regs(regs, mem, &reg, result);
}

fn bit_n_r(regs: &mut Registers, mem: &mut Memory, reg: &str, n: u8) {
    let value = register_or_memory(regs, mem, &reg);
    let mut flags = regs.read_8('f');
    //set H
    flags |= HALF_CARRY_FLAG;
    //reset N
    flags &= !SUBTRACT_FLAG;
    //set Z flag if result is 0
    flags &= !ZERO_FLAG;
    if value & (1 << n) == 0 {
        flags |= ZERO_FLAG;
    }
    regs.write_8('f', flags);
}

fn srl_r(regs: &mut Registers, mem: &mut Memory, reg: &str) {
    let value = register_or_memory(regs, mem, &reg);
    let mut flags = regs.read_8('f');
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
    regs.write_8('f', flags);
    write_mem_or_regs(regs, mem, &reg, result);
}

fn res_n_r(regs: &mut Registers, mem: &mut Memory, reg: &str, n: u8) {
    let value = register_or_memory(regs, mem, &reg);
    let result = value & !(1 << n);
    write_mem_or_regs(regs, mem, &reg, result);
}

fn set_n_r(regs: &mut Registers, mem: &mut Memory, reg: &str, n: u8) {
    let value = register_or_memory(regs, mem, &reg);
    let result = value | (1 << n);
    write_mem_or_regs(regs, mem, &reg, result);
}

fn call_CB(regs: &mut Registers, mem: &mut Memory) {
    //TODO change name
    const REG_NAMES: [&str; 8] = ["b", "c", "d", "e", "h", "l", "(hl)", "a"];
    //TODO implement CB instructions
    let cb_opcode = mem.read_8(regs.read_16("pc") + 1);
    match cb_opcode {
        0x00..0x07 => rlc_r(regs, mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
        0x08..0x0F => rrc_r(regs, mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
        0x10..0x17 => rl_r(regs, mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
        0x18..0x1F => rr_r(regs, mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
        0x20..0x27 => sla_r(regs, mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
        0x28..0x2F => sra_r(regs, mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
        0x30..0x37 => swap_r(regs, mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
        0x38..0x3F => srl_r(regs, mem, REG_NAMES[(cb_opcode & 0x0f) as usize]),
        0x40..0x7F => bit_n_r(
            regs,
            mem,
            REG_NAMES[(cb_opcode & 0x0f) as usize],
            (cb_opcode >> 3) & 0x07,
        ),
        0x80..0xBF => res_n_r(
            regs,
            mem,
            REG_NAMES[(cb_opcode & 0x0f) as usize],
            (cb_opcode >> 4) & 0x07,
        ),
        0xC0..0xFF => set_n_r(
            regs,
            mem,
            REG_NAMES[(cb_opcode & 0x0f) as usize],
            (cb_opcode >> 5) & 0x07,
        ),
        _ => panic!("Unknown CB opcode: {:x}", cb_opcode),
    }
}

fn di(regs: &mut Registers) {
    regs.ime = 0;
}

fn ei(regs: &mut Registers) {
    regs.ime = 1;
}

//flow
fn jr_e(regs: &mut Registers, mem: &mut Memory) {
    //temp register
    let tmp = regs.read_16("pc").clone() as i16;
    //read next byte
    let value = mem.read_8(tmp.clone() as u16 + 1) as i8;
    regs.write_16("pc", (tmp + value as i16) as u16);
}

fn jr_f_e(regs: &mut Registers, mem: &mut Memory, cflag: char, z: bool) {
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
    let tmp = regs.read_16("pc") as i16;
    //read next byte
    let value = mem.read_8(tmp as u16 + 1) as i8;
    let cond = if z { 1 } else { 0 };
    if (regs.read_8('f') & flag) >> shift == cond {
        regs.write_16("pc", (tmp + value as i16) as u16);
    }
}

fn jp_nn(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_16(regs.read_16("pc") + 1) - 3;
    //correct for the 2 bytes read
    regs.write_16("pc", value);
}

fn jp_f_nn(regs: &mut Registers, mem: &mut Memory, cflag: char, condition: bool) {
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

    if (regs.read_8('f') & flag) >> shift == cond {
        let value = mem.read_16(regs.clone().read_16("pc") + 1) - 3;
        //correct for the 2 bytes read
        regs.write_16("pc", value);
    }
}

fn jp_hl(regs: &mut Registers) {
    regs.write_16("pc", (regs).read_16("hl"));
}

fn call_nn(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_16(regs.read_16("pc") + 1) - 3;
    //correct for the 2 bytes read
    regs.write_16("sp", regs.clone().read_16("sp") - 2);
    mem.write_16(regs.clone().read_16("sp"), regs.clone().read_16("pc"));
    regs.write_16("pc", value);
}

fn call_f_nn(regs: &mut Registers, mem: &mut Memory, cflag: char, z: bool) {
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

    if (regs.read_8('f') & flag) >> shift == cond {
        let value = mem.read_16(regs.clone().read_16("pc") + 1) - 3;
        //correct for the 2 bytes read
        regs.write_16("sp", regs.clone().read_16("sp") - 2);
        mem.write_16(regs.clone().read_16("sp"), regs.clone().read_16("pc") + 3);
        regs.write_16("pc", value);
    }
}

fn rst(regs: &mut Registers, mem: &mut Memory, value: u16) {
    regs.write_16("sp", regs.clone().read_16("sp") - 2);
    mem.write_16(regs.clone().read_16("sp"), regs.clone().read_16("pc") + 1);
    regs.write_16("pc", value - 1);
}

fn ret(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_16(regs.clone().read_16("sp"));
    regs.write_16("sp", regs.clone().read_16("sp") + 2);
    regs.write_16("pc", value - 1);
}

fn ret_f(regs: &mut Registers, mem: &mut Memory, cflag: char, z: bool) {
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

    if (regs.read_8('f') & flag) >> shift == cond {
        let value = mem.read_16(regs.clone().read_16("sp"));
        regs.write_16("sp", regs.clone().read_16("sp") + 2);
        regs.write_16("pc", value - 1);
    }
}

fn reti(regs: &mut Registers, mem: &mut Memory) {
    let value = mem.read_16(regs.clone().read_16("sp"));
    regs.write_16("sp", regs.clone().read_16("sp") + 2);
    regs.write_16("pc", value - 1);
    regs.write_8('i', 1);
}

//end of cpu
fn execute(opcode: u8, regs: &mut Registers, mem: &mut Memory) {
    const REG_NAMES: [&str; 8] = ["b", "c", "d", "e", "h", "l", "(hl)", "a"];
    match opcode {
        0x00 => nop(),
        0x01 => ld_nn(regs, mem, "bc"),
        0x02 => ld_r1_r2(regs, mem, "(bc)", "a"), //load (bc) into a, bc is the memory address
        0x03 => inc_r(regs, mem, "bc"),
        0x04 => inc_r(regs, mem, "b"),
        0x05 => dec_r(regs, mem, "b"),
        0x06 => ld_n(regs, mem, "b"),
        0x07 => rlca(regs),
        0x08 => ld_nn(regs, mem, "sp"),
        0x09 => add_hl(regs, mem, "bc"),
        0x0A => ld_r1_r2(regs, mem, "a", "(bc)"), //load a into (bc), bc is the memory address
        0x0B => dec_r(regs, mem, "bc"),
        0x0C => inc_r(regs, mem, "c"),
        0x0D => dec_r(regs, mem, "c"),
        0x0E => ld_n(regs, mem, "c"),
        0x0F => rrca(regs),
        0x10 => stop(),
        0x11 => ld_nn(regs, mem, "de"),
        0x12 => ld_r1_r2(regs, mem, "(de)", "a"), //load (de) into a, de is the memory address
        0x13 => inc_r(regs, mem, "de"),
        0x14 => inc_r(regs, mem, "d"),
        0x15 => dec_r(regs, mem, "d"),
        0x16 => ld_n(regs, mem, "d"),
        0x17 => rla(regs),
        0x18 => jr_e(regs, mem),
        0x19 => add_hl(regs, mem, "de"),
        0x1A => ld_r1_r2(regs, mem, "a", "(de)"), //load a into (de), de is the memory address
        0x1B => dec_r(regs, mem, "de"),
        0x1C => inc_r(regs, mem, "e"),
        0x1D => dec_r(regs, mem, "e"),
        0x1E => ld_n(regs, mem, "e"),
        0x1F => rra(regs, mem),
        0x20 => jr_f_e(regs, mem, 'z', false),
        0x21 => ld_nn(regs, mem, "hl"),
        0x22 => {
            ld_r1_r2(regs, mem, "(hl)", "a");
            inc_r(regs, mem, "hl");
        } //load a into (hl), hl is the memory address, then increment hl
        0x23 => inc_r(regs, mem, "hl"),
        0x24 => inc_r(regs, mem, "h"),
        0x25 => dec_r(regs, mem, "h"),
        0x26 => ld_n(regs, mem, "h"),
        0x27 => daa(regs, mem),
        0x28 => jr_f_e(regs, mem, 'z', true),
        0x29 => add_hl(regs, mem, "hl"),
        0x2A => {
            ld_r1_r2(regs, mem, "a", "(hl)");
            inc_r(regs, mem, "hl");
        } //load (hl) into a, hl is the memory address, then increment hl
        0x2B => dec_r(regs, mem, "hl"),
        0x2C => inc_r(regs, mem, "l"),
        0x2D => dec_r(regs, mem, "l"),
        0x2E => ld_n(regs, mem, "l"),
        0x2F => cpl(regs),
        0x30 => jr_f_e(regs, mem, 'c', false),
        0x31 => ld_nn(regs, mem, "sp"),
        0x32 => {
            ld_r1_r2(regs, mem, "(hl)", "a");
            dec_r(regs, mem, "hl");
        } //load a into (hl), hl is the memory address, then decrement hl
        0x33 => inc_r(regs, mem, "sp"),
        0x34 => inc_m(regs, mem, "hl"),
        0x35 => dec_m(regs, mem, "hl"),
        0x36 => ld_m_n(regs, mem),
        0x37 => scf(regs),
        0x38 => jr_f_e(regs, mem, 'c', true),
        0x39 => add_hl(regs, mem, "sp"),
        0x3A => {
            ld_r1_r2(regs, mem, "a", "(hl)");
            dec_r(regs, mem, "hl");
        } //load (hl) into a, hl is the memory address, then decrement hl
        0x3B => dec_r(regs, mem, "sp"),
        0x3C => inc_r(regs, mem, "a"),
        0x3D => dec_r(regs, mem, "a"),
        0x3E => ld_n(regs, mem, "a"),
        0x3F => ccf(regs),
        0x40..0x7F => ld_r1_r2(
            regs,
            mem,
            REG_NAMES[((opcode >> 3) & 0b111) as usize],
            REG_NAMES[(opcode & 0x0f) as usize],
        ),
        0x80..0x87 => add_a_r(regs, mem, REG_NAMES[(opcode & 0x0f) as usize]),
        0x88..0x8F => adc_a_r(regs, mem, REG_NAMES[(opcode & 0x0f) as usize]),
        0x90..0x97 => sub_a_r(regs, mem, REG_NAMES[(opcode & 0x0f) as usize]),
        0x98..0x9F => sbc_a_r(regs, mem, REG_NAMES[(opcode & 0x0f) as usize]),
        0xA0..0xA7 => and_a_r(regs, mem, REG_NAMES[(opcode & 0x0f) as usize]),
        0xA8..0xAF => xor_a_r(regs, mem, REG_NAMES[(opcode & 0x0f) as usize]),
        0xB0..0xB7 => or_a_r(regs, mem, REG_NAMES[(opcode & 0x0f) as usize]),
        0xB8..0xBF => cp_a_r(regs, mem, REG_NAMES[(opcode & 0x0f) as usize]),
        0xC0 => ret_f(regs, mem, 'z', false), //return if z flag is false
        0xC1 => pop(regs, mem, "bc"),
        0xC2 => jp_f_nn(regs, mem, 'z', false),
        0xC3 => jp_nn(regs, mem),
        0xC4 => call_f_nn(regs, mem, 'z', false),
        0xC5 => push(regs, mem, "bc"),
        0xC6 => add_a_n(regs, mem),
        0xC7 => rst(regs, mem, 0x00),
        0xC8 => ret_f(regs, mem, 'z', true), //return if z flag is true
        0xC9 => ret(regs, mem),
        0xCA => jp_f_nn(regs, mem, 'z', true),
        0xCB => call_CB(regs, mem),
        0xCC => call_f_nn(regs, mem, 'z', true),
        0xCD => call_nn(regs, mem),
        0xCE => adc_a_n(regs, mem),
        0xCF => rst(regs, mem, 0x08),
        0xD0 => ret_f(regs, mem, 'c', false), //return if c flag is false
        0xD1 => pop(regs, mem, "de"),
        0xD2 => jp_f_nn(regs, mem, 'c', false),
        0xD3 => unimplemented!(),
        0xD4 => call_f_nn(regs, mem, 'c', false),
        0xD5 => push(regs, mem, "de"),
        0xD6 => sub_a_n(regs, mem),
        0xD7 => rst(regs, mem, 0x10),
        0xD8 => ret_f(regs, mem, 'c', true), //return if c flag is true
        0xD9 => reti(regs, mem),
        0xDA => jp_f_nn(regs, mem, 'c', true),
        0xDB => unimplemented!(),
        0xDC => call_f_nn(regs, mem, 'c', true),
        0xDD => unimplemented!(),
        0xDE => sbc_a_n(regs, mem),
        0xDF => rst(regs, mem, 0x18),
        0xE0 => ldh_n_a(regs, mem),
        0xE1 => pop(regs, mem, "hl"),
        0xE2 => ldh_c_a(regs, mem),
        0xE3..0xE4 => unimplemented!(),
        0xE5 => push(regs, mem, "hl"),
        0xE6 => and_a_n(regs, mem),
        0xE7 => rst(regs, mem, 0x20),
        0xE8 => add_sp_e(regs, mem),
        0xE9 => jp_hl(regs),
        0xEA => ld_nn_a(regs, mem),
        0xEB..0xED => unimplemented!(),
        0xEE => xor_a_n(regs, mem),
        0xEF => rst(regs, mem, 0x28),
        0xF0 => ldh_a_n(regs, mem),
        0xF1 => pop(regs, mem, "af"),
        0xF2 => ldh_a_c(regs, mem),
        0xF3 => di(regs),
        0xF4 => unimplemented!(),
        0xF5 => push(regs, mem, "af"),
        0xF6 => or_a_n(regs, mem),
        0xF7 => rst(regs, mem, 0x30),
        0xF8 => ld_sp_e(regs, mem),
        0xF9 => ld_sp_hl(regs, mem),
        0xFA => ld_a_nn(regs, mem),
        0xFB => ei(regs),
        0xFC..0xFD => unimplemented!(),
        0xFE => cp_a_n(regs, mem),
        0xFF => rst(regs, mem, 0x38),
        _ => println!("Opcode not implemented: {:2X}", opcode),
    }
}

fn handle_post_instruction(regs: &mut Registers, mem: &mut Memory, opcode: u8, mut lenght: u64) {
    //increment pc
    let mut delta = 0;
    let pc = regs.read_16("pc").clone();
    regs.write_16("pc", pc + OPCODE_LENGHTS[opcode.clone() as usize] as u16);
    lenght += OPCODE_LENGHTS[opcode.clone() as usize] as u64;
    delta += OPCODE_LENGHTS[opcode as usize] as u64;
    if opcode == 0xCB {
        let pc = regs.read_16("pc").clone();
        let opcode = mem.read_8(pc);
        regs.write_16("pc", pc + OPCODE_LENGHTS_CB[opcode.clone() as usize] as u16);
        lenght += OPCODE_LENGHTS_CB[opcode.clone() as usize] as u64;
        delta += OPCODE_LENGHTS_CB[opcode as usize] as u64;
    }

    //handle interrupts
    //draw delta pixels if vblank
    //handle stuff
}

fn main() {
    println!("Hello, world!");
    let mut header = CartridgeHeader {
        title: [' '; 16],
        logo: [' '; 48],
        c_type: 0,
        rom_size: 0,
        ram_size: 0,
        destination: 0,
        old_licensee: 0,
        mask_rom_version: 0,
        header_checksum: 0,
        global_checksum: [0; 2],
    };
    let mut regs = Registers {
        af: 0,
        bc: 0,
        de: 0,
        hl: 0,
        sp: 0,
        pc: 0,
        ime: 0,
    };
    regs.af = 0x01B0;
    regs.write_16("bc", 0x0112);
    println!("af : {:X}", regs.read_16("bc"));
    //load Rom.gb to Rom buffer

    let mut mem: Memory = Memory::new();
    //;load Rom to Rom buffer
    let mut file = File::open("rom.gb").unwrap();
    #[allow(clippy::unused_io_amount)]
    file.read(&mut mem.rom.buffer).unwrap();
    //load Rom header
    for i in 0..16 {
        header.title[i] = mem.rom.buffer[i + 0x134] as char;
    }
    for i in 0..48 {
        header.logo[i] = mem.rom.buffer[0x100 + i] as char;
    }
    header.c_type = mem.rom.buffer[0x147];
    header.rom_size = mem.rom.buffer[0x148];
    header.ram_size = mem.rom.buffer[0x149];
    header.destination = mem.rom.buffer[0x14A];
    header.old_licensee = mem.rom.buffer[0x14B];
    header.mask_rom_version = mem.rom.buffer[0x14C];
    header.header_checksum = mem.rom.buffer[0x14D];
    header.global_checksum[0] = mem.rom.buffer[0x14E];
    header.global_checksum[1] = mem.rom.buffer[0x14F];
    println!("title : {}", header.title.iter().collect::<String>());
    println!("logo : {}", header.logo.iter().collect::<String>());
    println!("c_type : {:X}", header.c_type);
    println!("rom_size : {:X}", header.rom_size);
    println!("ram_size : {:X}", header.ram_size);
    println!("destination : {:X}", header.destination);
    println!("old_licensee : {:X}", header.old_licensee);
    println!("mask_rom_version : {:X}", header.mask_rom_version);
    println!("header_checksum : {:X}", header.header_checksum);
    println!("global_checksum : {:X}", header.global_checksum[0]);
    println!("global_checksum : {:X}", header.global_checksum[1]);

    unsafe {
        mem.rom.bank = mem.rom.buffer.as_mut_ptr().offset(0x4000);
    }

    for i in 0..0xFFFF {
        mem.main_memory[i] = 0;
    }
    mem.write_16(0x9000, 0x1234);
    println!("mem[0x9000] : {:X}", mem.read_16(0x9000));
}
