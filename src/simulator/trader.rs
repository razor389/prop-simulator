use std::str::FromStr;

use log::debug;
use serde::{Serialize, Deserialize};
use super::prop_account::{create_account, ftt_account::{FttAccount, FttAccountType}, AccountStatus, AccountType, PropAccount, TopstepAccount, TopstepAccountType};
use super::trade_data::Trade;


#[derive(Debug)]
pub struct BankAccount {
    pub balance: f64,  
}

// Struct representing the user, with a bank account and FTT account
pub struct Trader {
    pub bank_account: BankAccount,
    pub prop_account: Box<dyn PropAccount + Send + Sync>,
    max_trades_per_day: Option<u64>,    //should be positive if Some
    daily_profit_target: Option<f64>, //should be positive if Some
    daily_stop_loss: Option<f64>, //should be negative if Some
    max_simulation_days: u64,
    max_payouts: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EndOfGame {
    Busted,
    TimeOut,
    MaxPayouts,
}

#[derive(Debug)]
pub enum DailyStopTPStatus {
    StopHit,
    TPHit,
    Neither,
}

#[derive(Debug)]
pub struct TradingDayResult{
    pub end_of_game: Option<EndOfGame>,
}

impl Trader {

    // Create a new Trader by specifying only the FTT account type
    pub fn new(account_type: AccountType, 
        max_trades_per_day: Option<u64>, 
        daily_profit_target: Option<f64>, 
        daily_stop_loss: Option<f64>,
        max_simulation_days: u64,
        max_payouts: u8,
    ) -> Self {
        // Create the PropAccount based on the account type
        let prop_account: Box<dyn PropAccount + Send +Sync> = create_account(account_type);
        // Set the bank account balance to the negative cost of the FTT account
        let bank_account = BankAccount {
            balance: -prop_account.get_cost(),
        };

        //TODO: ensure stop/pt / trades per day are properly signed if Some

        // Return the new user with both accounts initialized
        Self {
            bank_account,
            prop_account,
            max_trades_per_day,
            daily_profit_target,
            daily_stop_loss,
            max_simulation_days,
            max_payouts,
        }
    }

    fn adj_trade_for_daily_stop_or_target(&self, trade: &mut Trade, daily_pnl_pretrade: f64) -> DailyStopTPStatus{
        if let Some(daily_sl) = self.daily_stop_loss{
            if trade.return_value + daily_pnl_pretrade <= daily_sl { 
                trade.return_value = daily_sl - daily_pnl_pretrade;
                return DailyStopTPStatus::StopHit;
            }
            if trade.max_opposite_excursion + daily_pnl_pretrade <= daily_sl{
                trade.return_value = daily_sl - daily_pnl_pretrade;
                return DailyStopTPStatus::StopHit;
            }
        }

        if let Some(daily_pt) = self.daily_profit_target{
            if trade.return_value + daily_pnl_pretrade >= daily_pt { 
                trade.return_value = daily_pt - daily_pnl_pretrade;
                return DailyStopTPStatus::TPHit;

            }
            if trade.max_opposite_excursion + daily_pnl_pretrade >= daily_pt{
                //being conservative here
                trade.max_opposite_excursion = trade.return_value;
                trade.return_value = daily_pt - daily_pnl_pretrade;
                return  DailyStopTPStatus::TPHit;
            }
        }
        return DailyStopTPStatus::Neither;

    }

    // given simulated trades for today, apply updates to account balance
    pub fn trade_day(&mut self, trades_today: &mut Vec<Trade>) -> TradingDayResult {

        let mut daily_pnl = 0.0;
        let mut num_trades_today = 0;

        debug!("Starting a new trading day");

        for trade in trades_today.iter_mut(){
            //for a given trade:
            if let Some(max_trades) = self.max_trades_per_day{
                if num_trades_today >= max_trades{
                    debug!("Reached max trades per day limit: {}", max_trades);
                    break;
                }
            }
            //do we adjust trade to account for daily stop/target?
            let daily_stop_tp_status = 
                self.adj_trade_for_daily_stop_or_target(trade, daily_pnl);
            //did we blow account?
            let account_status = self.prop_account.process_trade(trade);

            match account_status {
                AccountStatus::Blown(ret) =>{
                    debug!("Trade executed, return: {:.2}, cumulative daily P&L: {:.2}", ret, daily_pnl+ret);
                    debug!("Account blown during trade, daily P&L: {:.2}, trades taken: {}", daily_pnl+ret, num_trades_today+1);
                    return TradingDayResult{
                        end_of_game: Some(EndOfGame::Busted),
                    }
                },
                AccountStatus::Active(ret) =>{
                    daily_pnl += ret;
                    debug!("Trade executed, return: {:.2}, cumulative daily P&L: {:.2}", ret, daily_pnl);
                },
                AccountStatus::PassedEval =>{
                    self.bank_account.balance -= self.prop_account.get_funded_acct_cost();
                    debug!("Passed eval");
                    return TradingDayResult{
                        end_of_game: None,
                    }
                }
            }
            //didnt blow acct if we got here. did we hit daily stop/target?
            match daily_stop_tp_status {
                DailyStopTPStatus::TPHit => {
                    debug!("Daily profit target hit with P&L: {:.2}", daily_pnl); 
                    break;
                },
                DailyStopTPStatus::StopHit => {
                    debug!("Daily stop loss hit with P&L: {:.2}", daily_pnl);
                    break;
                },
                _ => (),
            }
            num_trades_today += 1;
        }
        // Update account at the end of the day
        self.prop_account.update_end_of_day(daily_pnl);
        self.prop_account.increment_simulation_day();

        // Log the bank and ccount balances at the end of the trading day
        debug!(
            "End of trading day summary: daily P&L: {:.2}, trades taken: {}, bank balance: {:.2}, FTT account balance: {:.2}",
            daily_pnl, num_trades_today, self.bank_account.balance, self.prop_account.get_current_balance()
        );
        //can we make a withdrawal?
        if let Some(amount) = self.prop_account.allowed_withdrawal_amount(){
            let num_payouts = self.prop_account.make_withdrawal(amount);
            self.bank_account.balance += amount;
            debug!("Withdrawal made: {:.2}, bank balance after withdrawal: {:.2}", amount, self.bank_account.balance);
            if num_payouts >= self.max_payouts{
                debug!("Reached max payouts: {}, ending simulation for this trader.", self.max_payouts);
                return TradingDayResult{
                    end_of_game: Some(EndOfGame::MaxPayouts),
                }
            }
        }

        if self.prop_account.get_simulation_days() >= self.max_simulation_days{
            debug!("Max simulation days reached: {}", self.max_simulation_days);
            return TradingDayResult{
                end_of_game: Some(EndOfGame::TimeOut),
            }
        }
        
        debug!("Trading day completed without hitting max payouts, max days, or blowing account.");
        return TradingDayResult{
            end_of_game: None,
        }
    }

}
