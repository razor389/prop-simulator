# Prop Simulator

`prop-simulator` is a Monte Carlo simulator for evaluating prop account expected value using historical trade data. currently implemented for fast track trading accounts. more to come.

## Getting Started

### Prerequisites

Before running the simulator, ensure you have installed:
- **Git**: Used to clone the project from GitHub.
- **Rust**: The programming language required to compile and run the simulator.

#### Install Git

If you don't have Git installed, you can download it [here](https://git-scm.com/downloads) and follow the installation instructions for your operating system.

After installation, you can verify it by running the following command in your terminal or command prompt:

```bash
git --version
```

#### Install Rust

To install Rust, use the official Rust toolchain installer, `rustup`. You can find instructions [here](https://www.rust-lang.org/tools/install).

After installation, verify it with:

```bash
rustc --version
```

### Cloning the Project

Once Git and Rust are installed, you need to clone this project from GitHub to your local machine. Run the following command in your terminal or command prompt:

```bash
git clone https://github.com/razor389/prop-simulator.git
```

This will download the project files into a folder named `prop-simulator`. Navigate into the project directory:

```bash
cd prop-simulator
```

### Building and Running the Simulator

To build and run the simulator with a .csv file of trades, use `cargo`, Rust's package manager and build tool. From the `prop-simulator` directory, run:

```bash
cargo run -- --csv-file ./sample_trades.csv --iterations 50000 --max-simulation-days 200 --account-type GT --multiplier 20
```

This command will execute the simulator with the following parameters:
- `--csv-file ./sample_trades.csv`: Specifies the path to the CSV file containing historical trade data.
- `--iterations 50000`: Sets the number of Monte Carlo simulation iterations.
- `--max-simulation-days 200`: Simulates up to 200 trading days.
- `--account-type GT`: Simulates the GT account type.
- `--multiplier 20`: Multiplies the trade return values by 20 (useful for converting points into dollars or other units).

### Options:

- `--csv-file` (required): Path to the CSV file containing trade data (see sample_trades.csv for format).
- `--iterations`: Number of Monte Carlo iterations (default: `10000`).
- `--max-simulation-days`: Maximum number of days to simulate (default: `365`).
- `--account-type`: Type of account to simulate (e.g., `Rally`, `Daytona`, `GT`, `LeMans`).
- `--multiplier`: Multiplier for trade return and excursion values (if your trade data is not in dollars or otherwise needs rescaling).

To run the simulator with a bracket and win % instead of a .csv file, run:

```bash
cargo run -- --iterations 1000000 --avg-trades-per-day 10 --stop-loss 40 --take-profit 40 --win-percentage 50 --max-simulation-days 200 --account-type Rally --multiplier 20
```

### Example CSV Format

The CSV file should contain columns like `DateTime`, `Return`, and `Max Opposite Excursion`, formatted as shown below:

```csv
DateTime,Return,Max Opposite Excursion
2024-09-12 19:20:00,17.45,-9
2024-09-12 20:02:00,18.45,-6
2024-09-13 00:59:00,22.20,-18.75
```

## TODO:

- [ ] Add logging for simulation events and results. (use `log` and `env_logger` libs)
- [x] Visualizations for simulation results. (use `plotters` lib)
- [x] Support for bracket and win percentage options (for those not using a returns file).
- [ ] Make `max_opposite_excursion` optional in trade data.
- [ ] Add support for other account types, like Apex Trader Funding, Tradeify, Topstep Futures, etc.

---

This project uses **Rust** for parallel simulations, leveraging `rayon` for efficient performance on multi-core systems. Future plans include better output analysis and visual representation of results.
