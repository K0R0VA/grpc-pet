use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "alerts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i32,
    pub user_id: i32,
    pub route_id: i32,
    pub target_price: f32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    Users,
    #[sea_orm(
        belongs_to = "super::route::Entity",
        from = "Column::RouteId",
        to = "super::route::Column::Id"
    )]
    Routes,
}

#[derive(Debug)]
pub struct AlertSubscription;

impl Linked for AlertSubscription {
    type FromEntity = Entity;

    type ToEntity = super::subscription::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            Relation::Users.def().rev(),
            crate::user::Relation::Subscriptions.def()
        ]
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::route::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Routes.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}