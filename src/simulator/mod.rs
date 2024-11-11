// src/simulator/lib.rs
pub mod trade_data;
pub mod prop_account;
pub mod trader;
pub mod plotting;

#[allow(unused_imports)]
use prop_account::AccountType;
use serde::{Serialize, Deserialize};
use trade_data::read_csv_from_string;
pub use trade_data::{read_csv, calculate_trades_per_day, generate_simulated_trades, TradeRecord};
pub use prop_account::ftt_account::FttAccountType;
pub use trader::{Trader, EndOfGame};
pub use plotting::plot_histogram;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use log::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub csv_file: Option<String>,
    pub csv_data: Option<String>,
    pub iterations: usize,
    pub max_trades_per_day: Option<u64>,
    pub daily_profit_target: Option<f64>,
    pub daily_stop_loss: Option<f64>,
    pub round_trip_cost: Option<f64>,
    pub avg_trades_per_day: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub win_percentage: Option<f64>,
    pub max_simulation_days: u64,
    pub max_payouts: u8,
    pub account_type: String,
    pub multiplier: f64,
    pub histogram: bool,
    pub histogram_file: Option<String>,
    pub condition_end_state: String,
}

#[derive(Debug, Serialize)]
pub struct SimulationResult {
    #[serde(skip_serializing)]
    pub final_balances: Vec<f64>,
    pub mean_balance: f64,
    pub median_balance: f64,
    pub std_dev: f64,
    pub mad: f64,
    pub iqr: f64,
    pub mad_median: f64,
    pub mean_days: f64,
    pub end_state_percentages: HashMap<EndOfGame, f64>,
    pub positive_balance_percentage: f64, 
    #[cfg(feature = "web")]
    pub histogram_plotly_json: Option<String>,
}

#[derive(Debug)]
struct IterationResult {
    final_balance: f64,
    end_state: EndOfGame,
    simulation_length: u64,
}

pub fn run_simulation(config: SimulationConfig) -> Result<SimulationResult, Box<dyn Error>> {
    // Initialize logging if not already initialized (optional)
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting the Prop Simulator with simulation config: {:?}", config.clone());
    // Clone the account type for use in the simulation
    let account_type = AccountType::from_str(&config.account_type)
        .map_err(|_| "Invalid account type format")?;

    info!("Running simulation with account type: {:?}", account_type);

    // Load or generate trades based on the provided configuration
    let trades = if let Some(csv_data) = &config.csv_data {
        // Read trades from CSV data
        read_csv_from_string(csv_data, config.multiplier, config.round_trip_cost)?
    } else if let Some(csv_file) = &config.csv_file {
        read_csv(csv_file, config.multiplier, config.round_trip_cost)?
    } else {
        let stop_loss = config.stop_loss.ok_or("Stop loss required")?;
        let take_profit = config.take_profit.ok_or("Take profit required")?;
        let win_percentage = config.win_percentage.ok_or("Win percentage required")?;
        let avg_trades_per_day = config.avg_trades_per_day.ok_or("Avg trades per day required")?;

        generate_simulated_trades(
            avg_trades_per_day,
            stop_loss,
            take_profit,
            win_percentage,
            config.multiplier,
            config.round_trip_cost,
        )
    };

    // Calculate the number of trades per day
    let trades_per_day_map = calculate_trades_per_day(&trades);
    let trades_per_day: Vec<usize> = trades_per_day_map.values().cloned().collect();

    // Run the Monte Carlo simulation
    let simulation_results = monte_carlo_simulation(
        &trades,
        &trades_per_day,
        config.iterations,
        account_type,
        config.max_trades_per_day,
        config.daily_profit_target,
        config.daily_stop_loss,
        config.max_simulation_days,
        config.max_payouts,
    );

    // Process the simulation results
    let mut final_balances = Vec::new();
    let mut aggregate_days = Vec::new();
    let mut balances_by_end_state = HashMap::new();
    let mut days_by_end_state = HashMap::new();
    let mut end_state_counts = HashMap::new();

    for result in &simulation_results {
        final_balances.push(result.final_balance);
        aggregate_days.push(result.simulation_length);
        *end_state_counts.entry(result.end_state.clone()).or_insert(0) += 1;
        balances_by_end_state
            .entry(result.end_state.clone())
            .or_insert_with(Vec::new)
            .push(result.final_balance);
        days_by_end_state
            .entry(result.end_state.clone())
            .or_insert_with(Vec::new)
            .push(result.simulation_length);
    }

    // Compute the percentage of each end state
    let mut end_state_percentages = HashMap::new();
    for (end_state, count) in &end_state_counts {
        let percentage = (*count as f64 / config.iterations as f64) * 100.0;
        end_state_percentages.insert(end_state.clone(), percentage);
    }

    // Determine the target end state for conditioned statistics
    let target_end_state = match config.condition_end_state.to_lowercase().as_str() {
        "busted" => Some(EndOfGame::Busted),
        "timeout" => Some(EndOfGame::TimeOut),
        "maxpayouts" => Some(EndOfGame::MaxPayouts),
        "all" => None,
        _ => {
            eprintln!(
                "Invalid end state condition '{}'. Using default aggregate data.",
                config.condition_end_state
            );
            None
        }
    };

    // Filter data based on the target end state
    let (filtered_balances, filtered_days) = if let Some(end_state) = target_end_state {
        (
            balances_by_end_state.get(&end_state).cloned().unwrap_or_default(),
            days_by_end_state.get(&end_state).cloned().unwrap_or_default(),
        )
    } else {
        (final_balances.clone(), aggregate_days.clone())
    };

    // Check if there is data to process
    if filtered_balances.is_empty() {
        return Err("No data available for the specified condition_end_state.".into());
    }

    // Calculate aggregate statistics
    let mean_balance: f64 = filtered_balances.iter().sum::<f64>() / filtered_balances.len() as f64;
    let mean_days: f64 = filtered_days.iter().sum::<u64>() as f64 / filtered_days.len() as f64;

    let variance: f64 = filtered_balances
        .iter()
        .map(|balance| (balance - mean_balance).powi(2))
        .sum::<f64>()
        / filtered_balances.len() as f64;
    let std_dev = variance.sqrt();

    let mad: f64 = filtered_balances
        .iter()
        .map(|balance| (balance - mean_balance).abs())
        .sum::<f64>()
        / filtered_balances.len() as f64;

    let mut sorted_balances = filtered_balances.clone();
    sorted_balances.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let median_balance = if sorted_balances.len() % 2 == 0 {
        let mid = sorted_balances.len() / 2;
        (sorted_balances[mid - 1] + sorted_balances[mid]) / 2.0
    } else {
        sorted_balances[sorted_balances.len() / 2]
    };

    let q1_index = sorted_balances.len() / 4;
    let q3_index = 3 * sorted_balances.len() / 4;
    let q1 = sorted_balances[q1_index];
    let q3 = sorted_balances[q3_index];
    let iqr = q3 - q1;

    let mut deviations: Vec<f64> = sorted_balances
        .iter()
        .map(|&balance| (balance - median_balance).abs())
        .collect();
    deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mad_median = if deviations.len() % 2 == 0 {
        let mid = deviations.len() / 2;
        (deviations[mid - 1] + deviations[mid]) / 2.0
    } else {
        deviations[deviations.len() / 2]
    };

    // Compute the percentage of positive balances
    let positive_balances_count = filtered_balances.iter().filter(|&&b| b > 0.0).count();
    let positive_balance_percentage = (positive_balances_count as f64 / filtered_balances.len() as f64) * 100.0;


    // Optionally generate and save a histogram

    #[cfg(feature = "web")]
    let mut histogram_plotly_json  = None;

    if config.histogram {
        #[cfg(feature = "web")]
        {
            let plot_json = plotting::generate_plotly_histogram_json(&filtered_balances)?;
            histogram_plotly_json = Some(plot_json);
            info!("Histogram generated using Plotly");
        }
        #[cfg(feature = "cli")]
        {
            if let Some(ref histogram_file) = config.histogram_file {
                plot_histogram(&filtered_balances, histogram_file)?;
                info!("Histogram saved to {}", histogram_file);
            } else {
                return Err("Histogram file path is required when histogram is enabled".into());
            }
        }
    }

    // Return the simulation result
    Ok(SimulationResult {
        final_balances: filtered_balances,
        mean_balance,
        median_balance,
        std_dev,
        mad,
        iqr,
        mad_median,
        mean_days,
        end_state_percentages,
        positive_balance_percentage,
        #[cfg(feature = "web")]
        histogram_plotly_json,   // Included in JSON response
    })
}

// Helper function to run the Monte Carlo simulation
fn monte_carlo_simulation(
    trades: &Vec<TradeRecord>,
    trades_per_day: &Vec<usize>,
    iterations: usize,
    account_type: AccountType,
    max_trades_per_day: Option<u64>,
    daily_profit_target: Option<f64>,
    daily_stop_loss: Option<f64>,
    max_simulation_days: u64,
    max_payouts: u8,
) -> Vec<IterationResult> {
    (0..iterations)
        .into_par_iter()
        .map(|_| {
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
                let num_trades_today = *trades_per_day.choose(&mut rng).unwrap_or(&0);
                let trades_today: Vec<_> = (0..num_trades_today)
                    .map(|_| trades.choose(&mut rng).unwrap().trade.clone())
                    .collect();

                let trading_day_result = trader.trade_day(&mut trades_today.clone());

                if let Some(end_of_game) = trading_day_result.end_of_game {
                    break end_of_game;
                }
            };

            IterationResult {
                final_balance: trader.bank_account.balance,
                end_state,
                simulation_length: trader.prop_account.get_simulation_days(),
            }
    }).collect()
}
