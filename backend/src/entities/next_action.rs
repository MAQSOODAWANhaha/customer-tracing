use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(64))")]
pub enum NextAction {
    #[sea_orm(string_value = "继续跟进")]
    Continue,
    #[sea_orm(string_value = "结束跟进")]
    End,
}

impl Default for NextAction {
    fn default() -> Self {
        NextAction::Continue
    }
}

impl NextAction {
    pub fn as_str(&self) -> &str {
        match self {
            NextAction::Continue => "继续跟进",
            NextAction::End => "结束跟进",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "继续跟进" => Some(NextAction::Continue),
            "结束跟进" => Some(NextAction::End),
            _ => None,
        }
    }
    
    pub fn variants() -> Vec<&'static str> {
        vec!["继续跟进", "结束跟进"]
    }
}