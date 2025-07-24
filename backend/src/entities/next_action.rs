use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(64))")]
pub enum NextAction {
    #[sea_orm(string_value = "继续跟进")]
    Continue,
    #[sea_orm(string_value = "结束跟进")]
    End,
}

impl Serialize for NextAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for NextAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).ok_or_else(|| {
            serde::de::Error::unknown_variant(&s, &["继续跟进", "结束跟进"])
        })
    }
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