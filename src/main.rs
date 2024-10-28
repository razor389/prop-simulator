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
use log::info;

mod trade_data;
mod ftt_account;
mod trader;
mod plotting;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'f', long)]
    csv_file: Option<String>,
    #[arg(short, long, default_value_t = 10000)]
    iterations: usize,
    #[arg(short = 't', long)]
    max_trades_per_day: Option<u64>,
    #[arg(short = 'p', long)]
    daily_profit_target: Option<f64>,
    #[arg(short = 's', long)]
    daily_stop_loss: Option<f64>,
    #[arg(short = 'a', long)]
    avg_trades_per_day: Option<f64>,
    #[arg(long)]
    stop_loss: Option<f64>,
    #[arg(long)]
    take_profit: Option<f64>,
    #[arg(long)]
    win_percentage: Option<f64>,
    #[arg(short = 'd', long, default_value_t = 365)]
    max_simulation_days: u64,
    #[arg(short = 'm', long, default_value_t = 12)]
    max_payouts: u8,
    #[arg(short = 'c', long, default_value_t = String::from("GT"))]
    account_type: String,
    #[arg(short = 'x', long, default_value_t = 1.0)]
    multiplier: f64,
    #[arg(long, default_value_t = false)]
    histogram: bool,
    #[arg(long, default_value = "final_balances_histogram.png")]
    histogram_file: String,
    
    /// Condition aggregate statistics based on end state (options: "Busted", "TimeOut", "MaxPayouts", "All")
    #[arg(long, default_value = "All")]
    condition_end_state: String,
}

#[derive(Debug)]
struct SimulationResult {
    final_balance: f64,
    end_state: trader::EndOfGame,
    simulation_length: u64,
}

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
) -> Vec<SimulationResult> {
    (0..iterations).into_par_iter().map(|_| {
        let mut rng = rand::thread_rng();
        let mut trader = Trader::new(
            account_type.clone(),
            max_trades_per_day,
            daily_profit_target,
            daily_stop_loss,
            max_simulation_days,
            max_payouts,
        );

        let end_state = loop {
            let num_trades_today = *trades_per_day.choose(&mut rng).unwrap();
            let trades_today: Vec<_> = (0..num_trades_today)
                .map(|_| trades.choose(&mut rng).unwrap().trade.clone())
                .collect();

            let trading_day_result = trader.trade_day(&mut trades_today.clone());

            if let Some(end_of_game) = trading_day_result.end_of_game {
                break end_of_game;
            }
        };

        SimulationResult {
            final_balance: trader.bank_account.balance,
            end_state,
            simulation_length: trader.ftt_account.simulation_days,
        }
    }).collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting the Prop Simulator");

    let cli = Cli::parse();

    let account_type = match cli.account_type.as_str() {
        "Rally" => FttAccountType::Rally,
        "Daytona" => FttAccountType::Daytona,
        "LeMans" => FttAccountType::LeMans,
        _ => FttAccountType::GT,
    };

    let trades = if let Some(csv_file) = cli.csv_file {
        read_csv(&csv_file, cli.multiplier)?
    } else {
        let stop_loss = cli.stop_loss.ok_or("Stop loss required")?;
        let take_profit = cli.take_profit.ok_or("Take profit required")?;
        let win_percentage = cli.win_percentage.ok_or("Win percentage required")?;
        let avg_trades_per_day = cli.avg_trades_per_day.ok_or("Avg trades per day required")?;

        generate_simulated_trades(
            avg_trades_per_day,
            stop_loss,
            take_profit,
            win_percentage,
            cli.multiplier
        )
    };

    let trades_per_day_map = calculate_trades_per_day(&trades);
    let trades_per_day: Vec<usize> = trades_per_day_map.values().cloned().collect();

    let simulation_results = monte_carlo_simulation(
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

    let mut final_balances = Vec::new();
    let mut aggregate_days = Vec::new();
    let mut balances_by_end_state = std::collections::HashMap::new();
    let mut days_by_end_state = std::collections::HashMap::new();
    let mut end_state_counts = std::collections::HashMap::new();

    for result in &simulation_results {
        final_balances.push(result.final_balance);
        aggregate_days.push(result.simulation_length);
        *end_state_counts.entry(result.end_state.clone()).or_insert(0) += 1;
        balances_by_end_state.entry(result.end_state.clone()).or_insert(Vec::new()).push(result.final_balance);
        days_by_end_state.entry(result.end_state.clone()).or_insert(Vec::new()).push(result.simulation_length);
    }

    // Display the percentage of each end state
    for (end_state, count) in &end_state_counts {
        println!(
            "End State: {:?}, Percentage: {:.2}%",
            end_state,
            (*count as f64 / cli.iterations as f64) * 100.0
        );
    }

    // Aggregate or Conditioned Data Selection
    // Determine the target end state (or None for unconditioned statistics)
    let target_end_state = match cli.condition_end_state.to_lowercase().as_str() {
        "busted" => Some(trader::EndOfGame::Busted),
        "timeout" => Some(trader::EndOfGame::TimeOut),
        "maxpayouts" => Some(trader::EndOfGame::MaxPayouts),
        "all" => None,
        _ => {
            eprintln!("Invalid end state condition '{}'. Using default aggregate data.", cli.condition_end_state);
            None
        }
    };

    // Choose filtered data based on the target end state
    let (filtered_balances, filtered_days) = if let Some(end_state) = target_end_state {
        (
            balances_by_end_state.get(&end_state).cloned().unwrap_or_default(),
            days_by_end_state.get(&end_state).cloned().unwrap_or_default(),
        )
    } else {
        (final_balances.clone(), aggregate_days.clone())
    };

    // Aggregate Statistics or Conditioned Statistics Based on Filtered Data
    let mean_balance: f64 = filtered_balances.iter().sum::<f64>() / filtered_balances.len() as f64;
    let mean_days: f64 = filtered_days.iter().sum::<u64>() as f64 / filtered_days.len() as f64;

    let variance: f64 = filtered_balances.iter()
        .map(|balance| (balance - mean_balance).powi(2))
        .sum::<f64>() / filtered_balances.len() as f64;
    let std_dev = variance.sqrt();

    let mad: f64 = filtered_balances.iter()
        .map(|balance| (balance - mean_balance).abs())
        .sum::<f64>() / filtered_balances.len() as f64;

    let mut sorted_balances = filtered_balances.clone();
    sorted_balances.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q1 = sorted_balances[sorted_balances.len() / 4];
    let q3 = sorted_balances[3 * sorted_balances.len() / 4];
    let iqr = q3 - q1;

    let median_balance = sorted_balances[sorted_balances.len() / 2];
    let mad_median = {
        let mut deviations: Vec<f64> = sorted_balances.iter()
            .map(|&balance| (balance - median_balance).abs())
            .collect();
        deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        deviations[deviations.len() / 2]
    };

    println!("Statistics Conditioned on End State '{}':", cli.condition_end_state);
    println!("Mean Simulation Length: {:.2} days", mean_days);
    println!("Median Final Bank Balance: {:.2}", median_balance);
    println!("Mean Final Bank Balance: {:.2}", mean_balance);
    println!("Standard Deviation of Final Bank Balances: {:.2}", std_dev);
    println!("Mean Absolute Deviation: {:.2}", mad);
    println!("Interquartile Range: {:.2}", iqr);
    println!("Median Absolute Deviation: {:.2}", mad_median);

    if cli.histogram {
        plot_histogram(&filtered_balances, &cli.histogram_file)?;
        println!("Histogram saved to {}", cli.histogram_file);
    }

    Ok(())
}
