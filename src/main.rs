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
                    trader::EndOfGame::MaxPayouts => {
                        println!("won game w/ max payouts");
                        break;
                    },
                    trader::EndOfGame::TimeOut => {
                        //println!("ended game w/ timeout");
                        break;
                    }
                    trader::EndOfGame::Busted => {
                        
                        break;
                    }
                }
            }
        }

        trader.bank_account.balance
    }).collect() // Collect the final bank balances into a Vec
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load the trade records from the CSV file
    let multiplier = 20.0; //for NQ, since file gives points and this is $/point
    let trades = read_csv("C:/Users/Raziel/Downloads/sample_trades.csv", multiplier)?;

    // Group trades by day and get the distribution of trades per day
    let trades_per_day_map = calculate_trades_per_day(&trades);

    // Convert the HashMap values into a Vec for easy random sampling
    let trades_per_day: Vec<usize> = trades_per_day_map.values().cloned().collect();

    // Simulation parameters
    let iterations = 10000;
    let max_trades_per_day = None; // Set or use None if not limiting trades per day
    let daily_profit_target = None; // Set a daily profit target (optional)
    let daily_stop_loss = None; // Set a daily stop-loss limit (optional)
    let max_simulation_days = 365; // Number of max trading days to simulate
    let max_payouts = 12; // Number of payouts before ending the simulation

    // Run Monte Carlo simulation
    let final_balances = monte_carlo_simulation(
        &trades,
        &trades_per_day,
        iterations,
        FttAccountType::GT, // Example: GT account
        max_trades_per_day,
        daily_profit_target,
        daily_stop_loss,
        max_simulation_days,
        max_payouts,
    );

    // Compute statistics from the results (e.g., mean balance)
    let mean_balance: f64 = final_balances.iter().sum::<f64>() / final_balances.len() as f64;
    println!("Mean final bank account balance: {}", mean_balance);

    Ok(())
}
