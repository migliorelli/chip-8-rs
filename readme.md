# Chip-8 Rust

My first emulator, written in Rust

## TODO

- [ ] Add configurable CHIP-8 vs SCHIP quirks (`8XY6`/`8XYE` shift operands, `FX55`/`FX65` I-register increment).
- [ ] **Audio implementation:**
    - [ ] Implement an audio callback/beeper via `sdl2::audio` when `st > 0`.
- [ ] **Timing & Performance:**
    - [ ] Decouple CPU cycles from display refresh (run timers at a strict 60 Hz, CPU at ~500–700 Hz).
    - [ ] Only redraw the screen when a display flag is set by `00E0` or `DXYN`.
- [ ] **CLI & UX:**
    - [ ] Support custom window scaling and clock frequency via CLI flags.
    - [ ] Add ROM drag-and-drop or file picker.
    - [ ] Custom color pallet
    - [ ] Add pause and step-by-step debug controls.
- [ ] **Testing:**
    - [ ] Pass Tim Duspoit's test suite (`test_opcode.ch8`, `bc_test.ch8`).

## Build & Run

### Prerequisites
- Rust
- Cargo
- A Chip-8 ROM

### Build

```shell
cargo build --release
```

Binary will be at `target/release/chip8rs`

### Run

Without building

```shell
cargo run /path/to/game
```

From binary
```shell
chip8rs /path/to/rom
```

## Author

Built by [Miguel Migliorelli Bringhenti](https://github.com/migliorelli)