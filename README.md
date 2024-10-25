# Prop Simulator

`prop-simulator` is a Monte Carlo simulator for evaluating the expected value (EV) of proprietary (prop) trading accounts using historical or simulated trade data. It is currently set up for Fast Track Trading accounts, with plans to add more options in the future.

## Getting Started

### Prerequisites

Before running the simulator, ensure you have:
- **Git**: Used to clone the project from GitHub.
- **Rust**: The programming language required to compile and run the simulator.

#### Install Git

If Git is not installed, you can download it [here](https://git-scm.com/downloads) and follow the installation instructions for your OS. Verify installation by running:

```bash
git --version
```

#### Install Rust

Install Rust using the `rustup` installer. You can find instructions [here](https://www.rust-lang.org/tools/install). Verify installation by running:

```bash
rustc --version
```

### Cloning the Project

Clone the project repository from GitHub:

```bash
git clone https://github.com/razor389/prop-simulator.git
```

Navigate into the project directory:

```bash
cd prop-simulator
```

### Building and Running the Simulator

To build and run the simulator, use the `cargo` tool:

```bash
cargo run -- <arguments>
```

## Running the Simulator

The simulator can be run in two modes:

1. **With Historical Trade Data (CSV)**
2. **With Simulated Bracket Parameters (stop loss, take profit, win %)**
   
### Mode 1: Using Historical Trade Data (CSV)

If you have historical trade data in a CSV file, you can run the simulator with the following command:

```bash
cargo run -- --csv-file ./sample_trades.csv --iterations 50000 --max-simulation-days 200 --account-type GT --multiplier 20
```

- `--csv-file ./sample_trades.csv`: Path to the CSV file containing historical trade data.
- `--iterations 50000`: Sets the number of Monte Carlo iterations.
- `--max-simulation-days 200`: Maximum days to simulate.
- `--account-type GT`: Account type to simulate.
- `--multiplier 20`: Multiplier for trade values (e.g., to convert points to dollars).

The CSV file should be formatted as follows:

```csv
DateTime,Return,Max Opposite Excursion
2024-09-12 19:20:00,17.45,-9
2024-09-12 20:02:00,18.45,-6
2024-09-13 00:59:00,22.20,-18.75
```

### Mode 2: Using Simulated Bracket Parameters

If you don't have historical data, you can simulate trade results based on a stop loss, take profit, win percentage, and average trades per day:

```bash
cargo run -- --iterations 1000000 --avg-trades-per-day 10 --stop-loss 40 --take-profit 40 --win-percentage 50 --max-simulation-days 200 --account-type Rally --multiplier 20
```

This command will simulate trades using the provided parameters instead of reading from a CSV file.

- `--avg-trades-per-day 10`: Average number of trades per day.
- `--stop-loss 40`: Stop loss in ticks.
- `--take-profit 40`: Take profit in ticks.
- `--win-percentage 50`: Win percentage for the simulated strategy.

## Viewing the Histogram

You can generate and save a histogram of the final account balances by including the `--histogram` flag:

```bash
cargo run -- --csv-file ./sample_trades.csv --iterations 50000 --max-simulation-days 200 --account-type GT --multiplier 20 --histogram --histogram-file balance_histogram.png
```

- `--histogram`: Enables histogram generation.
- `--histogram-file balance_histogram.png`: Specifies the filename for saving the histogram image (default is `final_balances_histogram.png`).

This will output a histogram showing the distribution of final balances after all simulation iterations.

## Enabling Logging

The program includes logging functionality that provides detailed debug information for each trading day, such as daily P&L, number of trades, trading day results, bank account balance, and FTT account balance. 

To enable detailed logging, set the `RUST_LOG` environment variable:

- **On Linux/macOS**:

  ```bash
  RUST_LOG=debug cargo run -- <arguments>
  ```

- **On Windows** (PowerShell):

  ```powershell
  $env:RUST_LOG="debug"
  cargo run -- <arguments>
  ```

This will output debug information to the console, allowing you to trace each day’s activities in the simulation.

## Options Summary

| Option                   | Description                                                                                     |
|--------------------------|-------------------------------------------------------------------------------------------------|
| `--csv-file <file>`      | Path to the CSV file containing historical trade data.                                          |
| `--iterations <number>`  | Number of Monte Carlo simulation iterations. Default is 10,000.                                 |
| `--max-simulation-days`  | Maximum days to simulate. Default is 365.                                                       |
| `--account-type <type>`  | Account type to simulate (e.g., Rally, Daytona, GT, LeMans). Default is GT.                     |
| `--multiplier <value>`   | Multiplier for scaling trade values (e.g., points to dollars).                                  |
| `--avg-trades-per-day`   | Average trades per day for simulated bracket strategy.                                          |
| `--stop-loss <ticks>`    | Stop loss in ticks for simulated bracket strategy.                                              |
| `--take-profit <ticks>`  | Take profit in ticks for simulated bracket strategy.                                            |
| `--win-percentage <%>`   | Win percentage for the simulated strategy.                                                      |
| `--histogram`            | Enables histogram generation for final account balances.                                        |
| `--histogram-file <file>`| Filename to save the histogram image. Default is `final_balances_histogram.png`.                |

## TODO

- [x] Add logging for simulation events and results (`log` and `env_logger`).
- [x] Visualizations for simulation results (`plotters`).
- [x] Support for bracket and win percentage options (for those not using a CSV file).
- [ ] Make `max_opposite_excursion` optional in trade data.
- [ ] Add support for additional account types, such as Apex Trader Funding, Tradeify, Topstep Futures, etc.
- [ ] Gather more data from simulation: distribution of account lifetimes, percentage blown/timeout/max payouts, average lifetimes and returns for those groupings

---

This project leverages **Rust**'s parallelism with `rayon` for efficient performance on multi-core systems. Future plans include enhanced output analysis and improved visualization.