use std::str::FromStr;

use super::{AccountStatus, PropAccount};
use crate::simulator::trade_data::Trade;
use log::debug;
use serde::{Serialize, Deserialize};

// Enum for FTT account types and their rule sets
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum TopstepAccountType {
    Fifty,
    OneHundred,
    OneFifty,
}

const WINNING_DAY_TOPSTEP: f64 = 200.0;
const TOPSTED_CONSISTENCY_FRACTION: f64 = 0.5;
const XFA_COST: f64 = 149.0;

impl TopstepAccountType {

    fn initialize_account(&self) -> TopstepAccount{
        match self {
            TopstepAccountType::Fifty => {
                TopstepAccount{
                    current_balance: 0.0,
                    hwm_balance: 0.0,
                    drawdown: 2_000.0,
                    profit_target: 3_000.0,
                    loss_balance: -2_000.0,
                    simulation_days: 0,
                    winning_days_since_last_payout: 0,
                    total_winning_days: 0,
                    passed_eval: false,
                    max_winning_day_profit: 0.0,
                    account_type: TopstepAccountType::Fifty,
                }
            },
            TopstepAccountType::OneHundred => {
                TopstepAccount{
                    current_balance: 0.0,
                    hwm_balance: 0.0,
                    drawdown: 3_000.0,
                    profit_target: 6_000.0,
                    loss_balance: -3_000.0,
                    simulation_days: 0,
                    winning_days_since_last_payout: 0,
                    total_winning_days: 0,
                    passed_eval: false,
                    max_winning_day_profit: 0.0,
                    account_type: TopstepAccountType::OneHundred,
                }
            },
            TopstepAccountType::OneFifty => {
                TopstepAccount{
                    current_balance: 0.0,
                    hwm_balance: 0.0,
                    drawdown: 4_500.0,
                    profit_target: 9_000.0,
                    loss_balance: -4_500.0,
                    simulation_days: 0,
                    winning_days_since_last_payout: 0,
                    total_winning_days: 0,
                    passed_eval: false,
                    max_winning_day_profit: 0.0,
                    account_type: TopstepAccountType::OneFifty,
                }
            },
        }
    }

    // Function to return the cost of each account type
    pub fn get_cost(&self) -> f64 {
        match self {
            TopstepAccountType::Fifty => 49.0,
            TopstepAccountType::OneHundred => 99.0,
            TopstepAccountType::OneFifty => 149.0,
        }
    }

    pub fn funded_acct_cost() -> f64{
        return XFA_COST
    }
}

impl FromStr for TopstepAccountType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fifty" => Ok(TopstepAccountType::Fifty),
            "onehundred" => Ok(TopstepAccountType::OneHundred),
            "onefifty" => Ok(TopstepAccountType::OneFifty),
            _ => Err("Unknown Topstep account type"),
        }
    }
}

#[derive(Debug)]
pub struct TopstepAccount {
    current_balance: f64,        // current balance
    hwm_balance: f64,           //high water mark
    profit_target: f64,
    drawdown: f64,          //drawdown  == profit target
    loss_balance: f64,   // accounts for max loss limit / drawdown allowance (Drawdown updates EOD, stops at initial balance. max loss is intraday)
    winning_days_since_last_payout: u32,
    total_winning_days: u32, //total winning days
    max_winning_day_profit: f64,
    passed_eval: bool,
    simulation_days: u64, //every 30 simulation days not in xfa incurs cost
    account_type: TopstepAccountType,
}

impl TopstepAccount {
    pub fn new(account_type: TopstepAccountType) -> Self {
        account_type.initialize_account()
    }

    pub fn trade_on_combine(&mut self, trade: &Trade) -> AccountStatus{
        if trade.return_value > 0.0 {
            if self.current_balance + trade.max_opposite_excursion <= self.loss_balance{
                //trade would have won but mae blew us out
                self.current_balance += trade.max_opposite_excursion;
                return AccountStatus::Blown(trade.max_opposite_excursion);
            }
            else{
                self.current_balance += trade.return_value;
                if self.current_balance >= self.profit_target {
                    self.current_balance = self.profit_target;
                    self.passed_eval = true;
                    return AccountStatus::PassedEval;
                }
                return  AccountStatus::Active(trade.return_value);
            }
        }
        else{
            if self.current_balance + trade.return_value <= self.loss_balance{
                self.current_balance += trade.return_value;
                return AccountStatus::Blown(trade.return_value);
            }
            else if self.current_balance + trade.max_opposite_excursion >= self.profit_target{
                self.current_balance = self.profit_target;
                self.passed_eval = true;
                return AccountStatus::PassedEval;                
            } else {
                self.current_balance += trade.return_value;
                return AccountStatus::Active(trade.return_value);
            }
        }
    }

    pub fn trade_on_account(&mut self, trade: &Trade) -> AccountStatus{
        if trade.return_value > 0.0 {
            if self.current_balance + trade.max_opposite_excursion <= self.loss_balance{
                //trade would have won but mae blew us out
                self.current_balance += trade.max_opposite_excursion;
                return AccountStatus::Blown(trade.max_opposite_excursion);
            }
            else{
                self.current_balance += trade.return_value;
                return  AccountStatus::Active(trade.return_value);
            }
        }
        else{
            if self.current_balance + trade.return_value <= self.loss_balance{
                self.current_balance += trade.return_value;
                return AccountStatus::Blown(trade.return_value);
            }
            else{
                self.current_balance += trade.return_value;
                return AccountStatus::Active(trade.return_value);
            }
        }
    }

    // Update drawdown based on the current balance (EOD)
    pub fn update_loss_balance(&mut self) {
        if self.hwm_balance < self.profit_target{
            //if havent hit profit target yet, still trailing dd
            if self.current_balance > self.hwm_balance{
                //made new hwm
                self.loss_balance = self.current_balance - self.drawdown;
                
                if self.loss_balance > 0.0{
                    self.loss_balance = 0.0;
                }
                debug!("eod trail updated. new loss balance: {}", self.loss_balance);
                self.hwm_balance = self.current_balance;
            }
        }
    }

    pub fn passes_consistency_rule(&self) -> bool{
        if self.max_winning_day_profit  > TOPSTED_CONSISTENCY_FRACTION * self.current_balance {
            return false;
        }
        true
    }

    pub fn allowed_withdrawal_amount(&self) -> Option<f64>{
        if self.total_winning_days >= 30{
            return Some(self.current_balance);
        } else if self.winning_days_since_last_payout >= 5{
            return Some(self.current_balance * 0.5);
        }
        else{
            return None;
        }
    }

    pub fn make_withdrawal(&mut self, amount: f64) -> u8 {
        self.current_balance -= amount;
        self.max_winning_day_profit = 0.0; //TODO: is this reset every withdrawal?
        self.winning_days_since_last_payout = 0;
        if self.current_balance <= 0.01{
            return 1; //end of game for topstep account
        }
        else{
            return 0;
        }
    }

    pub fn try_add_trading_day(&mut self, daily_pnl: f64){
        
        if self.passed_eval{
            if daily_pnl >= WINNING_DAY_TOPSTEP {
                self.total_winning_days += 1;
                self.winning_days_since_last_payout += 1;

            }
            if daily_pnl > self.max_winning_day_profit{
                self.max_winning_day_profit = daily_pnl;
            }
        }
    }
}

impl PropAccount for TopstepAccount {
    fn process_trade(&mut self, trade: &Trade) -> AccountStatus {
        if !self.passed_eval {
            // During the combine phase
            self.trade_on_combine(trade)
        } else {
            // Live trading
            self.trade_on_account(trade)
        }
    }

    fn update_end_of_day(&mut self, daily_pnl: f64) {
        self.update_loss_balance();
        self.try_add_trading_day(daily_pnl);
    }

    fn allowed_withdrawal_amount(&self) -> Option<f64> {
        if self.passed_eval {
            self.allowed_withdrawal_amount()
        } else {
            None
        }
    }

    fn make_withdrawal(&mut self, amount: f64) -> u8 {
        self.make_withdrawal(amount)
    }

    fn get_current_balance(&self) -> f64 {
        self.current_balance
    }

    fn get_simulation_days(&self) -> u64 {
        self.simulation_days
    }

    fn increment_simulation_day(&mut self) {
        self.simulation_days += 1;
    }

    fn get_cost(&self) -> f64 {
        self.account_type.get_cost()
    }
    fn get_funded_acct_cost(&self)-> f64 {
        TopstepAccountType::funded_acct_cost()
    }
}
