use chrono::{Utc, DateTime};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub from: String,
    pub to: String,
    pub current_price: f32,
    pub updated_at: DateTime<Utc>
}

#[derive(EnumIter, DeriveActiveEnum, Clone, Debug, PartialEq)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
#[repr(i32)]
pub enum Role {
    User = 1,
    Admin = 2
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        has_many = "super::alert::Entity"
    )]
    Alerts,
}

impl Related<super::alert::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Alerts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}