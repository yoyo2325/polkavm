Here's the updated version of the Markdown file with the additional instruction added before the "Set Up the Toolchain" section:

---

# JAM Service: Privileged Service

This is a Rust-based JAM privileged service named "jam-service-priviliged," featuring the following entry points and functionality:

* `refine` (entry point 5) imports a 12-byte segment and exports the next one based on the calculation. Currently, `refine` is not in use.
* `accumulate` (entry point 10) reads the code hash from the PVM argument input and writes the service index into the privileged service's storage. The storage key is `[]byte{0, 0, 0, 0}`.
* `is_authorized` (entry point 0) and `on_transfer` (entry point 15) are basic placeholders for authorization control and transfer handling, but are not yet implemented.

The core function of this service is to add a new privileged service and write the new service's index to the privileged service's storage.

---

### Step 1: Download the Toolchain File

Please first visit [https://github.com/paritytech/rustc-rv32e-toolchain/releases](https://github.com/paritytech/rustc-rv32e-toolchain/releases) and download the appropriate toolchain file for your system.

---

### Step 2: Set Up the Toolchain

1. **Extract and Move the Toolchain**  
   Extract the toolchain file using `tar` with zstd compression and move it to the Rustup toolchain directory:
   ```bash
   tar --zstd -xf rust-rve-nightly-2024-01-05-x86_64-unknown-linux-gnu.tar.zst
   mv rve-nightly ~/.rustup/toolchains/
   ```

2. **Verify Toolchain Installation**  
   Check if the toolchain was successfully installed by listing the contents of the `toolchains` directory:
   ```bash
   ls ~/.rustup/toolchains/
   ```
   Ensure that `rve-nightly` appears among the other installed toolchains.

3. **Set the Toolchain as the Default**  
   Set `rve-nightly` as the default toolchain using the following commands:
   ```bash
   echo "export RUSTUP_TOOLCHAIN=rve-nightly" >> ~/.bashrc
   source ~/.bashrc
   ```

---

### Step 3: Build the JAM Privileged Service

1. **Navigate to the Service Directory**  
   Change to the directory containing the JAM privileged service:
   ```bash
   cd ./jamservices/services/priviliged
   ```

2. **Build the Service**  
   Build the service in release mode using Cargo:
   ```bash
   cargo build --release --target-dir ./target
   ```

---

### Step 4: Build the Polkatool

1. **Move to the Polkatool Directory**  
   Change to the `polkatool` tool directory:
   ```bash
   cd ./jamservices/tools/polkatool
   ```

2. **Build the Polkatool**  
   Build the tool in release mode:
   ```bash
   cargo build --release --target-dir ./target
   ```

---

### Step 5: Generate the Privileged Service and Blob

1. **Create the JAM Privileged Service and Blob from the Compiled Binary**  
   Use `polkatool` to generate the JAM privileged service and blob:
   ```bash
   cargo run -p polkatool jam-service services/priviliged/target/riscv32ema-unknown-none-elf/release/priviliged -o services/priviliged/priviliged.pvm -d services/priviliged/blob.pvm
   ```

   After running this command, two output files will be generated:
   - `priviliged.pvm`: A JAM-ready top-level service blob, as defined in equation 259 of A.7 in GP v0.4.1. Currently, this cannot be disassembled using `polkatool`.
   - `blob.pvm`: This can be disassembled using `polkatool`, and the disassembly result is provided below.

---

### Step 6: Disassemble the Blob

1. **Disassemble the Blob File and Optionally Show the Raw Bytes**  
   Disassemble the generated blob to view the instructions and raw bytes:
   ```bash
   cargo run -p polkatool disassemble services/priviliged/blob.pvm --show-raw-bytes
   ```

2. **Disassembly Result**  
   Below is the output of the disassembled `blob.pvm` file:

```
// RO data = 0/0 bytes
// RW data = 0/0 bytes
// Stack size = 4096 bytes
// Jump table entry point size = 0 bytes
// RO data = []
// RW data = []
// Instructions = 33
// Code size = 108 bytes

      :                          @0
     0: 05 11 00 00 00           jump @4
      :                          @1
     5: 05 10 00 00 00           jump @5
      :                          @2
    10: 05 0f 00 00 00           jump @6
      :                          @3
    15: 05 59                    jump @7
      :                          @4 [export #0: 'is_authorized']
    17: 04 07                    a0 = 0x0
    19: 13 00                    ret
      :                          @5 [export #1: 'refine']
    21: 04 07                    a0 = 0x0
    23: 13 00                    ret
      :                          @6 [export #2: 'accumulate']
    25: 02 11 f8                 sp = sp - 8
    28: 03 10 04                 u32 [sp + 4] = ra
    31: 03 15                    u32 [sp] = s0
    33: 04 05 00 00 fe fe        s0 = 0xfefe0000
    39: 04 07 00 00 fe fe        a0 = 0xfefe0000
    45: 04 08 b5 00              a1 = 0xb5
    49: 04 09 00 20              a2 = 0x2000
    53: 04 0a 00 10              a3 = 0x1000
    57: 04 0b 00 30              a4 = 0x3000
    61: 04 0c 00 40              a5 = 0x4000
    65: 4e 09                    ecalli 9 // 'new'
    67: 0d 05                    u32 [s0 + 0] = 0
    69: 03 57 04                 u32 [s0 + 4] = a0
    72: 04 09 04 00 fe fe        a2 = 0xfefe0004
    78: 04 07 00 00 fe fe        a0 = 0xfefe0000
    84: 04 08 04                 a1 = 0x4
    87: 04 0a 04                 a3 = 0x4
    90: 4e 03                    ecalli 3 // 'write'
    92: 04 07                    a0 = 0x0
    94: 01 10 04                 ra = u32 [sp + 4]
    97: 01 15                    s0 = u32 [sp]
    99: 02 11 08                 sp = sp + 0x8
   102: 13 00                    ret
      :                          @7 [export #3: 'on_transfer']
   104: 04 07                    a0 = 0x0
   106: 13 00                    ret
```