use std::ops::DerefMut;

use chrono::Utc;
use sea_query::{PostgresQueryBuilder, Query};
use sqlx::{Error, Row};

use crate::routes::NewProduct;
use crate::utils::PostgresSession;

#[derive(sea_query::Iden)]
enum Products {
    Table,
    Id,
    Name,
    Currency,
    Price,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[async_trait::async_trait]
pub trait ProductRepository {
    async fn create(&self, new_product: NewProduct) -> Result<i64, Error>;
}

pub struct PostgresProductRepoImpl {
    pub session: PostgresSession,
}

impl PostgresProductRepoImpl {
    pub fn new(session: PostgresSession) -> Self {
        Self { session }
    }
}

#[async_trait::async_trait]
impl ProductRepository for PostgresProductRepoImpl {
    #[tracing::instrument(name = "Save a new product into database", skip(self, customer))]
    async fn create(&self, new_product: NewProduct) -> Result<i64, Error> {
        let mut conn = self.session.get_session().await;

        let query = {
            let id = new_product.id.into();
            let name = new_product.name.into();
            let currency = new_product.currency.into();
            let price = new_product.price.into();
            let now = Utc::now();
            let created_at = now.clone().into();
            let updated_at = now.into();

            let query = Query::insert()
                .into_table(Products::Table)
                .columns([
                    Products::Id,
                    Products::Name,
                    Products::Currency,
                    Products::Price,
                    Products::CreatedAt,
                    Products::UpdatedAt,
                ])
                .values_panic([id, name, currency, price, created_at, updated_at])
                .returning(Query::returning().column(Products::Id))
                .to_string(PostgresQueryBuilder);

            query
        };

        let res = sqlx::query(dbg!(&query))
            .fetch_one(conn.deref_mut())
            .await?;

        Ok(res.get(0))
    }
}
