#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use simplealloc::SimpleAlloc;

#[global_allocator]
static ALLOCATOR: SimpleAlloc<4096> = SimpleAlloc::new();

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("unimp", options(noreturn));
    }
}

#[polkavm_derive::polkavm_import]
extern "C" {
    #[polkavm_import(index = 0)]
    pub fn gas() -> i64;
    #[polkavm_import(index = 1)]
    pub fn lookup(service: u32, hash_ptr: *const u8, out: *mut u8, out_len: u32) -> u32;
    #[polkavm_import(index = 2)]
    pub fn read(service: u32, key_ptr: *const u8, key_len: u32, out: *mut u8, out_len: u32) -> u32;
    #[polkavm_import(index = 3)]
    pub fn write(key_ptr: *const u8, key_len: u32, value: *const u8, value_len: u32) -> u32;
    #[polkavm_import(index = 4)]
    pub fn info(service: u32, out: *mut u8) -> u32;
    #[polkavm_import(index = 5)]
    pub fn empower(m: u32, a: u32, v: u32, o: u32, n: u32) -> u32;
    #[polkavm_import(index = 6)]
    pub fn assign(c: u32, out: *mut u8) -> u32;
    #[polkavm_import(index = 7)]
    pub fn designate(out: *mut u8) -> u32;
    #[polkavm_import(index = 8)]
    pub fn checkpoint() -> u64;
    #[polkavm_import(index = 9)]
    pub fn new(service: u32, hash_ptr: *const u8, out: *mut u8, out_len: u32) -> u32;
    #[polkavm_import(index = 10)]
    pub fn upgrade(out: *const u8, g: u64, m: u64) -> u32;
    #[polkavm_import(index = 11)]
    pub fn transfer(d: u32, a: u64, g: u64, out: *mut u8) -> u32;
    #[polkavm_import(index = 12)]
    pub fn quit(d: u32, a: u64, g: u64, out: *mut u8) -> u32;
    #[polkavm_import(index = 13)]
    pub fn solicit(hash_ptr: *const u8, z: u32) -> u32;
    #[polkavm_import(index = 14)]
    pub fn forget(hash_ptr: *const u8, z: u32) -> u32;
    #[polkavm_import(index = 15)]
    pub fn historical_lookup(service: u32, hash_ptr: *const u8, out: *mut u8, out_len: u32) -> u32;
    #[polkavm_import(index = 16)]
    pub fn import(import_index: u32, out: *mut u8, out_len: u32) -> u32;
    #[polkavm_import(index = 17)]
    pub fn export(out: *const u8, out_len: u32) -> u32;
    #[polkavm_import(index = 18)]
    pub fn machine(out: *const u8, out_len: u32) -> u32;
    #[polkavm_import(index = 19)]
    pub fn peek(out: *const u8, out_len: u32, i: u32) -> u32;
    #[polkavm_import(index = 20)]
    pub fn poke(n: u32, a: u32, b: u32, l: u32) -> u32;
    #[polkavm_import(index = 21)]
    pub fn invoke(n: u32, out: *mut u8) -> u32;
    #[polkavm_import(index = 22)]
    pub fn expunge(n: u32) -> u32;
    #[polkavm_import(index = 99)]
    pub fn blake2b(data: *const u8, data_len: u32, hash_ptr: *mut u8) -> u32;
    #[polkavm_import(index = 100)]
    pub fn blake2s(data: *const u8, data_len: u32, hash_ptr: *mut u8) -> u32;
    #[polkavm_import(index = 101)]
    pub fn ecrecover(h: *const u8, v: *const u8, r: *const u8, s: *const u8, out: *mut u8) -> u32;
    #[polkavm_import(index = 102)]
    pub fn sha2_256(data: *const u8, data_len: u32, hash_ptr: *mut u8) -> u32;
}

#[polkavm_derive::polkavm_export]
extern "C" fn is_authorized() -> u32 {
    0
}

#[polkavm_derive::polkavm_export]
extern "C" fn refine() -> u32 {
     // Input message
     let input = b"Hello, world!";
     let output_len = 32; // For Keccak-256, the output length is 32 bytes
 
     // Constant definitions
     let keccak_round_constants: [u64; 24] = [
         0x0000000000000001,
         0x0000000000008082,
         0x800000000000808A,
         0x8000000080008000,
         0x000000000000808B,
         0x0000000080000001,
         0x8000000080008081,
         0x8000000000008009,
         0x000000000000008A,
         0x0000000000000088,
         0x0000000080008009,
         0x000000008000000A,
         0x000000008000808B,
         0x800000000000008B,
         0x8000000000008089,
         0x8000000000008003,
         0x8000000000008002,
         0x8000000000000080,
         0x000000000000800A,
         0x800000008000000A,
         0x8000000080008081,
         0x8000000000008080,
         0x0000000080000001,
         0x8000000080008008,
     ];
 
     let rotation_offsets: [[u64; 5]; 5] = [
         [0, 36, 3, 41, 18],
         [1, 44, 10, 45, 2],
         [62, 6, 43, 15, 61],
         [28, 55, 25, 21, 56],
         [27, 20, 39, 8, 14],
     ];
 
     // State array initialization
     let mut state = [[0u64; 5]; 5];
     let rate_in_bytes = 1088 / 8; // The bitrate of Keccak-256 is 1088 bits, i.e., 136 bytes
 
     // Padding
     let mut padded_input = input.to_vec();
     padded_input.push(0x01); // Padding start bit for Keccak-256
     while (padded_input.len() % rate_in_bytes) != (rate_in_bytes - 1) {
         padded_input.push(0x00);
     }
     padded_input.push(0x80); // Padding end bit
 
     // Absorption phase
     for chunk in padded_input.chunks(rate_in_bytes) {
         for i in 0..(rate_in_bytes / 8) {
             let mut bytes = [0u8; 8];
             let start = i * 8;
             let end = start + 8;
             bytes.copy_from_slice(&chunk[start..end]);
             let block = u64::from_le_bytes(bytes);
             let x = i % 5;
             let y = i / 5;
             state[x][y] ^= block;
         }
 
         // Keccak-f[1600] permutation
         for round in 0..24 {
             // θ (Theta) step
             let mut c = [0u64; 5];
             for x in 0..5 {
                 c[x] = state[x][0]
                     ^ state[x][1]
                     ^ state[x][2]
                     ^ state[x][3]
                     ^ state[x][4];
             }
 
             let mut d = [0u64; 5];
             for x in 0..5 {
                 d[x] = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
             }
 
             for x in 0..5 {
                 for y in 0..5 {
                     state[x][y] ^= d[x];
                 }
             }
 
             // ρ (Rho) and π (Pi) steps
             let mut b = [[0u64; 5]; 5];
             for x in 0..5 {
                 for y in 0..5 {
                     b[y][(2 * x + 3 * y) % 5] =
                         state[x][y].rotate_left(rotation_offsets[x][y] as u32);
                 }
             }
 
             // χ (Chi) step
             for x in 0..5 {
                 for y in 0..5 {
                     state[x][y] =
                         b[x][y] ^ ((!b[(x + 1) % 5][y]) & b[(x + 2) % 5][y]);
                 }
             }
 
             // ι (Iota) step
             state[0][0] ^= keccak_round_constants[round];
         }
     }
 
     // Squeezing phase
     let mut output = Vec::new();
     while output.len() < output_len {
         for y in 0..5 {
             for x in 0..5 {
                 if output.len() + 8 <= output_len {
                     output.extend_from_slice(&state[x][y].to_le_bytes());
                 } else if output.len() < output_len {
                     let remaining = output_len - output.len();
                     output.extend_from_slice(
                         &state[x][y].to_le_bytes()[..remaining],
                     );
                 }
             }
         }
 
         if output.len() < output_len {
             // Apply Keccak-f[1600] permutation again
             for round in 0..24 {
                 // θ (Theta) step
                 let mut c = [0u64; 5];
                 for x in 0..5 {
                     c[x] = state[x][0]
                         ^ state[x][1]
                         ^ state[x][2]
                         ^ state[x][3]
                         ^ state[x][4];
                 }
 
                 let mut d = [0u64; 5];
                 for x in 0..5 {
                     d[x] = c[(x + 4) % 5] ^ c[(x + 1) % 5].rotate_left(1);
                 }
 
                 for x in 0..5 {
                     for y in 0..5 {
                         state[x][y] ^= d[x];
                     }
                 }
 
                 // ρ (Rho) and π (Pi) steps
                 let mut b = [[0u64; 5]; 5];
                 for x in 0..5 {
                     for y in 0..5 {
                         b[y][(2 * x + 3 * y) % 5] =
                             state[x][y].rotate_left(rotation_offsets[x][y] as u32);
                     }
                 }
 
                 // χ (Chi) step
                 for x in 0..5 {
                     for y in 0..5 {
                         state[x][y] =
                             b[x][y] ^ ((!b[(x + 1) % 5][y]) & b[(x + 2) % 5][y]);
                     }
                 }
 
                 // ι (Iota) step
                 state[0][0] ^= keccak_round_constants[round];
             }
         }
     }
    // Change output vec to u8 array
    let mut output_array = [0u8; 32];
    output_array.copy_from_slice(&output[..32]);
    let output_ptr = output_array.as_ptr();
    let output_len = output_array.len() as u32;
    unsafe {
        core::arch::asm!(
            "mv a3, {0}",
            "mv a4, {1}",
            in(reg) output_ptr,
            in(reg) output_len,
        );
    }
    0
}

#[polkavm_derive::polkavm_export]
extern "C" fn accumulate() -> u32 {
    0
}

#[polkavm_derive::polkavm_export]
extern "C" fn on_transfer() -> u32 {
    0
}


