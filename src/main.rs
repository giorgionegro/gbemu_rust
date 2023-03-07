#![feature(exclusive_range_pattern)]
#![allow(clippy::unused_io_amount)]
extern crate bitintr;
use bitintr::*;

mod cpu;
mod memory;
mod ppu;

use crate::memory::Memory;
use cpu::Cpu;
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
    let mut cpu = Cpu::new();

    cpu.registers.write_16("af", 0x01B0);
    cpu.registers.write_16("bc", 0x0112);
    println!("af : {:X}", cpu.registers.read_16("bc"));
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
