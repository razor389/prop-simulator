use crate::{ftt_account::{ AccountStatus, FttAccount, FttAccountType}, trade_data::Trade};


#[derive(Debug)]
pub struct BankAccount {
    pub balance: f64,  
}

// Struct representing the user, with a bank account and FTT account
#[derive(Debug)]
pub struct Trader {
    pub bank_account: BankAccount,
    ftt_account: FttAccount,
    max_trades_per_day: Option<u64>,    //should be positive if Some
    daily_profit_target: Option<f64>, //should be positive if Some
    daily_stop_loss: Option<f64>, //should be negative if Some
    max_simulation_days: u64,
    max_payouts: u8,
}

#[derive(Debug)]
pub enum EndOfGame{
    Busted, //blew account
    TimeOut, //hit max simulation days
    MaxPayouts, //hit max payouts
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
    pub fn new(account_type: FttAccountType, 
        max_trades_per_day: Option<u64>, 
        daily_profit_target: Option<f64>, 
        daily_stop_loss: Option<f64>,
        max_simulation_days: u64,
        max_payouts: u8,
    ) -> Self {
        // Create the FttAccount based on the account type
        let ftt_account = FttAccount::new(account_type.clone());

        // Set the bank account balance to the negative cost of the FTT account
        let bank_account = BankAccount {
            balance: -account_type.get_cost(),
        };

        //TODO: ensure stop/pt / trades per day are properly signed if Some

        // Return the new user with both accounts initialized
        Self {
            bank_account,
            ftt_account,
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

        for trade in trades_today.iter_mut(){
            //for a given trade:
            if let Some(max_trades) = self.max_trades_per_day{
                if num_trades_today >= max_trades{
                    break;
                }
            }
            //do we adjust trade to account for daily stop/target?
            let daily_stop_tp_status = 
                self.adj_trade_for_daily_stop_or_target(trade, daily_pnl);
            //did we blow account?
            match self.ftt_account.trade_on_account(trade) {
                AccountStatus::Blown =>{
                    return TradingDayResult{
                        end_of_game: Some(EndOfGame::Busted),
                    }
                },
                AccountStatus::Active(ret) =>{
                    daily_pnl += ret;
                }
            }
            //didnt blow acct if we got here. did we hit daily stop/target?
            match daily_stop_tp_status {
                DailyStopTPStatus::TPHit => break,
                DailyStopTPStatus::StopHit => break,
                _ => (),
            }
            num_trades_today += 1;
        }
        //update drawdown/max loss
        self.ftt_account.update_loss_balance();
        //was it a real trading day?
        self.ftt_account.try_add_trading_day(daily_pnl);
        //can we make a withdrawal?
        if let Some(amount) = self.ftt_account.allowed_withdrawal_amount(){
            let num_payouts = self.ftt_account.make_withdrawal(amount);
            self.bank_account.balance += amount;
            //println!("made withdrawal, bank balance: {}", self.bank_account.balance);
            if num_payouts >= self.max_payouts{
                return TradingDayResult{
                    end_of_game: Some(EndOfGame::MaxPayouts),
                }
            }
        }

        if self.ftt_account.simulation_days >= self.max_simulation_days{
            return TradingDayResult{
                end_of_game: Some(EndOfGame::TimeOut),
            }
        }
        
        return TradingDayResult{
            end_of_game: None,
        }
    }

}
