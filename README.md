# ModbusTcp-Server

A cross-platform Modbus TCP Server with an interactive GUI, built with Rust.

![Rust](https://img.shields.io/badge/Rust-2024-orange)
![License](https://img.shields.io/badge/License-MIT%20with%20Attribution-blue)

## Features

- **Modbus TCP Server** — Full compliance with the Modbus TCP protocol, supporting standard function codes:
  - Read Coils (FC01)
  - Read Discrete Inputs (FC02)
  - Read Input Registers (FC03)
  - Read Holding Registers (FC04)
  - Write Single Coil (FC05)
  - Write Multiple Coils (FC15)
  - Write Single Register (FC06)
  - Write Multiple Registers (FC16)

- **Interactive GUI** — Real-time visualization and editing of all Modbus data areas via an egui/eframe desktop application.

- **Multi-Data-Type Registers** — Registers can be configured to display and edit values as:
  - `U16`, `I16` (16-bit)
  - `U32`, `I32`, `F32` (32-bit, spanning 2 registers)
  - `U64`, `I64`, `F64` (64-bit, spanning 4 registers)

- **Multiple Display Formats** — View and input register values in:
  - Decimal (DEC)
  - Hexadecimal (HEX)
  - Binary (BIN)
  - Octal (OCT)

- **Word & Byte Swap** — Configure word-swap and byte-swap options for multi-register data types, accommodating different byte ordering conventions (Big-Endian, Little-Endian, etc.).

- **Configurable Offset & Count** — Dynamically adjust the count and offset for Coils, Discrete Inputs, Input Registers, and Holding Registers at runtime.

- **Dark & Light Themes** — Switch between dark and light UI themes with a single click.

- **Scalable UI** — Launch with a custom scale factor via the `--scale` CLI argument for high-DPI displays.

- **Server Address Configuration** — Choose between `127.0.0.1` (localhost) and `0.0.0.0` (all interfaces), with configurable port number (default: 502).

- **Real-Time Updates** — GUI instantly reflects changes made by connected Modbus clients, and client-side writes are immediately visible in the interface.

## Screenshots

> *(Add screenshots here if available)*

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024 or later)

### Build & Run

```bash
# Clone the repository
git clone https://github.com/nczyw/modbustcp-server.git
cd modbustcp-server

# Build and run
cargo run

# Run with a custom UI scale factor (e.g., 1.5 for high-DPI)
cargo run -- --scale 1.5
```

### Command-Line Options

| Option       | Short | Default | Description                          |
|-------------|-------|---------|--------------------------------------|
| `--scale`   | `-s`  | `1.0`   | UI display scale factor (e.g., 1.5) |

## Usage

1. **Start the server** — Click the **Connect** button in the status bar to start listening on the configured IP and port.
2. **Connect a client** — Use any Modbus TCP client (e.g., [modpoll](https://www.modbusdriver.com/modpoll.html), [PyModbus](https://github.com/pymodbus-dev/pymodbus), or your own application) to connect to the server.
3. **View & edit data** — 
   - **Coils / Discrete Inputs**: Toggle checkboxes directly in the table.
   - **Input / Holding Registers**: Left-click a register value to open the **Edit** dialog; right-click to open the **Settings** dialog (data type & display format).
4. **Stop the server** — Click the **Disconnect** button in the status bar.

## Architecture

```
src/
├── main.rs                  # Entry point: spawns Modbus server thread & launches GUI
├── modbus.rs                 # Modbus module declarations
├── modbus/
│   ├── modbustcp_server.rs   # Modbus TCP service implementation (Service trait)
│   └── share_data.rs         # Shared data model, register types, display formats, read/write logic
├── ui.rs                     # UI module declarations
├── ui/
│   └── app_ui.rs             # egui/eframe GUI application (tables, dialogs, status bar)
fonts/
├── AlibabaPuHuiTi-3-55-Regular.ttf   # Chinese font (embedded)
├── NotoEmoji-VariableFont_wght.ttf   # Emoji font (embedded)
```

The Modbus server runs on a dedicated Tokio runtime thread, while the GUI runs on the main thread. Shared state (`ShareData`) is protected by an `Arc<RwLock>` for safe concurrent access, with UI refresh notifications via Tokio `Notify` and `watch` channels.

## Dependencies

| Crate             | Purpose                        |
|-------------------|--------------------------------|
| `eframe`          | Desktop GUI framework          |
| `egui_extras`     | Table widget for register view  |
| `tokio`           | Async runtime for Modbus server|
| `tokio-modbus`    | Modbus TCP server protocol     |
| `clap`            | CLI argument parsing           |
| `anyhow`          | Error handling                 |

## License

This project is licensed under the MIT License with Attribution Requirement. Attribution Requirement: Any use, copy, modification, merge, publication, distribution, sublicense, or sale of this software (including derivative works and binary distributions) must clearly and prominently display the original repository and author information (WenJun) in the most conspicuous manner possible — either within the software's user interface (e.g., title bar, about dialog, status bar) or alongside the distributed binary files (e.g., README, LICENSE, splash screen). See the [LICENSE](LICENSE) file for full details.