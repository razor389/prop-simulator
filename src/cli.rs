use std::error::Error;
use clap::Parser;
use env_logger::Env;
use log::info;
use prop_simulator::simulator;
use simulator::{SimulationConfig, run_simulation, FttAccountType, plot_histogram};

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

// src/main.rs

pub fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("Starting the Prop Simulator");

    let cli = Cli::parse();

    // Map CLI arguments to SimulationConfig
    let account_type = match cli.account_type.as_str() {
        "Rally" => FttAccountType::Rally,
        "Daytona" => FttAccountType::Daytona,
        "LeMans" => FttAccountType::LeMans,
        _ => FttAccountType::GT,
    };

    let config = SimulationConfig {
        csv_file: cli.csv_file,
        csv_data: None,
        iterations: cli.iterations,
        max_trades_per_day: cli.max_trades_per_day,
        daily_profit_target: cli.daily_profit_target,
        daily_stop_loss: cli.daily_stop_loss,
        avg_trades_per_day: cli.avg_trades_per_day,
        stop_loss: cli.stop_loss,
        take_profit: cli.take_profit,
        win_percentage: cli.win_percentage,
        max_simulation_days: cli.max_simulation_days,
        max_payouts: cli.max_payouts,
        account_type,
        multiplier: cli.multiplier,
        histogram: cli.histogram,
        histogram_file: Some(cli.histogram_file.clone()),
        condition_end_state: cli.condition_end_state.clone(),
    };

    // Run the simulation
    let result = run_simulation(config)?;

    // Display the end state percentages
    println!("\nEnd State Percentages:");
    for (end_state, percentage) in &result.end_state_percentages {
        println!("  {:?}: {:.2}%", end_state, percentage);
    }


    // Display the results
    println!("\nStatistics Conditioned on End State '{}':", cli.condition_end_state);
    println!("Mean Simulation Length: {:.2} days", result.mean_days);
    println!("Median Final Bank Balance: {:.2}", result.median_balance);
    println!("Mean Final Bank Balance: {:.2}", result.mean_balance);
    println!("Standard Deviation of Final Bank Balances: {:.2}", result.std_dev);
    println!("Mean Absolute Deviation: {:.2}", result.mad);
    println!("Interquartile Range: {:.2}", result.iqr);
    println!("Median Absolute Deviation: {:.2}", result.mad_median);

    // Handle histogram if requested
    if cli.histogram {
        plot_histogram(&result.final_balances, &cli.histogram_file)?;
        println!("Histogram saved to {}", cli.histogram_file);
    }

    Ok(())
}

