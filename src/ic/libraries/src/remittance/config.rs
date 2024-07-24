use crate::crypto::ecdsa;
use candid::CandidType;
use serde_derive::Deserialize;

#[derive(CandidType, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum Environment {
    #[default]
    Development,
    Staging,
    Production,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct Config {
    pub env: Environment,
    pub key: ecdsa::EcdsaKeyIds,
    pub sign_cycles: u64,
}
impl Default for Config {
    fn default() -> Self {
        Self::from(Environment::Development)
    }
}
impl From<Environment> for Config {
    fn from(env: Environment) -> Self {
        if env == Environment::Staging {
            Self {
                env: Environment::Staging,
                key: ecdsa::EcdsaKeyIds::TestKey1,
                sign_cycles: 10_000_000_000,
            }
        } else if env == Environment::Production {
            Self {
                env: Environment::Production,
                key: ecdsa::EcdsaKeyIds::ProductionKey1,
                sign_cycles: 26_153_846_153,
            }
        } else {
            Self {
                env: Environment::Development,
                key: ecdsa::EcdsaKeyIds::TestKeyLocalDevelopment,
                sign_cycles: 25_000_000_000,
            }
        }
    }
}
