use std::ptr::{null, null_mut};

type MainMemory = [u8; 0xFFFF];

type RawBankNumber = u8;

const BANK_MASK: u8 = 0b00011111;

type Bank = [u8; 0x4000];

pub struct Memory {
    pub main_memory: MainMemory,
    pub rom: Rom,
}

impl Memory {
    pub(crate) fn new() -> Memory {
        Memory {
            main_memory: [0; 0xFFFF],
            rom: Rom {
                buffer: [0; 0x2FFFF],
                bank: null_mut(),
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct Rom {
    pub buffer: [u8; 0x2FFFF],
    pub bank: *mut u8,
}

const RBN: u16 = 0x2000;

impl Memory {
    pub fn read_8(&self, address: u16) -> u8 {
        if (0x4000..0x8000).contains(&address) {
            unsafe {
                let x = *(self.rom.bank.offset((address - 0x4000) as isize));
                return x;
            }
        }
        self.main_memory[address as usize]
    }

    pub fn read_16(&self, address: u16) -> u16 {
        let x = self.read_8(address);
        let y = self.read_8(address + 1);
        (y as u16) << 8 | x as u16
    }

    fn write_to_rom_register(&mut self, address: u16, value: u8) {
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
                self.rom.bank = self.rom.buffer.as_mut_ptr().offset(bank_adress as isize);
            }
        }
    }

    pub fn write_8(&mut self, address: u16, value: u8) {
        if address < 0x8000 {
            self.write_to_rom_register(address, value);
        } else {
            self.main_memory[address as usize] = value;
        }
    }
    pub fn write_16(&mut self, address: u16, value: u16) {
        self.write_8(address, (value >> 8) as u8);
        self.write_8(address + 1, (value & 0xFF) as u8);
    }

    //set entire rom buffer
    pub fn set_rom(&mut self, rom: [u8; 0x2FFFF]) {
        self.rom.buffer = rom;
        unsafe {
            self.rom.bank = self.rom.buffer.as_mut_ptr().offset(0x4000);
        }
    }
}
