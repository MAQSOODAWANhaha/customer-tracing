use serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;

/// 客户分组枚举
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
pub enum CustomerGroup {
    #[sea_orm(string_value = "团课")]
    GroupClass,
    #[sea_orm(string_value = "小班")]
    SmallClass,
    #[sea_orm(string_value = "私教")]
    Personal,
    #[sea_orm(string_value = "教培")]
    Training,
}

impl Default for CustomerGroup {
    fn default() -> Self {
        CustomerGroup::GroupClass
    }
}

impl std::fmt::Display for CustomerGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomerGroup::GroupClass => write!(f, "团课"),
            CustomerGroup::SmallClass => write!(f, "小班"),
            CustomerGroup::Personal => write!(f, "私教"),
            CustomerGroup::Training => write!(f, "教培"),
        }
    }
}