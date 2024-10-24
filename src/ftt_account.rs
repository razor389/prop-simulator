use crate::trade_data::Trade;

#[derive(Debug)]
pub struct RealTradingDay{
    min_win: f64,
    min_loss: f64,
}

#[derive(Debug)]
struct PayoutCap{
    first_8_payouts: f64,
    payouts_9_to_12: f64,
}

impl RealTradingDay{
    fn new(min_loss: f64, min_win: f64) -> Self{
        RealTradingDay{min_win, min_loss}
    }

    pub fn was_rtd(&self, daily_return: f64) -> bool{
        (daily_return > self.min_win) || (daily_return < self.min_loss)
    }
}

// Enum for FTT account types and their rule sets
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FttAccountType {
    Rally,
    Daytona,
    GT,
    LeMans,
}

impl FttAccountType {

    fn initialize_account(&self) -> FttAccount{
        match self {
            FttAccountType::Rally => {
                FttAccount{
                    current_balance: 0.0,
                    hwm_balance: 0.0,
                    drawdown: 1_250.0,
                    loss_balance: -1_250.0,
                    payout_cap: PayoutCap { first_8_payouts: 1_500.0, payouts_9_to_12: 3_000.0 },
                    real_trading_day: RealTradingDay::new(-62.5, 62.5),
                    payout_count: 0,
                    min_balance_to_withdraw_first_payout: 1_500.0,
                    min_balance_to_withdraw_subsequent_payouts: 1_500.0,
                    min_balance_after_withdrawal: 1_250.0,
                    max_winning_day_profit: 0.0,
                    trading_days: 0,
                    simulation_days: 0,
                }
            },
            FttAccountType::Daytona => {
                FttAccount{
                    current_balance: 0.0,
                    hwm_balance: 0.0,
                    drawdown: 2_500.0,
                    loss_balance: -2_500.0,
                    payout_cap: PayoutCap { first_8_payouts: 2_000.0, payouts_9_to_12: 4_000.0 },
                    real_trading_day: RealTradingDay::new(-125.0, 125.0),
                    payout_count: 0,
                    min_balance_to_withdraw_first_payout: 2_750.0,
                    min_balance_to_withdraw_subsequent_payouts: 2_750.0,
                    min_balance_after_withdrawal: 2_500.0,
                    max_winning_day_profit: 0.0,
                    trading_days: 0,
                    simulation_days: 0,
                }
            },
            FttAccountType::GT => {
                FttAccount{
                    current_balance: 0.0,
                    hwm_balance: 0.0,
                    drawdown: 7_500.0,
                    loss_balance: -7_500.0,
                    payout_cap: PayoutCap { first_8_payouts: 3_000.0, payouts_9_to_12: 6_000.0 },
                    real_trading_day: RealTradingDay::new(-187.5, 375.0),
                    payout_count: 0,
                    min_balance_to_withdraw_first_payout: 7_500.0,
                    min_balance_to_withdraw_subsequent_payouts: 4_750.0,
                    min_balance_after_withdrawal: 4_500.0,
                    max_winning_day_profit: 0.0,
                    trading_days: 0,
                    simulation_days: 0,
                }
            },
            FttAccountType::LeMans => {
                FttAccount{
                    current_balance: 0.0,
                    hwm_balance: 0.0,
                    drawdown: 15_000.0,
                    loss_balance: -15_000.0,
                    payout_cap: PayoutCap { first_8_payouts: 4_000.0, payouts_9_to_12: 8_000.0 },
                    real_trading_day: RealTradingDay::new(-300.0, 600.0),
                    payout_count: 0,
                    min_balance_to_withdraw_first_payout: 15_000.0,
                    min_balance_to_withdraw_subsequent_payouts: 11_250.0,
                    min_balance_after_withdrawal: 11_000.0,
                    max_winning_day_profit: 0.0,
                    trading_days: 0,
                    simulation_days: 0,
                }
            },
        }
    }

    // Function to return the cost of each account type
    pub fn get_cost(&self) -> f64 {
        match self {
            FttAccountType::Rally => 179.0,
            FttAccountType::Daytona => 449.0,
            FttAccountType::GT => 599.0,
            FttAccountType::LeMans => 799.0,
        }
    }
}


#[derive(Debug)]
pub struct FttAccount {
    current_balance: f64,        // current balance
    hwm_balance: f64,           //high water mark
    drawdown: f64,          //drawdown  == profit target
    loss_balance: f64,   // accounts for max loss limit / drawdown allowance (Drawdown updates EOD, stops at initial balance. max loss is intraday)
    payout_cap: PayoutCap,
    real_trading_day: RealTradingDay, //rtd params for account
    payout_count: u8,   // Number of successful payouts
    min_balance_to_withdraw_first_payout: f64,
    min_balance_to_withdraw_subsequent_payouts: f64,
    min_balance_after_withdrawal: f64,
    max_winning_day_profit: f64, //for consistency rule
    trading_days: u64, //since last withdrawal
    pub simulation_days: u64,
}

#[derive(Debug)]
pub enum AccountStatus{
    Blown,
    Active(f64),
}

impl FttAccount {
    pub fn new(account_type: FttAccountType) -> Self {
        account_type.initialize_account()
    }

    pub fn trade_on_account(&mut self, trade: &Trade) -> AccountStatus{
        if trade.return_value > 0.0 {
            if self.current_balance + trade.max_opposite_excursion < self.loss_balance{
                //trade would have won but mae blew us out
                self.current_balance += trade.max_opposite_excursion;
                return AccountStatus::Blown;
            }
            else{
                self.current_balance += trade.return_value;
                return  AccountStatus::Active(trade.return_value);
            }
        }
        else{
            if self.current_balance + trade.return_value < self.loss_balance{
                self.current_balance += trade.return_value;
                return AccountStatus::Blown;
            }
            else{
                self.current_balance += trade.return_value;
                return AccountStatus::Active(trade.return_value);
            }
        }
    }

    // Update drawdown based on the current balance (EOD)
    pub fn update_loss_balance(&mut self) {
        if self.hwm_balance < self.drawdown{
            //if havent hit profit target yet, still trailing dd
            if self.current_balance > self.hwm_balance{
                //made new hwm
                self.loss_balance = self.current_balance - self.drawdown;
                if self.loss_balance > 0.0{
                    self.loss_balance = 0.0;
                }
                self.hwm_balance = self.current_balance;
            }
        }
    }

    pub fn passes_consistency_rule(&self) -> bool{
        if self.max_winning_day_profit  > 0.2 * self.current_balance {
            return false;
        }
        true
    }

    pub fn allowed_withdrawal_amount(&self) -> Option<f64>{
        if self.trading_days >= 10{
            if self.payout_count == 0{
                if self.current_balance >= self.min_balance_to_withdraw_first_payout && self.passes_consistency_rule(){
                    if self.current_balance - self.min_balance_after_withdrawal > self.payout_cap.first_8_payouts{
                        return Some(self.payout_cap.first_8_payouts)
                    }
                    return Some(self.current_balance - self.min_balance_after_withdrawal);
                }
            } else{
                if self.current_balance >= self.min_balance_to_withdraw_subsequent_payouts && self.passes_consistency_rule(){
                    if self.payout_count + 1 > 8{
                        if self.current_balance - self.min_balance_after_withdrawal > self.payout_cap.payouts_9_to_12{
                            return Some(self.payout_cap.payouts_9_to_12)
                        }
                    }
                    else{
                        if self.current_balance - self.min_balance_after_withdrawal > self.payout_cap.first_8_payouts{
                            return Some(self.payout_cap.first_8_payouts)
                        }
                    }
                    return Some(self.current_balance - self.min_balance_after_withdrawal);
                }
            }
        }
        None
    }

    pub fn make_withdrawal(&mut self, amount: f64) -> u8 {
         self.current_balance -= amount;
         self.max_winning_day_profit = 0.0; //TODO: is this reset every withdrawal?
         self.trading_days = 0;
         self.payout_count += 1;
         return self.payout_count;
    }

    pub fn try_add_trading_day(&mut self, daily_pnl: f64){
        
        self.simulation_days += 1;
        if self.real_trading_day.was_rtd(daily_pnl){
            self.trading_days += 1;

        }
        if daily_pnl > self.max_winning_day_profit{
            self.max_winning_day_profit = daily_pnl;
        }

    }
}
