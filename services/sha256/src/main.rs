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
   // Initial hash values (first 32 bits of the fractional parts of the square roots of the first 8 primes)
   let mut h0: u32 = 0x6a09e667;
   let mut h1: u32 = 0xbb67ae85;
   let mut h2: u32 = 0x3c6ef372;
   let mut h3: u32 = 0xa54ff53a;
   let mut h4: u32 = 0x510e527f;
   let mut h5: u32 = 0x9b05688c;
   let mut h6: u32 = 0x1f83d9ab;
   let mut h7: u32 = 0x5be0cd19;

   // Round constants (first 32 bits of the fractional parts of the cube roots of the first 64 primes)
   let k: [u32; 64] = [
       0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
       0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
       0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
       0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
       0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
       0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
       0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
       0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
       0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
       0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
       0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
       0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
       0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
       0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
       0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
       0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
   ];

   // Input message (hardcoded)
   let message: &[u8] = b"hello";

   // Pre-processing: Padding the message
   let message_len = message.len();
   let bit_len = (message_len as u64) * 8;

   // Calculate the number of bytes needed for padding
   let mut padded_message_len = message_len + 1; // Add 1 byte for the '1' bit
   while (padded_message_len * 8) % 512 != 448 {
       padded_message_len += 1;
   }
   padded_message_len += 8; // Add 8 bytes for the length

   // Create a new array with the padded message (assuming it fits in one block)
   let mut padded_message = [0u8; 64];
   for i in 0..message_len {
       padded_message[i] = message[i];
   }
   // Append the '1' bit followed by zeros
   padded_message[message_len] = 0x80;

   // Append the original message length in bits as a 64-bit big-endian integer
   let bit_len_bytes = bit_len.to_be_bytes();
   for i in 0..8 {
       padded_message[56 + i] = bit_len_bytes[i];
   }

   // Process the message in 512-bit (64-byte) chunks
   let chunks = padded_message.len() / 64;
   for _ in 0..chunks {
       // Prepare the message schedule
       let mut w = [0u32; 64];
       for t in 0..16 {
           let i = t * 4;
           w[t] = ((padded_message[i] as u32) << 24)
                | ((padded_message[i + 1] as u32) << 16)
                | ((padded_message[i + 2] as u32) << 8)
                | (padded_message[i + 3] as u32);
       }
       for t in 16..64 {
           let s0 = w[t - 15].rotate_right(7) ^ w[t - 15].rotate_right(18) ^ (w[t - 15] >> 3);
           let s1 = w[t - 2].rotate_right(17) ^ w[t - 2].rotate_right(19) ^ (w[t - 2] >> 10);
           w[t] = w[t - 16]
                .wrapping_add(s0)
                .wrapping_add(w[t - 7])
                .wrapping_add(s1);
       }

       // Initialize working variables
       let mut a = h0;
       let mut b = h1;
       let mut c = h2;
       let mut d = h3;
       let mut e = h4;
       let mut f = h5;
       let mut g = h6;
       let mut h = h7;

       // Main compression function
       for t in 0..64 {
           let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
           let ch = (e & f) ^ ((!e) & g);
           let temp1 = h
               .wrapping_add(s1)
               .wrapping_add(ch)
               .wrapping_add(k[t])
               .wrapping_add(w[t]);
           let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
           let maj = (a & b) ^ (a & c) ^ (b & c);
           let temp2 = s0.wrapping_add(maj);

           h = g;
           g = f;
           f = e;
           e = d.wrapping_add(temp1);
           d = c;
           c = b;
           b = a;
           a = temp1.wrapping_add(temp2);
       }

       // Add the compressed chunk to the current hash value
       h0 = h0.wrapping_add(a);
       h1 = h1.wrapping_add(b);
       h2 = h2.wrapping_add(c);
       h3 = h3.wrapping_add(d);
       h4 = h4.wrapping_add(e);
       h5 = h5.wrapping_add(f);
       h6 = h6.wrapping_add(g);
       h7 = h7.wrapping_add(h);
   }

   // Output the final hash value
   // println!(
   //     "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
   //     h0, h1, h2, h3, h4, h5, h6, h7
   // );
   // put into bytes
   let mut output = [0u8; 32];
   let h0_bytes = h0.to_be_bytes();
   let h1_bytes = h1.to_be_bytes();
   let h2_bytes = h2.to_be_bytes();
   let h3_bytes = h3.to_be_bytes();
   let h4_bytes = h4.to_be_bytes();
   let h5_bytes = h5.to_be_bytes();
   let h6_bytes = h6.to_be_bytes();
   let h7_bytes = h7.to_be_bytes();
   for i in 0..4 {
       output[i] = h0_bytes[i];
       output[i + 4] = h1_bytes[i];
       output[i + 8] = h2_bytes[i];
       output[i + 12] = h3_bytes[i];
       output[i + 16] = h4_bytes[i];
       output[i + 20] = h5_bytes[i];
       output[i + 24] = h6_bytes[i];
       output[i + 28] = h7_bytes[i];
   }


    // Get pointer to output and its length
    let output_ptr = output.as_ptr();
    let output_len = output.len() as u32;

    // Pass `output_ptr` and `output_len` via registers a3 and a4
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


