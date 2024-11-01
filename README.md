# Prop Simulator

`prop-simulator` is a Monte Carlo simulator for evaluating the expected value (EV) of proprietary (prop) trading accounts using historical or simulated trade data. It supports both command-line interface (CLI) and web server modes, allowing users to interact with the simulator according to their preferences.

---

## Table of Contents

- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Cloning the Project](#cloning-the-project)
- [Building and Running the Simulator](#building-and-running-the-simulator)
  - [CLI Mode](#cli-mode)
    - [Building and Running the CLI](#building-and-running-the-cli)
  - [Web Server Mode](#web-server-mode)
    - [Building and Running the Web Server](#building-and-running-the-web-server)
- [Using the Simulator](#using-the-simulator)
  - [CLI Mode](#cli-mode-1)
    - [Mode 1: Using Historical Trade Data (CSV)](#mode-1-using-historical-trade-data-csv)
    - [Mode 2: Using Simulated Bracket Parameters](#mode-2-using-simulated-bracket-parameters)
    - [Viewing the Histogram](#viewing-the-histogram)
  - [Web Server Mode](#web-server-mode-1)
    - [Sending Requests](#sending-requests)
      - [Using `curl`](#using-curl)
      - [Using Postman](#using-postman)
    - [Viewing the Histogram](#viewing-the-histogram-1)
    - [Example Request and Response](#example-request-and-response)
- [Enabling Logging](#enabling-logging)
- [Options Summary](#options-summary)
- [TODO](#todo)

---

## Getting Started

### Prerequisites

Before running the simulator, ensure you have:

- **Git**: Used to clone the project from GitHub.
- **Rust**: The programming language required to compile and run the simulator.
- **Cargo**: Rust's package manager and build tool (installed with Rust).

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

---

## Building and Running the Simulator

The simulator can be built and run in two modes:

1. **CLI Mode**: Interact with the simulator via the command line.
2. **Web Server Mode**: Run the simulator as a web server and interact via HTTP requests.

### CLI Mode

#### Building and Running the CLI

To build and run the simulator in CLI mode, use the following command:

```bash
cargo run --features "cli" -- <arguments>
```

**Example**:

```bash
cargo run --features "cli" -- --csv-file ./sample_trades.csv --iterations 50000 --max-simulation-days 200 --account-type ftt:GT --multiplier 20
```

### Web Server Mode

#### Building and Running the Web Server

To build and run the simulator in web server mode, use the following command:

```bash
cargo run --no-default-features --features "web"
```

This will start the web server on `http://127.0.0.1:8080`.

---

## Using the Simulator

### CLI Mode

#### Mode 1: Using Historical Trade Data (CSV)

If you have historical trade data in a CSV file, you can run the simulator with the following command:

```bash
cargo run --features "cli" -- --csv-file ./sample_trades.csv --iterations 50000 --max-simulation-days 200 --account-type topstep:OneFifty --multiplier 20
```

- `--csv-file ./sample_trades.csv`: Path to the CSV file containing historical trade data.
- `--iterations 50000`: Sets the number of Monte Carlo iterations.
- `--max-simulation-days 200`: Maximum days to simulate.
- `--account-type topstep:OneFifty`: Account type to simulate, in the format `company:account`.
- `--multiplier 20`: Multiplier for trade values (e.g., to convert points to dollars).

**CSV File Format**:

The CSV file should be formatted as follows:

```csv
DateTime,Return,Max Opposite Excursion
2024-09-12 19:20:00,17.45,-9
2024-09-12 20:02:00,18.45,-6
2024-09-13 00:59:00,22.20,-18.75
```

#### Mode 2: Using Simulated Bracket Parameters

If you don't have historical data, you can simulate trade results based on stop loss, take profit, win percentage, and average trades per day:

```bash
cargo run --features "cli" -- --iterations 1000000 --avg-trades-per-day 10 --stop-loss 40 --take-profit 40 --win-percentage 50 --max-simulation-days 200 --account-type ftt:Rally --multiplier 20
```

- `--avg-trades-per-day 10`: Average number of trades per day.
- `--stop-loss 40`: Stop loss in ticks.
- `--take-profit 40`: Take profit in ticks.
- `--win-percentage 50`: Win percentage for the simulated strategy.

#### Viewing the Histogram

You can generate and save a histogram of the final account balances by including the `--histogram` flag:

```bash
cargo run --features "cli" -- --csv-file ./sample_trades.csv --iterations 50000 --max-simulation-days 200 --account-type ftt:GT --multiplier 20 --histogram --histogram-file balance_histogram.png
```

- `--histogram`: Enables histogram generation.
- `--histogram-file balance_histogram.png`: Specifies the filename for saving the histogram image (default is `final_balances_histogram.png`).

This will output a histogram showing the distribution of final balances after all simulation iterations.

### Web Server Mode

The web server mode allows you to interact with the simulator via HTTP requests. You can send simulation configurations and receive results, including a histogram image.

#### Sending Requests

You can interact with the web server using tools like `curl` or Postman.

##### Using `curl`

**Example Request**:

```bash
curl -X POST http://127.0.0.1:8080/simulate \
     -F 'config={"iterations":10000,"max_simulation_days":200,"max_payouts":12,"account_type":"ftt:GT","multiplier":40,"histogram":true,"condition_end_state":"All"}' \
     -F 'csv_file=@./sample_trades.csv' \
     -H "Accept: application/json" \
     -o response.json
```

- `-X POST`: Specifies a POST request.
- `http://127.0.0.1:8080/simulate`: The endpoint for simulation.
- `-F 'config=...'`: The simulation configuration in JSON format.
- `-F 'csv_file=@./sample_trades.csv'`: Uploads the CSV file.
- `-H "Accept: application/json"`: Indicates that we expect a JSON response.
- `-o response.json`: Saves the response to `response.json`.

**Configuration Parameters**:

- `iterations`: Number of simulation iterations.
- `max_simulation_days`: Maximum days to simulate.
- `max_payouts`: Maximum number of payouts.
- `account_type`: Account type (e.g., "ftt:GT").
- `multiplier`: Multiplier for trade values.
- `histogram`: Set to `true` to generate a histogram.
- `condition_end_state`: Specifies the condition end state (e.g., "All").

##### Using Postman

1. **Create a New POST Request**:

   - URL: `http://127.0.0.1:8080/simulate`

2. **Set Request Type to `form-data`**:

   - In the "Body" tab, select "form-data".

3. **Add Form Fields**:

   - **Key**: `config`
     - **Type**: Text
     - **Value**: Paste your JSON configuration.

   - **Key**: `csv_file`
     - **Type**: File
     - **Value**: Select your CSV file.

4. **Send the Request**.

5. **View the Response**:

   - The response will be in JSON format, including the simulation results and the histogram image as a Base64-encoded string.

#### Viewing the Histogram

Since the histogram image is returned as a Base64-encoded string within the JSON response, you'll need to extract and decode it.

**Decoding the Histogram Image Using Python**:

```python
import base64
import json

# Load the JSON response
with open('response.json', 'r') as f:
    data = json.load(f)

# Get the Base64-encoded image data
image_base64 = data.get('histogram_image_base64')
if image_base64:
    # Decode and save the image
    image_data = base64.b64decode(image_base64)
    with open('histogram.png', 'wb') as f:
        f.write(image_data)
    print("Histogram image saved as 'histogram.png'")
else:
    print("No histogram image found in the response.")
```

#### Example Request and Response

**Request**:

```bash
curl -X POST http://127.0.0.1:8080/simulate \
     -F 'config={"iterations":5000,"max_simulation_days":100,"account_type":"ftt:GT","multiplier":20,"histogram":true,"condition_end_state":"All"}' \
     -F 'csv_file=@./sample_trades.csv' \
     -H "Accept: application/json" \
     -o response.json
```

**Response (`response.json`)**:

```json
{
  "mean_balance": 1200.50,
  "median_balance": 1100.00,
  "std_dev": 300.75,
  "mad": 250.60,
  "iqr": 400.00,
  "mad_median": 200.50,
  "mean_days": 90.25,
  "end_state_percentages": {
    "Busted": 25.0,
    "TimeOut": 50.0,
    "MaxPayouts": 25.0
  },
  "positive_balance_percentage":16.27,
  "histogram_image_base64": "iVBORw0KGgoAAAANSUhEUgAA..."
}
```

---

## Enabling Logging

The program includes logging functionality that provides detailed information about the simulation process.

To enable detailed logging, set the `RUST_LOG` environment variable:

- **On Linux/macOS**:

  ```bash
  RUST_LOG=info cargo run --features "cli" -- <arguments>
  ```

- **On Windows** (PowerShell):

  ```powershell
  $env:RUST_LOG="info"
  cargo run --features "cli" -- <arguments>
  ```

Replace `<arguments>` with your specific command-line arguments.

---

## Options Summary

### Common Options

| Option                         | Description                                                                                     |
|--------------------------------|-------------------------------------------------------------------------------------------------|
| `--iterations <number>`        | Number of Monte Carlo simulation iterations. Default is 10,000.                                 |
| `--max-simulation-days <days>` | Maximum days to simulate. Default is 365.                                                       |
| `--account-type <type>`        | Account type to simulate (e.g., ftt:Rally, ftt:Daytona, ftt:GT, ftt:LeMans, topstep:Fifty, topstep:OneHundred, topstep:OneFifty). Default is ftt:GT.                     |
| `--multiplier <value>`         | Multiplier for scaling trade values (e.g., points to dollars).                                  |
| `--histogram`                  | Enables histogram generation for final account balances.                                        |
| `--histogram-file <file>`      | Filename to save the histogram image (CLI mode only). Default is `final_balances_histogram.png`.|
| `--condition-end-state <state>`| Condition end state for statistics (e.g., "All", "Busted", "TimeOut", "MaxPayouts").            |

### Options for Historical Data Mode

| Option                   | Description                                                                                     |
|--------------------------|-------------------------------------------------------------------------------------------------|
| `--csv-file <file>`      | Path to the CSV file containing historical trade data.                                          |

### Options for Simulated Bracket Mode

| Option                     | Description                                                                                     |
|----------------------------|-------------------------------------------------------------------------------------------------|
| `--avg-trades-per-day <n>` | Average number of trades per day for simulated bracket strategy.                                |
| `--stop-loss <ticks>`      | Stop loss in ticks for simulated bracket strategy.                                              |
| `--take-profit <ticks>`    | Take profit in ticks for simulated bracket strategy.                                            |
| `--win-percentage <%>`     | Win percentage for the simulated strategy.                                                      |

---

## TODO

- [x] Add logging for simulation events and results (`log` and `env_logger`).
- [x] Visualizations for simulation results (`plotters`).
- [x] Support for bracket and win percentage options (for those not using a CSV file).
- [ ] Make `max_opposite_excursion` optional in trade data.
- [x] Add support for additional account types, such as Apex Trader Funding, Tradeify, Topstep Futures, etc.
- [x] Gather more data from simulation: distribution of account lifetimes, percentage blown/timeout/max payouts, average lifetimes and returns for those groupings.
- [x] Use `actix-web` to handle HTTP requests.
- [x] Include histogram image in the JSON response as Base64-encoded data.
- [ ] Use different histogram plotting library for web server, maybe `plotly`. Keep plotters for CLI

---

This project leverages **Rust**'s performance and safety features, along with parallelism provided by `rayon`, to efficiently perform large-scale simulations. Future plans include enhanced output analysis, improved visualization, and support for additional account types.

---

**Note**: Always ensure you have the latest version of the project and dependencies by pulling updates from the repository and updating Rust and Cargo.

**Contact**: For questions or contributions, please open an issue or submit a pull request on the [GitHub repository](https://github.com/razor389/prop-simulator).

---

# License

This project is licensed under the terms of the MIT license.

---