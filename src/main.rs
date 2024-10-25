use clap::Parser;
use plotting::plot_histogram;
use crate::trade_data::read_csv;
use ftt_account::FttAccountType;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use trade_data::{calculate_trades_per_day, generate_simulated_trades, TradeRecord};
use trader::Trader;
use std::error::Error;
use env_logger::Env;
#[allow(unused_imports)]
use log::{info, debug, warn, error};

mod trade_data;
mod ftt_account;
mod trader;
mod plotting;

/// Monte Carlo simulation for trading accounts.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the CSV file containing trade data (optional, required if not using simulation)
    #[arg(short = 'f', long)]
    csv_file: Option<String>,

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

    /// Average number of trades per day (for simulated strategy)
    #[arg(short = 'a', long)]
    avg_trades_per_day: Option<f64>,

    /// Stop loss (ticks) for simulated strategy (required if using simulated strategy)
    #[arg(long)]
    stop_loss: Option<f64>,

    /// Take profit (ticks) for simulated strategy (required if using simulated strategy)
    #[arg(long)]
    take_profit: Option<f64>,

    /// Win percentage (for simulated strategy)
    #[arg(long)]
    win_percentage: Option<f64>,

    /// Maximum number of simulation days
    #[arg(short = 'd', long, default_value_t = 365)]
    max_simulation_days: u64,

    /// Maximum number of payouts before ending the simulation
    #[arg(short = 'm', long, default_value_t = 12)]
    max_payouts: u8,

    /// Account type: Rally, Daytona, GT, LeMans
    #[arg(short = 'c', long, default_value_t = String::from("GT"))]
    account_type: String,

    /// Multiplier for trade values (optional)
    #[arg(short = 'x', long, default_value_t = 1.0)]
    multiplier: f64,

    /// Option to generate and save a histogram of final account balances
    #[arg(long, default_value_t = false)]
    histogram: bool,
    
    /// File path to save the histogram image
    #[arg(long, default_value = "final_balances_histogram.png")]
    histogram_file: String,
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
    // Initialize the logger with a default log level of "info"
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("Starting the Prop Simulator");

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
    let trades = if let Some(csv_file) = cli.csv_file {
        // If CSV file is provided, read trades from the CSV
        read_csv(&csv_file, cli.multiplier)?
    } else {
        // Otherwise, generate trades using the provided parameters
        let stop_loss = cli.stop_loss.ok_or("Not using csv. Stop loss is required for simulated bracket trades")?;
        let take_profit = cli.take_profit.ok_or("Not using csv. Take profit is required for simulated bracket trades")?;
        let win_percentage = cli.win_percentage.ok_or("Not using csv. Win percentage is required for simulated bracket trades")?;
        let avg_trades_per_day = cli.avg_trades_per_day.ok_or("Not using csv. Average trades per day is required for simulated bracket trades")?;

        generate_simulated_trades(
            avg_trades_per_day,
            stop_loss,
            take_profit,
            win_percentage,
            cli.multiplier
        )
    };

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

    // Compute mean balance
    let mean_balance: f64 = final_balances.iter().sum::<f64>() / final_balances.len() as f64;

    // Compute standard deviation
    let variance: f64 = final_balances.iter()
        .map(|balance| (balance - mean_balance).powi(2))
        .sum::<f64>() / final_balances.len() as f64;
    let std_dev = variance.sqrt();


    // Compute Mean Absolute Deviation (MAD)
    let mad: f64 = final_balances.iter()
        .map(|balance| (balance - mean_balance).abs())
        .sum::<f64>() / final_balances.len() as f64;

    // Compute Interquartile Range (IQR)
    let mut sorted_balances = final_balances.clone();
    sorted_balances.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q1 = sorted_balances[sorted_balances.len() / 4];
    let q3 = sorted_balances[3 * sorted_balances.len() / 4];
    let iqr = q3 - q1;

    // Compute Median Absolute Deviation (MAD-Median)
    let median_balance = sorted_balances[sorted_balances.len() / 2];
    let mad_median = {
        let mut deviations: Vec<f64> = sorted_balances.iter()
            .map(|&balance| (balance - median_balance).abs())
            .collect();
        deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        deviations[deviations.len() / 2]
    };

    println!("Median final bank account balance: {}", median_balance);
    println!("Mean final bank account balance: {}", mean_balance);
    println!("Standard deviation of final balances: {}", std_dev);
    println!("Mean Absolute Deviation (MAD): {}", mad);
    println!("Interquartile Range (IQR): {}", iqr);
    println!("Median Absolute Deviation (MAD-Median): {}", mad_median);

    // Plot histogram if enabled
    if cli.histogram {
        plot_histogram(&final_balances, &cli.histogram_file)?;
        println!("Histogram saved to {}", cli.histogram_file);
    }

    Ok(())
}
