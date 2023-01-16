use std::ops::DerefMut;

use chrono::Utc;
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sqlx::{Error, Row};

use crate::routes::{NewProduct, ProductJson};
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
    async fn get(&self, id: i64) -> Result<Option<ProductJson>, Error>;

    async fn create(&self, new_product: NewProduct) -> Result<i64, Error>;

    async fn delete(&self, id: i64) -> Result<(), Error>;
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
    #[tracing::instrument(name = "get a product from database", skip(self))]
    async fn get(&self, id: i64) -> Result<Option<ProductJson>, Error> {
        let mut conn = self.session.get_session().await;
        let query = {
            Query::select()
                .from(Products::Table)
                .columns([
                    Products::Id,
                    Products::Name,
                    Products::Currency,
                    Products::Price,
                    Products::CreatedAt,
                    Products::UpdatedAt,
                    Products::DeletedAt,
                ])
                .and_where(Expr::col((Products::Table, Products::Id)).eq(id))
                .to_string(PostgresQueryBuilder)
        };

        sqlx::query_as::<_, ProductJson>(query.as_str())
            .fetch_optional(conn.deref_mut())
            .await
    }

    #[tracing::instrument(name = "Save a new product into database", skip(self, new_product))]
    async fn create(&self, new_product: NewProduct) -> Result<i64, Error> {
        let mut conn = self.session.get_session().await;

        let query = {
            let id = new_product.id.into();
            let name = new_product.name.0.into();
            let currency = new_product.currency.0.into();
            let price = new_product.price.into();
            let now = Utc::now();
            let created_at = now.into();
            let updated_at = now.into();

            Query::insert()
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
                .to_string(PostgresQueryBuilder)
        };

        let res = sqlx::query(dbg!(&query))
            .fetch_one(conn.deref_mut())
            .await?;

        Ok(res.get(0))
    }

    async fn delete(&self, id: i64) -> Result<(), Error> {
        let mut conn = self.session.get_session().await;

        let query = Query::update()
            .table(Products::Table)
            .values([(Products::DeletedAt, Utc::now().into())])
            .and_where(Expr::col((Products::Table, Products::Id)).eq(id))
            .to_string(PostgresQueryBuilder);

        let _ = sqlx::query(query.as_str()).execute(conn.deref_mut()).await;

        Ok(())
    }
}
