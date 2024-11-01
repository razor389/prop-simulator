use std::{cmp::max, collections::HashMap, error::Error};
use csv::Reader;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use rand::Rng;
use rand_distr::{Poisson, Normal, Distribution};

#[derive(Debug, Clone)]
pub struct Trade{
    pub return_value: f64,
    pub max_opposite_excursion: f64,
}

// Struct to store the data from the CSV
#[derive(Debug)]
pub struct TradeRecord {
    datetime: DateTime<Utc>,
    pub trade: Trade,
}

// Function to read and parse the CSV file
pub fn read_csv(file_path: &str, multiplier: f64, round_trip_cost: f64) -> Result<Vec<TradeRecord>, Box<dyn Error>> {
    let mut rdr = Reader::from_path(file_path)?;
    let mut trades = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let datetime_str = &record[0];
        let naive = NaiveDateTime::parse_from_str(datetime_str, "%Y%m%d %H:%M:%S")?;
        // Using TimeZone::from_utc_datetime
        let datetime = Utc.from_utc_datetime(&naive);
        let return_value: f64 = record[1].parse()?;
        let max_opposite_excursion: f64 = record[2].parse()?;

        trades.push(TradeRecord {
            datetime,
            trade: Trade{
                return_value: return_value*multiplier - round_trip_cost,
                max_opposite_excursion: max_opposite_excursion*multiplier - round_trip_cost
            },
        });
    }

    Ok(trades)
}

// Function to read and parse CSV data from a string
pub fn read_csv_from_string(data: &str, multiplier: f64, round_trip_cost: f64) -> Result<Vec<TradeRecord>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(data.as_bytes());
    let mut trades = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let datetime_str = &record[0];
        let naive = NaiveDateTime::parse_from_str(datetime_str, "%Y%m%d %H:%M:%S")?;
        let datetime = Utc.from_utc_datetime(&naive);
        let return_value: f64 = record[1].parse()?;
        let max_opposite_excursion: f64 = record[2].parse()?;

        trades.push(TradeRecord {
            datetime,
            trade: Trade {
                return_value: return_value * multiplier - round_trip_cost,
                max_opposite_excursion: max_opposite_excursion * multiplier - round_trip_cost,
            },
        });
    }

    Ok(trades)
}

#[allow(dead_code)]
// Function to generate simulated trades using Poisson distribution and win percentage
pub fn generate_simulated_trades(
    avg_trades_per_day: f64,
    stop_loss: f64,
    take_profit: f64,
    win_percentage: f64,
    multiplier: f64,
    round_trip_cost: f64,
) -> Vec<TradeRecord> {
    let mut rng = rand::thread_rng();
    let poisson = Poisson::new(avg_trades_per_day).unwrap();
    
    // Normal distribution for adverse excursions (MAE for wins)
    let mae_mean = stop_loss * 0.5; // Mean of adverse move (50% of stop-loss)
    let mae_stddev = stop_loss * 0.25; // Stddev of adverse move (25% of stop-loss)
    let normal_mae = Normal::new(mae_mean, mae_stddev).unwrap();

    // Normal distribution for favorable excursions (MFE for losses)
    let mfe_mean = take_profit * 0.5; // Mean of favorable move (50% of take-profit)
    let mfe_stddev = take_profit * 0.25; // Stddev of favorable move (25% of take-profit)
    let normal_mfe = Normal::new(mfe_mean, mfe_stddev).unwrap();

    let start_date = Utc.with_ymd_and_hms(2024, 1, 1, 9, 30, 0).unwrap();

    let mut trades = Vec::new();

    for day in 0..365 { // Simulating 365 days
        let num_trades_today = poisson.sample(&mut rng) as usize;
        for _ in 0..num_trades_today {
            let datetime = start_date + chrono::Duration::days(day);

            // Randomly determine if the trade is a win or a loss based on win_percentage
            let win = rng.gen_bool(win_percentage / 100.0);
            let (return_value, max_opposite_excursion) = if win {
                // Winning trade: use adverse move for max_opposite_excursion
                let mae = normal_mae.sample(&mut rng).abs().min(stop_loss); // Cap MAE at stop-loss
                (take_profit * multiplier, mae * multiplier) // Take profit is the return value
            } else {
                // Losing trade: use favorable move for max_opposite_excursion
                let mfe = normal_mfe.sample(&mut rng).abs().min(take_profit); // Cap MFE at take-profit
                (-1.0 * stop_loss * multiplier, mfe * multiplier) // Stop loss is the return value (loss)
            };

            trades.push(TradeRecord {
                datetime,
                trade: Trade {
                    return_value: return_value - round_trip_cost,
                    max_opposite_excursion: max_opposite_excursion - round_trip_cost,
                },
            });
        }
    }
    //println!("{:#?}", trades);
    trades
}

// Group trades by day and calculate the number of trades per day
pub fn calculate_trades_per_day(trades: &Vec<TradeRecord>) -> HashMap<NaiveDate, usize> {
    let mut trades_per_day = HashMap::new();

    for trade in trades {
        let date = trade.datetime.date_naive(); // Get the date without time component
        *trades_per_day.entry(date).or_insert(0) += 1; // Increment count of trades for this date
    }

    trades_per_day
}
