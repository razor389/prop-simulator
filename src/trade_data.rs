use std::{collections::HashMap, error::Error};
use csv::Reader;
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};

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
pub fn read_csv(file_path: &str, multiplier: f64) -> Result<Vec<TradeRecord>, Box<dyn Error>> {
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
                return_value: return_value*multiplier,
                max_opposite_excursion: max_opposite_excursion*multiplier
            },
        });
    }

    Ok(trades)
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
