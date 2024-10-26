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
- [ ] Use `actix-web` to handle HTTP requests (see below)

## Command Line + Actix Web App

### 1. Modularize Core Logic

Put the core simulation functionality into a dedicated Rust module or library that both the CLI and web server can call. This way,we avoid code duplication and ensure that any updates apply to both interfaces.

- Move the core functionality (e.g., `monte_carlo_simulation`, data handling, calculations) into a separate module like `src/simulator/`.
- Create functions within `src/simulator/` that provide a consistent API for running simulations, regardless of input source (CSV or simulated data).

For example, the folder structure could look like this:

```
prop-simulator/
├── src/
│   ├── main.rs            // CLI entry point
│   ├── web.rs             // Web server entry point
│   ├── simulator/         // Core simulation logic
│   │   ├── lib.rs
│   │   ├── trade_data.rs
│   │   ├── ftt_account.rs
│   │   ├── trader.rs
│   │   └── utils.rs
└── Cargo.toml
```

### 2. Implement the Core Library in `src/simulator/lib.rs`

Move all the core simulation logic to `src/simulator/lib.rs`. The CLI and web server can then import this as a module.

#### `src/simulator/lib.rs`

```rust
pub mod trade_data;
pub mod ftt_account;
pub mod trader;
pub mod utils;

use trade_data::{generate_simulated_trades, read_csv, calculate_trades_per_day, TradeRecord};
use ftt_account::FttAccountType;
use trader::Trader;
use utils::*;

pub struct SimulationConfig {
    // configuration fields (iterations, account type, etc.)
}

pub struct SimulationResult {
    pub mean_balance: f64,
    pub median_balance: f64,
    pub std_dev: f64,
    pub mad: f64,
    pub iqr: f64,
    pub mad_median: f64,
}

pub fn run_simulation(config: SimulationConfig) -> SimulationResult {
    // Core simulation logic here
}
```

### 3. CLI Entry Point (`main.rs`)

Keep the command-line interface code in `src/main.rs`, but update it to call the core functionality in `simulator::lib`.

#### `src/main.rs`

```rust
use clap::Parser;
use simulator::{SimulationConfig, run_simulation};

#[derive(Parser)]
struct Cli {
    // Define CLI arguments here
}

fn main() {
    let args = Cli::parse();

    let config = SimulationConfig {
        // Populate config based on CLI arguments
    };

    let result = run_simulation(config);

    // Output results in the console
    println!("Mean balance: {}", result.mean_balance);
}
```

### 4. Web Server Entry Point (`web.rs`)

Implement the Actix Web API in `src/web.rs`. This file can import `simulator::lib` to use the same core functionality as the CLI.

#### `src/web.rs`

```rust
use actix_web::{post, web, App, HttpServer, Responder};
use simulator::{SimulationConfig, run_simulation};

#[post("/simulate")]
async fn simulate(params: web::Json<SimulationConfig>) -> impl Responder {
    let result = run_simulation(params.into_inner());
    web::Json(result)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(simulate))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
```

### 5. Use Feature Flags (Optional)

If we want users to choose between CLI and web versions when they build the app, we can use [Cargo feature flags](https://doc.rust-lang.org/cargo/reference/features.html). This allows us to enable the web server or CLI separately during compilation.

In `Cargo.toml`:

```toml
[features]
default = ["cli"]
cli = []
web = ["actix-web"]

[dependencies]
actix-web = { version = "4", optional = true }
clap = { version = "4", features = ["derive"] }
```

In `src/main.rs`:

```rust
#[cfg(feature = "cli")]
fn main() {
    // CLI entry point logic here
}

#[cfg(feature = "web")]
fn main() {
    web::start_server();  // Call a function in web.rs to start the server
}
```

Run the CLI version with:

```bash
cargo run --features "cli"
```

Run the web version with:

```bash
cargo run --features "web"
```

---

This project leverages **Rust**'s parallelism with `rayon` for efficient performance on multi-core systems. Future plans include enhanced output analysis and improved visualization.