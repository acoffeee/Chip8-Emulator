# Chip8-Emulator

A [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) interpreter core written in Rust, with support for the CHIP-8E extension opcodes and a custom extension of my own, **CHIP-8C(offeee)**.

> **Status:** early work in progress. The CPU core and opcode set are written, but there is no frontend yet (no window, no input, no sound, no ROM loader), and there are known bugs (see below).

## What's in here

```
.
├── src/
│   ├── lib.rs        # Chip8 struct, opcodes, fetch/decode/execute cycle
│   ├── lexer.rs      # (WIP) lexer for the planned assembly language
│   ├── token.rs      # (WIP) token definitions for that language
│   └── readfile.rs   # (WIP) source file reader
├── Cargo.toml        # Rust 2024 edition, depends on `rand`
└── Cargo.lock
```

`lexer.rs`, `token.rs`, and `readfile.rs` are drafts and are not wired into the crate yet.

## Emulated hardware

| Component     | Details                                             |
| ------------- | --------------------------------------------------- |
| Memory        | 4 KB, programs load at `0x200`                      |
| Registers     | 16 8-bit registers (`V0`-`VF`), 16-bit index `I`    |
| Stack         | 16 levels                                           |
| Display       | 64x32 buffer                                        |
| Keypad        | 16 keys                                             |
| Timers        | Delay and sound                                     |
| Font          | Built-in 4x5 hex font, expected at `0x50`           |

## Opcode Reference

Opcodes are split into three sets: the **base** CHIP-8 instructions, the **CHIP-8E** extension, and my own **CHIP-8C(offeee)** extension. Notation: `nnn` = 12-bit address, `kk` / `NN` = 8-bit value, `n` = 4-bit value, `x` / `y` = register indices, `I` = index register.

### Base CHIP-8

| Opcode | Mnemonic          | Description                                                                 |
| ------ | ----------------- | --------------------------------------------------------------------------- |
| `00E0` | `cls`             | Clear the display                                                           |
| `00EE` | `ret`             | Return from a subroutine (pop the stack into `pc`)                          |
| `1nnn` | `jp addr`         | Jump to `nnn`                                                               |
| `2nnn` | `call addr`       | Call the subroutine at `nnn` (push `pc` onto the stack)                     |
| `3xkk` | `se Vx, byte`     | Skip the next instruction if `Vx == kk`                                     |
| `4xkk` | `sne Vx, byte`    | Skip the next instruction if `Vx != kk`                                     |
| `5xy0` | `se Vx, Vy`       | Skip the next instruction if `Vx == Vy`                                     |
| `6xkk` | `ld Vx, byte`     | Set `Vx = kk`                                                               |
| `7xkk` | `add Vx, byte`    | Set `Vx = Vx + kk`                                                          |
| `8xy0` | `ld Vx, Vy`       | Set `Vx = Vy`                                                               |
| `8xy1` | `or Vx, Vy`       | Set `Vx = Vx OR Vy`                                                         |
| `8xy2` | `and Vx, Vy`      | Set `Vx = Vx AND Vy`                                                        |
| `8xy3` | `xor Vx, Vy`      | Set `Vx = Vx XOR Vy`                                                        |
| `8xy4` | `add Vx, Vy`      | Set `Vx = Vx + Vy`; `VF = 1` on carry, else `0`                             |
| `8xy5` | `sub Vx, Vy`      | Set `Vx = Vx - Vy`; `VF = 1` if `Vx > Vy` (no borrow), else `0`             |
| `8xy6` | `shr Vx`          | Shift `Vx` right by 1; `VF` = the bit shifted out                           |
| `8xy7` | `subn Vx, Vy`     | Set `Vx = Vy - Vx`; `VF = 1` if `Vy > Vx` (no borrow), else `0`             |
| `8xyE` | `shl Vx {, Vy}`   | Shift `Vx` left by 1; `VF` = the bit shifted out                            |
| `9xy0` | `sne Vx, Vy`      | Skip the next instruction if `Vx != Vy`                                     |
| `Annn` | `ld I, addr`      | Set `I = nnn`                                                               |
| `Bnnn` | `jp V0, addr`     | Jump to `nnn + V0`                                                          |
| `Cxkk` | `rnd Vx, byte`    | Set `Vx = (random byte) AND kk`                                             |
| `Dxyn` | `drw Vx, Vy, n`   | Draw an `n`-byte sprite from memory at `I` to `(Vx, Vy)`; `VF = 1` on collision |
| `Ex9E` | `skp Vx`          | Skip the next instruction if the key in `Vx` is pressed                     |
| `ExA1` | `sknp Vx`         | Skip the next instruction if the key in `Vx` is not pressed                 |
| `Fx07` | `ld Vx, DT`       | Set `Vx` = delay timer                                                      |
| `Fx0A` | `ld Vx, K`        | Wait for a key press, then store the key in `Vx`                            |
| `Fx15` | `delay DT, Vx`    | Set the delay timer = `Vx`                                                  |
| `Fx18` | `sound ST, Vx`    | Set the sound timer = `Vx`                                                  |
| `Fx1E` | `add I, Vx`       | Set `I = I + Vx`                                                            |
| `Fx29` | `hex F, Vx`       | Set `I` to the font sprite address for the digit in `Vx`                    |
| `Fx33` | `bcd B, Vx`       | Store the BCD of `Vx` at `I`, `I+1`, `I+2` (hundreds, tens, ones)           |
| `Fx55` | `stor [I], Vx`    | Store `V0`-`Vx` in memory starting at `I`                                   |
| `Fx65` | `rstr Vx, [I]`    | Read `V0`-`Vx` from memory starting at `I`                                  |

The original `0nnn` (call machine-code routine) is not implemented.

### CHIP-8E Extension

Reference: [chip-8.github.io/extensions](https://chip-8.github.io/extensions/#chip-8e). The mnemonics below are my own, since the original documentation didn't define any. The I/O opcodes use a Rust channel (`Sender` / `Receiver`) in place of hardware port 3.

| Opcode | Mnemonic          | Description                                                                 |
| ------ | ----------------- | --------------------------------------------------------------------------- |
| `00ED` | `stop`            | Trap the CPU in an infinite loop, effectively halting execution             |
| `0151` | `wait`            | Loop in place until the delay timer reaches 0                               |
| `00F2` | `nope`            | Do nothing                                                                  |
| `0188` | `sp`              | Skip the next instruction                                                   |
| `5XY1` | `spg Vx, Vy`      | Skip the next instruction if `Vx > Vy`                                      |
| `5XY2` | `stor Vx, Vy`     | Store registers `Vx` through `Vy` in memory starting at `I`                 |
| `5XY3` | `rstr Vx, Vy`     | Load registers `Vx` through `Vy` from memory starting at `I`                |
| `BBNN` | `jpb NN`          | Jump backward by `NN` from the current location                             |
| `BFNN` | `jpf NN`          | Jump forward by `NN` from the current location                              |
| `FX03` | `out Vx`          | Output the contents of `Vx` to port 3                                       |
| `FX1B` | `sp Vx`           | Skip `Vx` bytes forward (no-op if `Vx` is 0); a substitute for `Bnnn`       |
| `FX4F` | `hlt Vx`          | Set the timer to `Vx` and wait until it reaches 0                           |
| `FXE3` | `hltread Vx`      | Block until data arrives on the input channel, then store it in `Vx`        |
| `FXE7` | `read Vx`         | Non-blocking read from the input channel into `Vx` (`0` if nothing is there) |

### CHIP-8C(offeee) Extension

My own additions. These use previously unassigned `Fx25` / `Fx35` opcodes.

| Opcode | Mnemonic  | Description                                    |
| ------ | --------- | ---------------------------------------------- |
| `Fx25` | `stor Vx` | Store register `Vx` in memory at address `I`   |
| `Fx35` | `rstr Vx` | Load the byte at memory address `I` into `Vx`  |

## Planned assembly language

The lexer drafts point toward a small assembler with named aliases:

```
.aliases
alias register_one v0;
alias memory_one 0x7D0;

.store_named_registers
ld I, memory_one
stor register_one, register_one
```

## Building

```bash
git clone https://github.com/acoffeee/Chip8-Emulator.git
cd Chip8-Emulator
cargo build
cargo test
```

Requires a recent stable Rust toolchain (edition 2024).

## To-Do

### Known bugs (found by reading the code)

- [ ] `6xkk` / `7xkk`: operator precedence bug in `self.opcode & 0x0F00 >> 8` (needs parentheses)
- [ ] `8xy2` (AND) uses XOR, and `8xy3` (XOR) uses OR
- [ ] `7xkk`, `8xy4`, `8xy5`, `8xy7` overflow/underflow and will panic in debug builds (use `wrapping_add` / `wrapping_sub`; compute the carry in `u16`)
- [ ] `8xyE` should set `VF` to 0 or 1 (MSB), not `0x80`
- [ ] `Cxkk` doesn't shift the register index (`>> 8`)
- [ ] `Dxyn`: draws `SCREEN_HEIGHT` rows instead of `n`, uses `&` instead of `%` for Y, has no wrapping/bounds check, and the collision check (`> 0xFFFFFFFF`) can never be true
- [ ] `SCREEN_WIDTH` / `SCREEN_HEIGHT` are 128 but the video buffer is 64x32
- [ ] Font set is never copied into memory at `0x50`
- [ ] `Fx35` reads from `registers[I]` instead of `memory[I]`
- [ ] Timers tick once per CPU cycle instead of at 60 Hz
- [ ] `hlt` (`FX4F`) blocks the thread with `sleep`

### Core

- [ ] Add a `main.rs` and make `Chip8` public
- [ ] ROM loading from a file path
- [ ] Run loop with a configurable clock speed
- [ ] Bounds checking on memory, stack, and `pc`
- [ ] Proper handling of unknown opcodes (return an error instead of printing)

### Frontend

- [ ] Display rendering (window or terminal)
- [ ] Keyboard input mapped to the 16-key keypad
- [ ] Sound (beep while the sound timer is active)

### Assembler

- [ ] Fix and finish `lexer.rs`, `token.rs`, and `readfile.rs`, then add them as modules
- [ ] Parser and code generator that emit ROM bytes
- [ ] Alias support and automatic memory allocation

### Quality and housekeeping

- [ ] Replace the placeholder test (`it_works` calls an `add` function that doesn't exist)
- [ ] Unit tests for every opcode
- [ ] Run the community CHIP-8 test ROMs
- [ ] Clean up compiler warnings (Rust naming conventions, unused values) and run `cargo clippy` / `cargo fmt`
- [ ] Set up CI
- [ ] Add a LICENSE
- [ ] Add a repo description and topics on GitHub
- [ ] Add screenshots once there is something to show

## Resources

- [Cowgod's CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Tobias V. Langhoff's guide to writing a CHIP-8 emulator](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/)
- [CHIP-8 extensions reference](https://chip-8.github.io/extensions/)
- [Timendus CHIP-8 test suite](https://github.com/Timendus/chip8-test-suite)

## License

TBD.
generated bby claude tbh