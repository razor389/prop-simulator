pub mod ftt_account;
pub mod topstep_account;
// Add other account modules here...
pub mod account_type;

use crate::simulator::trade_data::Trade;

#[derive(Debug)]
pub enum AccountStatus {
    Blown(f64),
    Active(f64),
    PassedEval,
}

pub trait PropAccount {
    fn process_trade(&mut self, trade: &Trade) -> AccountStatus;
    fn update_end_of_day(&mut self, daily_pnl: f64);
    fn allowed_withdrawal_amount(&self) -> Option<f64>;
    fn make_withdrawal(&mut self, amount: f64) -> u8;
    fn get_current_balance(&self) -> f64;
    fn get_simulation_days(&self) -> u64;
    fn increment_simulation_day(&mut self);
    fn get_cost(&self) -> f64;
    fn get_funded_acct_cost(&self)-> f64;
}

// Re-export account structs
pub use ftt_account::{FttAccount, FttAccountType};
pub use topstep_account::{TopstepAccount, TopstepAccountType};
pub use account_type::AccountType;
// Add other account re-exports here...


pub fn create_account(account_type: AccountType) -> Box<dyn PropAccount + Send + Sync> {
    match account_type {
        AccountType::Ftt(ftt_type) => Box::new(FttAccount::new(ftt_type)),
        AccountType::TopStep(topstep_type) => Box::new(TopstepAccount::new(topstep_type)),
        // Handle other companies...
    }
}
