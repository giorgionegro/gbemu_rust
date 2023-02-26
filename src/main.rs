#![feature(exclusive_range_pattern)]

use std::fs::File;
use std::io::Read;
use std::mem;
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

type Memory = [u8; 0xFFFF];

type RawBankNumber = u8;
const BANK_MASK: u8 = 0b00011111;
type Bank = [u8; 0x4000];

#[derive(Clone, Copy)]
struct Rom {
    buffer: [u8; 0x2FFFF],
    bank: *mut u8,
}
const RBN: u16 = 0x2000;
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
    pub fn read_register_8(&mut self, register: char) -> u8 {
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

    pub fn read_register_16(&mut self, register: &str) -> u16 {
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

    pub fn write_register_8(&mut self, register: char, value: u8) {
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

    fn write_register_16(&mut self, register: &str, value: u16) {
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
fn read_memory_8(mem: &Memory, rom: &Rom, address: u16) -> u8 {
    if (0x4000..0x8000).contains(&address) {
        unsafe {
            let x = *(rom.bank.offset((address - 0x4000) as isize));
            return x;
        }
    }
    mem[address as usize]
}

fn read_memory_16(mem: &Memory, rom: &Rom, address: u16) -> u16 {
    let x = read_memory_8(mem, rom, address);
    let y = read_memory_8(mem, rom, address + 1);
    (y as u16) << 8 | x as u16
}

fn write_to_rom_register(rom: &mut Rom, address: u16, value: u8) {
    if address > RBN && address < 2 * RBN {
        let bank_number: RawBankNumber = value & BANK_MASK;
        let bank_adress: u32;
        if bank_number == 0 {
            return;
        } else if bank_number == 1 {
            bank_adress = 0x4000;
        } else {
            bank_adress = 0x4000 * bank_number as u32;
        }
        unsafe {
            rom.bank = rom.buffer.as_mut_ptr().offset(bank_adress as isize);
        }
    }
}

fn write_memory_8(mem: &mut Memory, rom: &mut Rom, address: u16, value: u8) {
    if address < 0x8000 {
        write_to_rom_register(rom, address, value);
    } else {
        mem[address as usize] = value;
    }
}
fn write_memory_16(mem: &mut Memory, rom: &mut Rom, address: u16, value: u16) {
    write_memory_8(mem, rom, address, (value >> 8) as u8);
    write_memory_8(mem, rom, address + 1, (value & 0xFF) as u8);
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
fn ld_nn(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    let value = read_memory_16(mem, rom, regs.read_register_16("pc") + 1);
    regs.write_register_16(reg, value);
}

fn ld_n(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    let value = read_memory_8(mem, rom, regs.read_register_16("pc") + 1);
    regs.write_register_8(reg.chars().next().unwrap(), value);
}

fn ld_r1_r2(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, dest: &str, source: &str) {
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
            let value = regs.read_register_8(source.chars().next().unwrap());
            regs.write_register_8(dest.chars().next().unwrap(), value);
        } else {
            //dest is 16 bit register
            let value = regs.read_register_8(source.chars().next().unwrap());
            if dstmem {
                write_memory_8(mem, rom, regs.read_register_16(dest), value);
            } else {
                regs.write_register_16(dest, value as u16);
            }
        }
    } else {
        //source is 16 bit register could be memory address
        let value: u16 = if srcmem {
            read_memory_16(mem, rom, regs.read_register_16(source))
        } else {
            regs.read_register_16(source)
        };
        if dest.len() == 1 {
            regs.write_register_8(dest.chars().next().unwrap(), value as u8);
        } else {
            regs.write_register_16(dest, value);
        }
    }
}

fn ld_m_n(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    let value = read_memory_8(mem, rom, regs.read_register_16("pc") + 1);
    write_memory_8(mem, rom, regs.read_register_16("hl"), value);
}

fn pop(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    let value = read_memory_16(mem, rom, regs.read_register_16("sp"));
    regs.write_register_16(reg, value);
    regs.write_register_16("sp", regs.clone().read_register_16("sp") + 2);
}

fn push(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    let value = regs.read_register_16(reg);
    regs.write_register_16("sp", regs.clone().read_register_16("sp") - 2);
    write_memory_16(mem, rom, regs.read_register_16("sp"), value);
}

// incr and decr
fn inc_r(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    //if reg have 1 char, it's a 8 bit register
    if reg.len() == 1 {
        let value = regs.read_register_8(reg.chars().next().unwrap());
        regs.write_register_8(reg.chars().next().unwrap(), value + 1);
    } else {
        let value = regs.read_register_16(reg);
        regs.write_register_16(reg, value + 1);
    }
}

fn dec_r(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    //if reg have 1 char, it's a 8 bit register
    if reg.len() == 1 {
        let value = regs.read_register_8(reg.chars().next().unwrap());
        regs.write_register_8(reg.chars().next().unwrap(), value - 1);
    } else {
        let value = regs.read_register_16(reg);
        regs.write_register_16(reg, value - 1);
    }
}

fn inc_m(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    let value = read_memory_8(mem, rom, regs.read_register_16(reg));
    write_memory_8(mem, rom, regs.read_register_16(reg), value + 1);
}

fn dec_m(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    let value = read_memory_8(mem, rom, regs.read_register_16(reg));
    write_memory_8(mem, rom, regs.read_register_16(reg), value - 1);
}

//rotate and shift
fn rlca(regs: &mut Registers) {
    let value = regs.read_register_8('a').clone();
    let msb = value & 0x80;
    let new_value = (value << 1) | msb;
    regs.write_register_8('a', new_value);
    let mut flags = regs.read_register_8('f');
    //reset N and H flags bits 5 and 6 and Z flag
    flags &= !(HALF_CARRY_FLAG | SUBTRACT_FLAG | ZERO_FLAG);
    //set carry flag if carry from bit 7
    if msb != 0 {
        flags |= CARRY_FLAG;
    }

    regs.write_register_8('f', flags);
}

fn rla(regs: &mut Registers) {
    let value = regs.read_register_8('a');
    let msb = value & 0x80;
    let new_value = (value << 1) | ((regs.read_register_8('f') & CARRY_FLAG) >> 4);
    regs.write_register_8('a', new_value);
    let mut flags = regs.read_register_8('f');
    //reset N and H flags bits 5 and 6 and Z flag
    flags &= !(HALF_CARRY_FLAG | SUBTRACT_FLAG | ZERO_FLAG);
    //set carry flag if carry from bit 7
    if msb != 0 {
        flags |= CARRY_FLAG;
    }

    regs.write_register_8('f', flags);
}
fn rrca(regs: &mut Registers) {
    let value = regs.read_register_8('a');
    let lsb = value & 0x01;
    let new_value = (value >> 1) | (lsb << 7);
    regs.write_register_8('a', new_value);
    let mut flags = regs.read_register_8('f');
    //reset N and H flags bits 5 and 6 and Z flag
    flags &= !(HALF_CARRY_FLAG | SUBTRACT_FLAG | ZERO_FLAG);
    //set carry flag if carry from bit 0
    if lsb != 0 {
        flags |= CARRY_FLAG;
    }

    regs.write_register_8('f', flags);
}
fn rra(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    let value = regs.read_register_8('a');
    let lsb = value & 0x01;
    let new_value = (value >> 1) | ((regs.read_register_8('f') & CARRY_FLAG) << 3);
    regs.write_register_8('a', new_value);
    let mut flags = regs.read_register_8('f');
    //reset N and H flags bits 5 and 6 and Z flag
    flags &= !(HALF_CARRY_FLAG | SUBTRACT_FLAG | ZERO_FLAG);
    //set carry flag if carry from bit 0
    if lsb != 0 {
        flags |= CARRY_FLAG;
    }

    regs.write_register_8('f', flags);
}

//arithmetic and logic
fn add_hl(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    let value = regs.read_register_16(reg);
    let hl = regs.read_register_16("hl");
    let result: u32 = value as u32 + hl as u32;
    regs.write_register_16("hl", result as u16);
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

fn add_a_r(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = if reg.len() > 1 {
        read_memory_8(
            mem,
            rom,
            regs.read_register_16(&reg.replace("(", "").replace(")", "")),
        )
    } else {
        regs.read_register_8(reg.chars().next().unwrap())
    };
    let a = regs.read_register_8('a');
    let result: u16 = value as u16 + a as u16;
    regs.write_register_8('a', result as u8);
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

fn add_a_n(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    let value = read_memory_8(mem, rom, regs.read_register_16("pc") + 1);
    let a = regs.read_register_8('a');
    let result: u16 = value as u16 + a as u16;
    regs.write_register_8('a', result as u8);
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

fn adc_a_r(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = if reg.len() > 1 {
        read_memory_8(
            mem,
            rom,
            regs.read_register_16(&reg.replace("(", "").replace(")", "")),
        )
    } else {
        regs.read_register_8(reg.chars().next().unwrap())
    };
    let a = regs.read_register_8('a');
    let carry = (regs.read_register_8('f') & CARRY_FLAG) >> 4;

    let result: u16 = value as u16 + a as u16 + carry as u16;
    regs.write_register_8('a', result as u8);
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

fn adc_a_n(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    let value = read_memory_8(mem, rom, regs.read_register_16("pc") + 1);
    let a = regs.read_register_8('a');
    let carry = (regs.read_register_8('f') & CARRY_FLAG) >> 4;

    let result: u16 = value as u16 + a as u16 + carry as u16;
    regs.write_register_8('a', result as u8);
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

fn sub_a_r(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = if reg.len() > 1 {
        read_memory_8(
            mem,
            rom,
            regs.read_register_16(&reg.replace("(", "").replace(")", "")),
        )
    } else {
        regs.read_register_8(reg.chars().next().unwrap())
    };
    let a = regs.read_register_8('a');
    let result: u16 = a as u16 - value as u16;
    regs.write_register_8('a', result as u8);
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

fn sub_a_n(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    let value = read_memory_8(mem, rom, regs.read_register_16("pc") + 1);
    let a = regs.read_register_8('a');
    let result: u16 = a as u16 - value as u16;
    regs.write_register_8('a', result as u8);
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

fn sbc_a_r(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = register_or_memory(regs, mem, rom, &reg);
    let a = regs.read_register_8('a');
    let carry = (regs.read_register_8('f') & CARRY_FLAG) >> 4;

    let result: u16 = a as u16 - value as u16 - carry as u16;
    regs.write_register_8('a', result as u8);
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

fn sbc_a_n(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    let value = read_memory_8(mem, rom, regs.read_register_16("pc") + 1);
    let a = regs.read_register_8('a');
    let carry = (regs.read_register_8('f') & CARRY_FLAG) >> 4;

    let result: u16 = a as u16 - value as u16 - carry as u16;
    regs.write_register_8('a', result as u8);
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

fn daa(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    let mut value = regs.read_register_8('a');
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
    regs.write_register_8('a', value);
}

fn cpl(regs: &mut Registers) {
    let value = regs.read_register_8('a');
    regs.write_register_8('a', !value);
}

fn and_a_r(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = register_or_memory(regs, mem, rom, &reg);
    let a = regs.read_register_8('a');
    let result = a & value;
    regs.write_register_8('a', result);
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

fn xor_a_r(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = register_or_memory(regs, mem, rom, &reg);
    let a = regs.read_register_8('a');
    let result = a ^ value;
    regs.write_register_8('a', result);
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

fn or_a_r(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = register_or_memory(regs, mem, rom, &reg);
    let a = regs.read_register_8('a');
    let result = a | value;
    regs.write_register_8('a', result);
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

fn cp_a_r(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &str) {
    //if reg len is greater than 1, it is a 16 bit register pointing to memory in particular hl
    let value = register_or_memory(regs, mem, rom, &reg);
    let a = regs.read_register_8('a');
    let result: u16 = a as u16 - value as u16;
    let mut flags = regs.read_register_8('f');
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
    regs.write_register_8('f', flags);
}

//utils

fn register_or_memory(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, reg: &&str) -> u8 {
    let value = if reg.len() > 1 {
        read_memory_8(
            mem,
            rom,
            regs.read_register_16(&reg.replace(['(', ')'], "")),
        )
    } else {
        regs.read_register_8(reg.chars().next().unwrap())
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
    let mut flags = regs.read_register_8('f');
    flags |= CARRY_FLAG;
    flags &= !HALF_CARRY_FLAG;
    flags &= !SUBTRACT_FLAG;
    regs.write_register_8('f', flags);
}

fn ccf(regs: &mut Registers) {
    let mut flags = regs.read_register_8('f');
    flags ^= CARRY_FLAG;
    flags &= !HALF_CARRY_FLAG;
    flags &= !SUBTRACT_FLAG;
    regs.write_register_8('f', flags);
}

fn call_CB(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) { //TODO change name
                                                                    //TODO implement CB instructions
}

//flow
fn jr_e(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    //temp register
    let tmp = regs.read_register_16("pc").clone() as i16;
    //read next byte
    let value = read_memory_8(mem, rom, tmp.clone() as u16 + 1) as i8;
    regs.write_register_16("pc", (tmp + value as i16) as u16);
}

fn jr_f_e(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, cflag: char, z: bool) {
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
    let tmp = regs.read_register_16("pc") as i16;
    //read next byte
    let value = read_memory_8(mem, rom, tmp as u16 + 1) as i8;
    let cond = if z { 1 } else { 0 };
    if (regs.read_register_8('f') & flag) >> shift == cond {
        regs.write_register_16("pc", (tmp + value as i16) as u16);
    }
}

fn jp_nn(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    let value = read_memory_16(mem, rom, regs.read_register_16("pc") + 1) - 3;
    //correct for the 2 bytes read
    regs.write_register_16("pc", value);
}

fn jp_f_nn(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, cflag: char, condition: bool) {
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

    if (regs.read_register_8('f') & flag) >> shift == cond {
        let value = read_memory_16(mem, rom, regs.clone().read_register_16("pc") + 1) - 3;
        //correct for the 2 bytes read
        regs.write_register_16("pc", value);
    }
}

fn call_nn(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    let value = read_memory_16(mem, rom, regs.read_register_16("pc") + 1) - 3;
    //correct for the 2 bytes read
    regs.write_register_16("sp", regs.clone().read_register_16("sp") - 2);
    write_memory_16(
        mem,
        rom,
        regs.clone().read_register_16("sp"),
        regs.clone().read_register_16("pc"),
    );
    regs.write_register_16("pc", value);
}

fn call_f_nn(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, cflag: char, z: bool) {
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

    if (regs.read_register_8('f') & flag) >> shift == cond {
        let value = read_memory_16(mem, rom, regs.clone().read_register_16("pc") + 1) - 3;
        //correct for the 2 bytes read
        regs.write_register_16("sp", regs.clone().read_register_16("sp") - 2);
        write_memory_16(
            mem,
            rom,
            regs.clone().read_register_16("sp"),
            regs.clone().read_register_16("pc") + 3,
        );
        regs.write_register_16("pc", value);
    }
}

fn rst(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, value: u16) {
    regs.write_register_16("sp", regs.clone().read_register_16("sp") - 2);
    write_memory_16(
        mem,
        rom,
        regs.clone().read_register_16("sp"),
        regs.clone().read_register_16("pc") + 1,
    );
    regs.write_register_16("pc", value - 1);
}

fn ret(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    let value = read_memory_16(mem, rom, regs.clone().read_register_16("sp"));
    regs.write_register_16("sp", regs.clone().read_register_16("sp") + 2);
    regs.write_register_16("pc", value - 1);
}

fn ret_f(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom, cflag: char, z: bool) {
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

    if (regs.read_register_8('f') & flag) >> shift == cond {
        let value = read_memory_16(mem, rom, regs.clone().read_register_16("sp"));
        regs.write_register_16("sp", regs.clone().read_register_16("sp") + 2);
        regs.write_register_16("pc", value - 1);
    }
}

fn reti(regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    let value = read_memory_16(mem, rom, regs.clone().read_register_16("sp"));
    regs.write_register_16("sp", regs.clone().read_register_16("sp") + 2);
    regs.write_register_16("pc", value - 1);
    regs.write_register_8('i', 1);
}

//end of cpu
fn execute(opcode: u8, regs: &mut Registers, mem: &mut Memory, rom: &mut Rom) {
    const REG_NAMES: [&str; 8] = ["b", "c", "d", "e", "h", "l", "(hl)", "a"];
    match opcode {
        0x00 => nop(),
        0x01 => ld_nn(regs, mem, rom, "bc"),
        0x02 => ld_r1_r2(regs, mem, rom, "(bc)", "a"), //load (bc) into a, bc is the memory address
        0x03 => inc_r(regs, mem, rom, "bc"),
        0x04 => inc_r(regs, mem, rom, "b"),
        0x05 => dec_r(regs, mem, rom, "b"),
        0x06 => ld_n(regs, mem, rom, "b"),
        0x07 => rlca(regs),
        0x08 => ld_nn(regs, mem, rom, "sp"),
        0x09 => add_hl(regs, mem, rom, "bc"),
        0x0A => ld_r1_r2(regs, mem, rom, "a", "(bc)"), //load a into (bc), bc is the memory address
        0x0B => dec_r(regs, mem, rom, "bc"),
        0x0C => inc_r(regs, mem, rom, "c"),
        0x0D => dec_r(regs, mem, rom, "c"),
        0x0E => ld_n(regs, mem, rom, "c"),
        0x0F => rrca(regs),
        0x10 => stop(),
        0x11 => ld_nn(regs, mem, rom, "de"),
        0x12 => ld_r1_r2(regs, mem, rom, "(de)", "a"), //load (de) into a, de is the memory address
        0x13 => inc_r(regs, mem, rom, "de"),
        0x14 => inc_r(regs, mem, rom, "d"),
        0x15 => dec_r(regs, mem, rom, "d"),
        0x16 => ld_n(regs, mem, rom, "d"),
        0x17 => rla(regs),
        0x18 => jr_e(regs, mem, rom),
        0x19 => add_hl(regs, mem, rom, "de"),
        0x1A => ld_r1_r2(regs, mem, rom, "a", "(de)"), //load a into (de), de is the memory address
        0x1B => dec_r(regs, mem, rom, "de"),
        0x1C => inc_r(regs, mem, rom, "e"),
        0x1D => dec_r(regs, mem, rom, "e"),
        0x1E => ld_n(regs, mem, rom, "e"),
        0x1F => rra(regs, mem, rom),
        0x20 => jr_f_e(regs, mem, rom, 'z', false),
        0x21 => ld_nn(regs, mem, rom, "hl"),
        0x22 => {
            ld_r1_r2(regs, mem, rom, "(hl)", "a");
            inc_r(regs, mem, rom, "hl");
        } //load a into (hl), hl is the memory address, then increment hl
        0x23 => inc_r(regs, mem, rom, "hl"),
        0x24 => inc_r(regs, mem, rom, "h"),
        0x25 => dec_r(regs, mem, rom, "h"),
        0x26 => ld_n(regs, mem, rom, "h"),
        0x27 => daa(regs, mem, rom),
        0x28 => jr_f_e(regs, mem, rom, 'z', true),
        0x29 => add_hl(regs, mem, rom, "hl"),
        0x2A => {
            ld_r1_r2(regs, mem, rom, "a", "(hl)");
            inc_r(regs, mem, rom, "hl");
        } //load (hl) into a, hl is the memory address, then increment hl
        0x2B => dec_r(regs, mem, rom, "hl"),
        0x2C => inc_r(regs, mem, rom, "l"),
        0x2D => dec_r(regs, mem, rom, "l"),
        0x2E => ld_n(regs, mem, rom, "l"),
        0x2F => cpl(regs),
        0x30 => jr_f_e(regs, mem, rom, 'c', false),
        0x31 => ld_nn(regs, mem, rom, "sp"),
        0x32 => {
            ld_r1_r2(regs, mem, rom, "(hl)", "a");
            dec_r(regs, mem, rom, "hl");
        } //load a into (hl), hl is the memory address, then decrement hl
        0x33 => inc_r(regs, mem, rom, "sp"),
        0x34 => inc_m(regs, mem, rom, "hl"),
        0x35 => dec_m(regs, mem, rom, "hl"),
        0x36 => ld_m_n(regs, mem, rom),
        0x37 => scf(regs),
        0x38 => jr_f_e(regs, mem, rom, 'c', true),
        0x39 => add_hl(regs, mem, rom, "sp"),
        0x3A => {
            ld_r1_r2(regs, mem, rom, "a", "(hl)");
            dec_r(regs, mem, rom, "hl");
        } //load (hl) into a, hl is the memory address, then decrement hl
        0x3B => dec_r(regs, mem, rom, "sp"),
        0x3C => inc_r(regs, mem, rom, "a"),
        0x3D => dec_r(regs, mem, rom, "a"),
        0x3E => ld_n(regs, mem, rom, "a"),
        0x3F => ccf(regs),
        0x40..0x7F => ld_r1_r2(
            regs,
            mem,
            rom,
            REG_NAMES[((opcode >> 4) - 2) as usize],
            REG_NAMES[(opcode & 0x0f) as usize],
        ),
        0x80..0x87 => add_a_r(regs, mem, rom, REG_NAMES[(opcode & 0x0f) as usize]),
        0x88..0x8F => adc_a_r(regs, mem, rom, REG_NAMES[(opcode & 0x0f) as usize]),
        0x90..0x97 => sub_a_r(regs, mem, rom, REG_NAMES[(opcode & 0x0f) as usize]),
        0x98..0x9F => sbc_a_r(regs, mem, rom, REG_NAMES[(opcode & 0x0f) as usize]),
        0xA0..0xA7 => and_a_r(regs, mem, rom, REG_NAMES[(opcode & 0x0f) as usize]),
        0xA8..0xAF => xor_a_r(regs, mem, rom, REG_NAMES[(opcode & 0x0f) as usize]),
        0xB0..0xB7 => or_a_r(regs, mem, rom, REG_NAMES[(opcode & 0x0f) as usize]),
        0xB8..0xBF => cp_a_r(regs, mem, rom, REG_NAMES[(opcode & 0x0f) as usize]),
        0xC0 => ret_f(regs, mem, rom, 'z', false), //return if z flag is false
        0xC1 => pop(regs, mem, rom, "bc"),
        0xC2 => jp_f_nn(regs, mem, rom, 'z', false),
        0xC3 => jp_nn(regs, mem, rom),
        0xC4 => call_f_nn(regs, mem, rom, 'z', false),
        0xC5 => push(regs, mem, rom, "bc"),
        0xC6 => add_a_n(regs, mem, rom),
        0xC7 => rst(regs, mem, rom, 0x00),
        0xC8 => ret_f(regs, mem, rom, 'z', true), //return if z flag is true
        0xC9 => ret(regs, mem, rom),
        0xCA => jp_f_nn(regs, mem, rom, 'z', true),
        0xCB => call_CB(regs, mem, rom),
        0xCC => call_f_nn(regs, mem, rom, 'z', true),
        0xCD => call_nn(regs, mem, rom),
        0xCE => adc_a_n(regs, mem, rom),
        0xCF => rst(regs, mem, rom, 0x08),
        0xD0 => ret_f(regs, mem, rom, 'c', false), //return if c flag is false
        0xD1 => pop(regs, mem, rom, "de"),
        0xD2 => jp_f_nn(regs, mem, rom, 'c', false),
        0xD3 => unimplemented!(),
        0xD4 => call_f_nn(regs, mem, rom, 'c', false),
        0xD5 => push(regs, mem, rom, "de"),
        0xD6 => sub_a_n(regs, mem, rom),
        0xD7 => rst(regs, mem, rom, 0x10),
        0xD8 => ret_f(regs, mem, rom, 'c', true), //return if c flag is true
        0xD9 => reti(regs, mem, rom),
        0xDA => jp_f_nn(regs, mem, rom, 'c', true),
        0xDB => unimplemented!(),
        0xDC => call_f_nn(regs, mem, rom, 'c', true),
        0xDD => unimplemented!(),
        0xDE => sbc_a_n(regs, mem, rom),
        0xDF => rst(regs, mem, rom, 0x18),

        _ => println!("Opcode not implemented: {:X}", opcode),
    }
}

fn handle_post_instruction(
    regs: &mut Registers,
    mem: &mut Memory,
    rom: &mut Rom,
    opcode: u8,
    mut lenght: u64,
) {
    //increment pc
    let mut delta = 0;
    let pc = regs.read_register_16("pc").clone();
    regs.write_register_16("pc", pc + OPCODE_LENGHTS[opcode.clone() as usize] as u16);
    lenght += OPCODE_LENGHTS[opcode.clone() as usize] as u64;
    delta += OPCODE_LENGHTS[opcode as usize] as u64;
    if opcode == 0xCB {
        let pc = regs.read_register_16("pc").clone();
        let opcode = read_memory_8(mem, rom, pc);
        regs.write_register_16("pc", pc + OPCODE_LENGHTS_CB[opcode.clone() as usize] as u16);
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
    regs.write_register_16("bc", 0x0112);
    println!("af : {:X}", regs.read_register_16("bc"));
    //load Rom.gb to Rom buffer
    let mut rom = Rom {
        buffer: [0; 0x2FFFF],
        bank: null_mut(),
    };
    let mut mem: Memory = [0; 65535];
    //;load Rom to Rom buffer
    let mut file = File::open("rom.gb").unwrap();
    #[allow(clippy::unused_io_amount)]
    file.read(&mut rom.buffer).unwrap();
    //load Rom header
    for i in 0..16 {
        header.title[i] = rom.buffer[i + 0x134] as char;
    }
    for i in 0..48 {
        header.logo[i] = rom.buffer[0x100 + i] as char;
    }
    header.c_type = rom.buffer[0x147];
    header.rom_size = rom.buffer[0x148];
    header.ram_size = rom.buffer[0x149];
    header.destination = rom.buffer[0x14A];
    header.old_licensee = rom.buffer[0x14B];
    header.mask_rom_version = rom.buffer[0x14C];
    header.header_checksum = rom.buffer[0x14D];
    header.global_checksum[0] = rom.buffer[0x14E];
    header.global_checksum[1] = rom.buffer[0x14F];
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
        rom.bank = rom.buffer.as_mut_ptr().offset(0x4000);
    }

    for i in 0..0xFFFF {
        mem[i] = 0;
    }
    write_memory_16(&mut mem, &mut rom, 0x9000, 0x1234);
    println!("mem[0x9000] : {:X}", read_memory_16(&mem, &rom, 0x9000));
}
