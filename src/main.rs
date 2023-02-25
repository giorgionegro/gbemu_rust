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
const PixelWidth: u16 = 160;
const PixelHeight: u16 = 144;
const PixelSize: u16 = PixelWidth * PixelHeight;

const InterruptEnable: u16 = 0xFFFF;
const InterruptFlag: u16 = 0xFF0F;
const VBLANK_INTERRUPT: u8 = 0b00000001;
const LCD_INTERRUPT: u8 = 0b00000010;
const TIMER_INTERRUPT: u8 = 0b00000100;
const SERIAL_INTERRUPT: u8 = 0b00001000;
const JOYPAD_INTERRUPT: u8 = 0b00010000;

type sprite = [u8; 4];

const ZERO_FLAG: u8 = 0b10000000;
const SUBTRACT_FLAG: u8 = 0b01000000;
const HALF_CARRY_FLAG: u8 = 0b00100000;
const CARRY_FLAG: u8 = 0b00010000;

type Memory = [u8; 0xFFFF];

type RawBankNumber = u8;
const BANK_MASK: u8 = 0b00011111;
type bank = [u8; 0x4000];

#[derive(Clone, Copy)]
struct rom {
    buffer: [u8; 0x2FFFF],
    bank: *mut u8,
}
const RBN: u16 = 0x2000;
#[derive(Clone, Copy)]
pub struct registers {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,
    ime: u8,
}
//read_register_8 from register
impl registers {
    pub fn read_register_8(&mut self, register: char) -> u8 {
        //switch case for each 8 bit register a, b, c, d, e, h, l, f,
        let tmp = self.clone();
        return (match register {
            'a' => tmp.af >> 8,
            'b' => tmp.bc >> 8,
            'c' => tmp.bc & 0xFF,
            'd' => tmp.de >> 8,
            'e' => tmp.de & 0xFF,
            'h' => tmp.hl >> 8,
            'l' => tmp.hl & 0xFF,
            'f' => tmp.af & 0xFF,
            _ => 0,
        }) as u8;
    }

    pub fn read_register_16(&mut self, register: &str) -> u16 {
        //switch case for each 16 bit register af, bc, de, hl, sp, pc
        let af = "af";
        let bc = "bc";
        let de = "de";
        let hl = "hl";
        let sp = "sp";
        let pc = "pc";
        let tmp = self.clone();
        return match register {
            af => tmp.af,
            bc => tmp.bc,
            de => tmp.de,
            hl => tmp.hl,
            sp => tmp.sp,
            pc => tmp.pc,
            _ => 0,
        };
    }

    pub fn write_register_8(&mut self, register: char, value: u8) {
        //switch case for each 8 bit register a, b, c, d, e, h, l, f,
        let tmp = self.clone();
        match register {
            'a' => self.af = (tmp.af & 0xFF) | ((value as u16) << 8),
            'b' => self.bc = (tmp.bc & 0xFF) | ((value as u16) << 8),
            'c' => self.bc = (tmp.bc & 0xFF00) | value as u16,
            'd' => self.de = (tmp.de & 0xFF) | ((value as u16) << 8),
            'e' => self.de = (tmp.de & 0xFF00) | value as u16,
            'h' => self.hl = (tmp.hl & 0xFF) | ((value as u16) << 8),
            'l' => self.hl = (tmp.hl & 0xFF00) | value as u16,
            'f' => self.af = (tmp.af & 0xFF00) | value as u16,
            _ => (),
        }
    }

    fn write_register_16(&mut self, register: &str, value: u16) {
        //switch case for each 16 bit register _af, bc, de, hl, sp, pc
        let _af = "_af";
        let _bc = "bc";
        let _de = "de";
        let _hl = "hl";
        let _sp = "sp";
        let _pc = "pc";
        match register {
            _af => self.af = value,
            _bc => self.bc = value,
            _de => self.de = value,
            _hl => self.hl = value,
            _sp => self.sp = value,
            _pc => self.pc = value,
            _ => (),
        }
    }
}
fn read_memory_8(mem: &Memory, rom: &rom, address: u16) -> u8 {
    if address < 0x8000 && address >= 0x4000 {
        unsafe {
            let x = *(rom.bank.offset((address - 0x4000) as isize));
            return x;
        }
    }
    return mem[address as usize];
}

fn read_memory_16(mem: &Memory, rom: &rom, address: u16) -> u16 {
    let x = read_memory_8(mem, rom, address.clone());
    let y = read_memory_8(mem, rom, address + 1);
    return ((x as u16) << 8) | y as u16;
}

fn write_to_rom_register(rom: &mut rom, address: u16, value: u8) {
    if address > RBN && address < 2 * RBN {
        let bank_number: RawBankNumber = value & BANK_MASK;
        let mut bank_adress: u32;
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

fn write_memory_8(mem: &mut Memory, rom: &mut rom, address: u16, value: u8) {
    if address < 0x8000 {
        write_to_rom_register(rom, address, value);
    } else {
        mem[address as usize] = value;
    }
}
fn write_memory_16(mem: &mut Memory, rom: &mut rom, address: u16, value: u16) {
    write_memory_8(mem, rom, address.clone(), (value.clone() >> 8) as u8);
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
fn ld_nn(regs: &mut registers, mem: &mut Memory, rom: &mut rom, reg: &str) {
    let value = read_memory_16(mem, rom, regs.read_register_16("pc").clone() + 1);
    regs.write_register_16(reg, value);
}

fn ld_n(regs: &mut registers, mem: &mut Memory, rom: &mut rom, reg: &str) {
    let value = read_memory_8(mem, rom, regs.read_register_16("pc") + 1);
    regs.write_register_8(reg.chars().next().unwrap(), value);
}

fn ld_r1_r2(
    regs: &mut registers,
    mem: &mut Memory,
    rom: &mut rom,
    source: &str,
    dest: &str,
    srcmem: bool,
    dstmem: bool,
) {
    //load from memory(r1) to r2
    //if source have 1 char, it's a 8 bit register if dest have 1 char, it's a 8 bit register, if scrmem is true, it's a memory address(only for 16 bit registers) same for dstmem
    if source.len() == 1 {
        //source is 8 bit register can't be memory address
        if dest.len() == 1 {
            //dest is 8 bit register
            let value = regs.read_register_8(source.chars().next().unwrap()).clone();
            regs.write_register_8(dest.chars().next().unwrap(), value);
        } else {
            //dest is 16 bit register
            let value = regs.read_register_8(source.chars().next().unwrap()).clone();
            if dstmem {
                write_memory_8(mem, rom, regs.read_register_16(dest).clone(), value);
            } else {
                regs.write_register_16(dest, value as u16);
            }
        }
    } else {
        //source is 16 bit register could be memory address
        let value: u16 = if srcmem {
            read_memory_16(mem, rom, regs.read_register_16(source).clone())
        } else {
            regs.read_register_16(source).clone()
        };
        if dest.len() == 1 {
            regs.write_register_8(dest.chars().next().unwrap(), value as u8);
        } else {
            regs.write_register_16(dest, value);
        }
    }
}
// incr and decr
fn inc_r(regs: &mut registers, mem: &mut Memory, rom: &mut rom, reg: &str) {
    //if reg have 1 char, it's a 8 bit register
    if reg.len() == 1 {
        let value = regs.read_register_8(reg.chars().next().unwrap());
        regs.write_register_8(reg.chars().next().unwrap(), value + 1);
    } else {
        let value = regs.read_register_16(reg);
        regs.write_register_16(reg, value + 1);
    }
}

fn dec_r(regs: &mut registers, mem: &mut Memory, rom: &mut rom, reg: &str) {
    //if reg have 1 char, it's a 8 bit register
    if reg.len() == 1 {
        let value = regs.read_register_8(reg.chars().next().unwrap());
        regs.write_register_8(reg.chars().next().unwrap(), value - 1);
    } else {
        let value = regs.read_register_16(reg);
        regs.write_register_16(reg, value - 1);
    }
}
//rotate and shift
fn rlca(regs: &mut registers, mem: &mut Memory, rom: &mut rom) {
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

fn rla(regs: &mut registers, mem: &mut Memory, rom: &mut rom) {
    let value = regs.read_register_8('a').clone();
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
fn rrca(regs: &mut registers, mem: &mut Memory, rom: &mut rom) {
    let value = regs.read_register_8('a').clone();
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
fn rra(regs: &mut registers, mem: &mut Memory, rom: &mut rom) {
    let value = regs.read_register_8('a').clone();
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

//arithmetic
fn add_hl(regs: &mut registers, mem: &mut Memory, rom: &mut rom, reg: &str) {
    let value = regs.read_register_16(reg).clone();
    let hl = regs.read_register_16("hl").clone();
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

//misc
fn nop() {
    //do nothing
}

fn stop() {
    //stop cpu until button pressed
}

//logic
fn jr_n(regs: &mut registers, mem: &mut Memory, rom: &mut rom) {
    //temp register
    let tmp = regs.read_register_16("pc").clone() as i16;
    //read next byte
    let value = read_memory_8(mem, rom, tmp.clone() as u16 + 1) as i8;
    regs.write_register_16("pc", (tmp + value as i16) as u16);
}

fn jr_nz_n(regs: &mut registers, mem: &mut Memory, rom: &mut rom) {
    //temp register
    let tmp = regs.read_register_16("pc").clone() as i16;
    //read next byte
    let value = read_memory_8(mem, rom, tmp.clone() as u16 + 1) as i8;
    if regs.read_register_8('f') & ZERO_FLAG == 0 {
        regs.write_register_16("pc", (tmp + value as i16) as u16);
    }
}

fn execute(opcode: u8, regs: &mut registers, mem: &mut Memory, rom: &mut rom) {
    match opcode {
        0x00 => nop(),
        0x01 => ld_nn(regs, mem, rom, "bc"),
        0x02 => ld_r1_r2(regs, mem, rom, "bc", "a", true, false), //load (bc) into a, bc is the memory address
        0x03 => inc_r(regs, mem, rom, "bc"),
        0x04 => inc_r(regs, mem, rom, "b"),
        0x05 => dec_r(regs, mem, rom, "b"),
        0x06 => ld_n(regs, mem, rom, "b"),
        0x07 => rlca(regs, mem, rom),
        0x08 => ld_nn(regs, mem, rom, "sp"),
        0x09 => add_hl(regs, mem, rom, "bc"),
        0x0A => ld_r1_r2(regs, mem, rom, "a", "bc", false, true), //load a into (bc), bc is the memory address
        0x0B => dec_r(regs, mem, rom, "bc"),
        0x0C => inc_r(regs, mem, rom, "c"),
        0x0D => dec_r(regs, mem, rom, "c"),
        0x0E => ld_n(regs, mem, rom, "c"),
        0x0F => rrca(regs, mem, rom),
        0x10 => stop(),
        0x11 => ld_nn(regs, mem, rom, "de"),
        0x12 => ld_r1_r2(regs, mem, rom, "de", "a", true, false), //load (de) into a, de is the memory address
        0x13 => inc_r(regs, mem, rom, "de"),
        0x14 => inc_r(regs, mem, rom, "d"),
        0x15 => dec_r(regs, mem, rom, "d"),
        0x16 => ld_n(regs, mem, rom, "d"),
        0x17 => rla(regs, mem, rom),
        0x18 => jr_n(regs, mem, rom),
        0x19 => add_hl(regs, mem, rom, "de"),
        0x1A => ld_r1_r2(regs, mem, rom, "a", "de", false, true), //load a into (de), de is the memory address
        0x1B => dec_r(regs, mem, rom, "de"),
        0x1C => inc_r(regs, mem, rom, "e"),
        0x1D => dec_r(regs, mem, rom, "e"),
        0x1E => ld_n(regs, mem, rom, "e"),
        0x1F => rra(regs, mem, rom),
        0x20 => jr_nz_n(regs, mem, rom),
        0x21 => ld_nn(regs, mem, rom, "hl"),
        0x22 => {
            ld_r1_r2(regs, mem, rom, "a", "hl", false, true);
            inc_r(regs, mem, rom, "hl");
        } //load a into (hl), hl is the memory address, then increment hl
        0x23 => inc_r(regs, mem, rom, "hl"),

        _ => println!("Opcode not implemented: {:X}", opcode),
    }
}

fn handle_post_instruction(
    regs: &mut registers,
    mem: &mut Memory,
    rom: &mut rom,
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
    let mut regs = registers {
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
    //load rom.gb to rom buffer
    let mut rom = rom {
        buffer: [0; 0x2FFFF],
        bank: null_mut(),
    };
    let mut mem: Memory = [0; 65535];
    //;load rom to rom buffer
    let mut file = File::open("rom.gb").unwrap();
    file.read(&mut rom.buffer).unwrap();
    //load rom header
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
