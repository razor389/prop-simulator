use clap::Parser;
use crate::trade_data::read_csv;
use ftt_account::FttAccountType;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use trade_data::{calculate_trades_per_day, TradeRecord};
use trader::Trader;
use std::error::Error;

mod trade_data;
mod ftt_account;
mod trader;

/// Monte Carlo simulation for trading accounts.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the CSV file containing trade data
    #[arg(short, long)]
    csv_file: String,

    /// Number of iterations for the Monte Carlo simulation
    #[arg(short, long, default_value_t = 10000)]
    iterations: usize,

    /// Maximum number of trades per day (optional)
    #[arg(short = 't', long)]
    max_trades_per_day: Option<u64>,

    /// Daily profit target (optional)
    #[arg(short = 'p', long)]
    daily_profit_target: Option<f64>,

    /// Daily stop-loss limit (optional)
    #[arg(short = 's', long)]
    daily_stop_loss: Option<f64>,

    /// Maximum number of simulation days
    #[arg(short = 'd', long, default_value_t = 365)]
    max_simulation_days: u64,

    /// Maximum number of payouts before ending the simulation
    #[arg(short = 'm', long, default_value_t = 12)]
    max_payouts: u8,

    /// Account type: Rally, Daytona, GT, LeMans
    #[arg(short, long, default_value_t = String::from("GT"))]
    account_type: String,

    /// Multiplier for trade values (optional)
    #[arg(short = 'x', long, default_value_t = 1.0)]
    multiplier: f64,
}

// Monte Carlo simulation using parallel execution
fn monte_carlo_simulation(
    trades: &Vec<TradeRecord>,
    trades_per_day: &Vec<usize>,
    iterations: usize,
    account_type: FttAccountType,
    max_trades_per_day: Option<u64>,
    daily_profit_target: Option<f64>,
    daily_stop_loss: Option<f64>,
    max_simulation_days: u64,
    max_payouts: u8,
) -> Vec<f64> {
    // Parallel iterator for Monte Carlo simulation
    (0..iterations).into_par_iter().map(|_| {
        let mut rng = rand::thread_rng();

        // Initialize the trader with the specified FTT account type and rules
        let mut trader = Trader::new(
            account_type.clone(),
            max_trades_per_day,
            daily_profit_target,
            daily_stop_loss,
            max_simulation_days,
            max_payouts,
        );

        // Simulation loop for each iteration
        loop {
            // Randomly select the number of trades per day based on empirical distribution
            let num_trades_today = *trades_per_day.choose(&mut rng).unwrap();

            // Simulate a day of trading with randomly selected trades
            let trades_today: Vec<_> = (0..num_trades_today)
                .map(|_| trades.choose(&mut rng).unwrap().trade.clone())
                .collect();

            let trading_day_result = trader.trade_day(&mut trades_today.clone());

            // End simulation if the account was busted, timed out, or max payouts were hit
            if let Some(end_of_game) = trading_day_result.end_of_game {
                match end_of_game {
                    trader::EndOfGame::MaxPayouts | trader::EndOfGame::TimeOut | trader::EndOfGame::Busted => {
                        break;
                    }
                }
            }
        }

        trader.bank_account.balance
    }).collect() // Collect the final bank balances into a Vec
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments using Clap
    let cli = Cli::parse();

    // Parse account type from string
    let account_type = match cli.account_type.as_str() {
        "Rally" => FttAccountType::Rally,
        "Daytona" => FttAccountType::Daytona,
        "LeMans" => FttAccountType::LeMans,
        _ => FttAccountType::GT, // Default to GT if unknown
    };

    // Load the trade records from the CSV file
    let trades = read_csv(&cli.csv_file, cli.multiplier)?;

    // Group trades by day and get the distribution of trades per day
    let trades_per_day_map = calculate_trades_per_day(&trades);

    // Convert the HashMap values into a Vec for easy random sampling
    let trades_per_day: Vec<usize> = trades_per_day_map.values().cloned().collect();

    // Run Monte Carlo simulation
    let final_balances = monte_carlo_simulation(
        &trades,
        &trades_per_day,
        cli.iterations,
        account_type,
        cli.max_trades_per_day,
        cli.daily_profit_target,
        cli.daily_stop_loss,
        cli.max_simulation_days,
        cli.max_payouts,
    );

    // Compute statistics from the results (e.g., mean balance)
    let mean_balance: f64 = final_balances.iter().sum::<f64>() / final_balances.len() as f64;
    println!("Mean final bank account balance: {}", mean_balance);

    Ok(())
}
