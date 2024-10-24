# Prop Simulator

`prop-simulator` is a Monte Carlo simulator for evaluating prop account EV using historical trade data. It simulates trading accounts with flexible parameters, allowing you to analyze performance over time.

## Usage

To run the simulator, use the following command:

```bash
cargo run -- --csv-file ./sample_trades.csv --iterations 50000 --max-simulation-days 200 --account-type GT --multiplier 20
```

### Options:

- `--csv-file` (required): Path to the CSV file containing trade data.
- `--iterations`: Number of Monte Carlo iterations (default: `10000`).
- `--max-simulation-days`: Maximum number of days to simulate (default: `365`).
- `--account-type`: Type of account to simulate (e.g., `Rally`, `Daytona`, `GT`, `LeMans`).
- `--multiplier`: Multiplier for trade values (useful for instruments like NQ).

## TODO:

- [ ] Add logging for simulation events and results.
- [ ] Visualizations for simulation results.
- [ ] Support for bracket and win percentage options (for those not using returns file).
- [ ] Make `max_opposite_excursion` optional in trade data.

---

This project uses **Rust** for parallel simulations, leveraging `rayon` for efficient performance on multi-core systems. Future plans include better output analysis and visual representation of results.
```