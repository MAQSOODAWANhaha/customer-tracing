use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sea_orm::entity::prelude::*;

/// 客户分组枚举
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
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

impl Serialize for CustomerGroup {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            CustomerGroup::GroupClass => serializer.serialize_str("团课"),
            CustomerGroup::SmallClass => serializer.serialize_str("小班"),
            CustomerGroup::Personal => serializer.serialize_str("私教"),
            CustomerGroup::Training => serializer.serialize_str("教培"),
        }
    }
}

impl<'de> Deserialize<'de> for CustomerGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "团课" => Ok(CustomerGroup::GroupClass),
            "小班" => Ok(CustomerGroup::SmallClass),
            "私教" => Ok(CustomerGroup::Personal),
            "教培" => Ok(CustomerGroup::Training),
            // 为了兼容性，也支持英文变量名
            "GroupClass" => Ok(CustomerGroup::GroupClass),
            "SmallClass" => Ok(CustomerGroup::SmallClass),
            "Personal" => Ok(CustomerGroup::Personal),
            "Training" => Ok(CustomerGroup::Training),
            _ => Err(serde::de::Error::unknown_variant(&s, &["团课", "小班", "私教", "教培"])),
        }
    }
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