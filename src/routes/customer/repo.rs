use std::ops::DerefMut;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sqlx::{PgPool, Postgres, Row};
use sqlx::pool::PoolConnection;
use tokio::sync::Mutex;

use crate::routes::{Customers, NewCustomer, ValidEmail, ValidPhone};

#[async_trait]
pub trait CustomerRepo {
    async fn create(&self, customer: NewCustomer) -> Result<i64, sqlx::Error>;
    async fn check_if_customer_is_exist(
        &self,
        email: &Option<ValidEmail>,
        phone: &Option<ValidPhone>,
    ) -> Result<bool, sqlx::Error>;
}

#[derive(Clone, Debug)]
pub struct PostgresCustomerRepoImpl {
    session: Arc<Mutex<PoolConnection<Postgres>>>,
}

impl PostgresCustomerRepoImpl {
    pub async fn new(pool: PgPool) -> Result<Self, sqlx::Error> {
        let session = pool.acquire().await?;
        Ok(Self {
            session: Arc::new(Mutex::new(session)),
        })
    }
}

#[async_trait]
impl CustomerRepo for PostgresCustomerRepoImpl {
    #[tracing::instrument(name = "Save a new customer into database", skip(self, customer))]
    async fn create(&self, customer: NewCustomer) -> Result<i64, sqlx::Error> {
        let mut conn = self.session.lock().await;

        let sql = {
            let id = customer.id.into();
            let name = customer.name.into();
            let email = customer.email.map(|e| e.0).into();
            let phone = customer.phone.map(|e| e.0).into();
            let remark = customer.remark.into();
            let created_at = Utc::now().into();
            let sql = Query::insert()
                .into_table(Customers::Table)
                .columns([
                    Customers::Id,
                    Customers::Name,
                    Customers::Email,
                    Customers::Phone,
                    Customers::Remark,
                    Customers::CreatedAt,
                ])
                .values_panic([id, name, email, phone, remark, created_at])
                .returning(Query::returning().column(Customers::Id))
                .to_string(PostgresQueryBuilder);
            sql
        };

        dbg!(&sql);

        let res = sqlx::query(&sql).fetch_one(conn.deref_mut()).await?;

        Ok(res.get(0))
    }

    #[tracing::instrument(
        name = "check if customer is exist in postgres database",
        skip(self, email, phone)
    )]
    async fn check_if_customer_is_exist(
        &self,
        email: &Option<ValidEmail>,
        phone: &Option<ValidPhone>,
    ) -> Result<bool, sqlx::Error> {
        let mut conn = self.session.lock().await;

        let sql = Query::select()
            .column(Customers::Id)
            .from(Customers::Table)
            .and_where_option(
                email
                    .as_ref()
                    .map(|e| Expr::tbl(Customers::Table, Customers::Email).eq(&*e.0)),
            )
            .and_where_option(
                phone
                    .as_ref()
                    .map(|e| Expr::tbl(Customers::Table, Customers::Phone).eq(&*e.0)),
            )
            .to_string(PostgresQueryBuilder);

        sqlx::query(&sql)
            .fetch_optional(conn.deref_mut())
            .await
            .map(|row| row.map_or_else(|| false, |e| e.len() > 0))
    }
}
