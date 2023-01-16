use std::ops::DerefMut;

use chrono::Utc;
use sea_query::{Expr, PostgresQueryBuilder, Query, SimpleExpr};
use sqlx::{Error, Row};

use crate::routes::{NewProduct, ProductJson, ProductSearchParameters, UpdateProduct};
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

    async fn update(&self, update_product: UpdateProduct) -> Result<(), Error>;

    async fn delete(&self, id: i64) -> Result<(), Error>;

    async fn list(
        &self,
        param: ProductSearchParameters,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<ProductJson>, Error>;
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

    #[tracing::instrument(name = "Update a product into database", skip(self, update_product))]
    async fn update(&self, update_product: UpdateProduct) -> Result<(), Error> {
        let mut conn = self.session.get_session().await;

        let query = {
            let mut update_date = vec![];

            if let Some(name) = update_product.name {
                update_date.push((Products::Name, name.0.into()));
            }

            if let Some(currency) = update_product.currency {
                update_date.push((Products::Currency, currency.0.into()));
            }

            if let Some(price) = update_product.price {
                update_date.push((Products::Price, price.into()));
            }

            update_date.push((Products::UpdatedAt, Utc::now().into()));

            Query::update()
                .table(Products::Table)
                .values(update_date)
                .and_where(Expr::col((Products::Table, Products::Id)).eq(update_product.id))
                .to_string(PostgresQueryBuilder)
        };

        let _ = sqlx::query(query.as_str())
            .execute(conn.deref_mut())
            .await?;

        Ok(())
    }

    #[tracing::instrument(name = "Delete a product from database", skip(self, id))]
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

    #[tracing::instrument(name = "list products from database", skip(self))]
    async fn list(
        &self,
        keyboard: ProductSearchParameters,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<ProductJson>, Error> {
        let mut conn = self.session.get_session().await;
        let offset = page * page_size;

        fn format_like_string(col: Products, value: &str) -> SimpleExpr {
            let formatted_string = format!(r#"%{}%"#, value);
            Expr::col((Products::Table, col)).like(formatted_string.as_str())
        }

        let query = Query::select()
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
            .and_where_option(
                keyboard
                    .id
                    .as_ref()
                    .map(|e| Expr::col((Products::Table, Products::Id)).eq(*e)),
            )
            .and_where_option(
                keyboard
                    .partial_name
                    .as_ref()
                    .map(|e| format_like_string(Products::Name, e)),
            )
            .offset(offset)
            .limit(page_size)
            .to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, ProductJson>(dbg!(query.as_str()))
            .fetch_all(conn.deref_mut())
            .await
    }
}
