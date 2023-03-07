use crate::memory::Memory;

pub struct Ppu {
    temp: u8,
    oams: [Oam; 40],
    background_line: [u8; 160],
    window_line: [u8; 160],
    vx: u8,
    vy: u8,
}

const LCDC_REGISTER: u16 = 0xFF40;

const OAM_MEM_START: u16 = 0xFE00;

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

const LY_POSITION: u16 = 0xFF44;
const SCY_POSITION: u16 = 0xFF42;
const SCX_POSITION: u16 = 0xFF43;

#[derive(Clone, Copy)]
struct Oam {
    y_pos: u8,
    x_pos: u8,
    tile_indx: u8,
    flags: u8,
}

const PRIORITY: u8 = 0b10000000;
const Y_FLIP: u8 = 0b01000000;
const X_FLIP: u8 = 0b00100000;
const PALETTE: u8 = 0b00010000;
const VRAM_BANK: u8 = 0b00001000;
const CGB_PALETTE: u8 = 0b00000111;

type Tile = [u8; 16];

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            temp: 0,
            oams: [Oam {
                y_pos: 0,
                x_pos: 0,
                tile_indx: 0,
                flags: 0,
            }; 40],
        }
    }

    fn load_oam(&mut self, mem: &Memory) {
        let mut oams = [Oam {
            y_pos: 0,
            x_pos: 0,
            tile_indx: 0,
            flags: 0,
        }; 40];

        for i in 0..39 {
            let oam = &mut oams[i];
            let oam_addr = OAM_MEM_START + (i * 4) as u16;
            oam.y_pos = mem.read_8(oam_addr);
            oam.x_pos = mem.read_8(oam_addr + 1);
            oam.tile_indx = mem.read_8(oam_addr + 2);
            oam.flags = mem.read_8(oam_addr + 3);
        }

        self.oams = oams;
    }

    fn load_backgroundline(&mut self, mem: &Memory) {
        let mut background_line: [u8; 160] = [0; 160];
        let y = self.vy;
        let x = self.vx;
        let ty = y >> 3;
        for i in 0..159 {
            let x_pos = x.wrapping_add(i as u8);
            let tx = x_pos >> 3;
            let addr = 0x9800 + (ty * 0x20) + tx;
        }
    }
}
