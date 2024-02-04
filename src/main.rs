use std::ops::Index;
use std::ops::IndexMut;

// http://www.6502.org/users/obelisk/

const MAX_MEM: usize = 1024 * 64;

pub struct Memory {
    data: [u8; MAX_MEM],
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            data: [0; MAX_MEM],
        }
    }
}

impl Memory {
    pub fn initialize(&mut self) {
        self.data = [0; MAX_MEM];
    }

    /// Write 2 bytes to memory
    pub fn write_word(&mut self, cycles: &mut u8, data: u16, addr: u16) {
        self.data[addr as usize]     = (data & 0xFF) as u8;
        self.data[(addr + 1) as usize] = (data >> 8) as u8;
        *cycles -= 2;
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.data[index as usize]
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.data[index as usize]
    }
}

struct PS;

impl PS {
    // const C: u8 = 0b00000001; // Carry Flag
    const Z: u8 = 0b00000010; // Zero Flag
    // const I: u8 = 0b00000100; // Interrupt Flag
    // const D: u8 = 0b00001000; // Decimal Mode Flag
    // const B: u8 = 0b00010000; // Break Command
    // const V: u8 = 0b01000000; // Overflow Flag
    const N: u8 = 0b10000000; // Negative Flag
}

struct INS;

impl INS {
    const LDA_IM: u8 = 0xA9;  // Load Accumulator - Immediate
    const LDA_ZP: u8 = 0xA5;  // Load Accumulator - Zero Page
    const LDA_ZPX: u8 = 0xB5;  // Load Accumulator - Zero Page,X
    const JSR_ABS: u8 = 0x20; // Jump to Subroutine - Absolute
}

#[derive(Debug, Default)]
pub struct CPU {
    pc: u16, // Program Counter
    sp: u16, // Stack Pointer
    a: u8,   // Accumulator
    x: u8,   // Index Register X
    y: u8,   // Index Register Y
    ps: u8,  // Processor Status
}

impl CPU {
    pub fn reset(&mut self, memory: &mut Memory) {
        self.pc = 0xFFFC;
        self.sp = 0x0100;
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.ps = 0;

        memory.initialize();
    }

    fn fetch_byte(&mut self, cycles: &mut u8, memory: &mut Memory) -> u8 {
        let data = memory[self.pc];
        self.pc += 1;
        *cycles -= 1;

        data
    }

    fn fetch_word(&mut self, cycles: &mut u8, memory: &mut Memory) -> u16 {
        // 6502 is little endian
        let mut data = memory[self.pc] as u16;
        self.pc += 1;

        data |= ((memory[self.pc] as u16) << 8) as u16;
        self.pc += 1;

        *cycles -= 2;

        data
    }

    fn read_byte(&mut self, cycles: &mut u8, memory: &mut Memory, addr: u8) -> u8 {
        let data = memory[addr as u16];
        *cycles -= 1;

        data
    }

    pub fn execute(&mut self, cycles: u8, memory: &mut Memory) {
        let mut cycles = cycles;
        while cycles > 0 {
            match self.fetch_byte(&mut cycles, memory) {
                INS::LDA_IM => {
                    self.a = self.fetch_byte(&mut cycles, memory);
                    self.lda_set_status();
                },
                INS::LDA_ZP => {
                    let zp_addr = self.fetch_byte(&mut cycles, memory);
                    self.a = self.read_byte(&mut cycles, memory, zp_addr);
                    self.lda_set_status();
                },
                INS::LDA_ZPX => {
                    let mut zp_addr = self.fetch_byte(&mut cycles, memory);
                    zp_addr += self.x; cycles -= 1;
                    self.a = self.read_byte(&mut cycles, memory, zp_addr);
                    self.lda_set_status();
                },
                INS::JSR_ABS => {
                    let sub_addr = self.fetch_word(&mut cycles, memory); // Takes 2 cycles
                    memory.write_word(&mut cycles, self.pc - 1, self.sp); // Takes 2 cycles
                    self.sp += 2;
                    self.pc = sub_addr; cycles -= 1;
                },
                opcode => println!("Instruction not handled: ${:X}", opcode),
            }
        }
    }

    fn lda_set_status(&mut self) {
        if self.a == 0 {self.ps |= PS::Z}
        if self.a & 0b10000000 > 0 {self.ps |= PS::N}
    }
}

fn main() {
    let mut memory = Memory::default();
    let mut cpu = CPU::default();

    cpu.reset(&mut memory);

    // START - Inlined program for testing
    memory[0xFFFC] = INS::JSR_ABS; // 6 Cycles
    memory[0xFFFD] = 0x42;
    memory[0xFFFE] = 0x42;

    memory[0x4242] = INS::LDA_IM; // 2 Cycles
    memory[0x4243] = 0x84;
    // END - Inlined program for testing

    cpu.execute(8, &mut memory);

    dbg!(cpu);
}
