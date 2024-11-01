// src/simulator/prop_account/account_type.rs
use serde::{Serialize, Deserialize};
use std::str::FromStr;

use super::{FttAccountType, TopstepAccountType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountType {
    Ftt(FttAccountType),
    TopStep(TopstepAccountType),
    // Add other companies' account types here...
}

impl FromStr for AccountType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid account type format. Use 'company:account_type'.");
        }
        let company = parts[0].to_lowercase();
        let account_type = parts[1];

        match company.as_str() {
            "ftt" => {
                let ftt_type = FttAccountType::from_str(account_type)?;
                Ok(AccountType::Ftt(ftt_type))
            }
            "topstep" => {
                let topstep_type = TopstepAccountType::from_str(account_type)?;
                Ok(AccountType::TopStep(topstep_type))
            }
            // Add other companies...
            _ => Err("Unknown company"),
        }
    }
}

