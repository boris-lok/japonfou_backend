use std::ops::DerefMut;

use async_trait::async_trait;
use chrono::Utc;
use sea_query::{Expr, LikeExpr, PostgresQueryBuilder, Query};

use sqlx::{Error, Row};

use crate::routes::{
    CustomerJson, CustomerSearchParameters, NewCustomer, UpdateCustomer, ValidEmail, ValidPhone,
};
use crate::utils::PostgresSession;

#[derive(sea_query::Iden)]
enum Customers {
    Table,
    Id,
    Name,
    Email,
    Phone,
    Remark,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[async_trait]
pub trait CustomerRepo {
    async fn get(&self, customer_id: i64) -> Result<Option<CustomerJson>, Error>;

    async fn create(&self, customer: NewCustomer) -> Result<i64, Error>;

    async fn update(&self, customer: UpdateCustomer) -> Result<(), Error>;

    async fn delete(&self, id: i64) -> Result<(), Error>;

    async fn list(
        &self,
        keyword: CustomerSearchParameters,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<CustomerJson>, Error>;

    async fn check_if_customer_is_exist(
        &self,
        id: &Option<i64>,
        email: &Option<ValidEmail>,
        phone: &Option<ValidPhone>,
    ) -> Result<bool, Error>;
}

#[derive(Clone, Debug)]
pub struct PostgresCustomerRepoImpl {
    pub session: PostgresSession,
}
impl PostgresCustomerRepoImpl {
    pub fn new(session: PostgresSession) -> Self {
        Self { session }
    }
}

#[async_trait]
impl CustomerRepo for PostgresCustomerRepoImpl {
    #[tracing::instrument(name = "get a customer from database", skip(self))]
    async fn get(&self, customer_id: i64) -> Result<Option<CustomerJson>, Error> {
        let mut conn = self.session.get_session().await;
        let query = Query::select()
            .from(Customers::Table)
            .columns([
                Customers::Id,
                Customers::Name,
                Customers::Email,
                Customers::Phone,
                Customers::Remark,
                Customers::CreatedAt,
                Customers::UpdatedAt,
                Customers::DeletedAt,
            ])
            .and_where(Expr::tbl(Customers::Table, Customers::Id).eq(customer_id))
            .to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, CustomerJson>(dbg!(&query))
            .fetch_optional(conn.deref_mut())
            .await
    }

    #[tracing::instrument(name = "Save a new customer into database", skip(self, customer))]
    async fn create(&self, customer: NewCustomer) -> Result<i64, sqlx::Error> {
        let mut conn = self.session.get_session().await;

        let query = {
            let id = customer.id.into();
            let name = customer.name.into();
            let email = customer.email.map(|e| e.0).into();
            let phone = customer.phone.map(|e| e.0).into();
            let remark = customer.remark.into();
            let created_at = Utc::now().into();
            let query = Query::insert()
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
            query
        };

        let res = sqlx::query(dbg!(&query))
            .fetch_one(conn.deref_mut())
            .await?;

        Ok(res.get(0))
    }

    #[tracing::instrument(name = "update a customer in database", skip(self, customer))]
    async fn update(&self, customer: UpdateCustomer) -> Result<(), Error> {
        let mut conn = self.session.get_session().await;
        let query = {
            let mut update_data = vec![];
            if let Some(name) = customer.name {
                update_data.push((Customers::Name, name.into()));
            }

            if let Some(email) = customer.email {
                update_data.push((Customers::Email, email.0.into()));
            }

            if let Some(phone) = customer.phone {
                update_data.push((Customers::Phone, phone.0.into()));
            }

            if let Some(remark) = customer.remark {
                update_data.push((Customers::Remark, remark.into()));
            }

            update_data.push((Customers::UpdatedAt, Utc::now().into()));

            let query = Query::update()
                .table(Customers::Table)
                .values(update_data)
                .and_where(Expr::tbl(Customers::Table, Customers::Id).eq(customer.id))
                .to_string(PostgresQueryBuilder);

            query
        };

        let _ = sqlx::query(dbg!(&query)).execute(conn.deref_mut()).await?;

        Ok(())
    }

    #[tracing::instrument(name = "mark a customer deleted_at in database", skip(self))]
    async fn delete(&self, id: i64) -> Result<(), Error> {
        let mut conn = self.session.get_session().await;

        let query = Query::update()
            .table(Customers::Table)
            .values([(Customers::DeletedAt, Utc::now().into())])
            .and_where(Expr::tbl(Customers::Table, Customers::Id).eq(id))
            .to_string(PostgresQueryBuilder);

        let _ = sqlx::query(dbg!(&query)).execute(conn.deref_mut()).await;

        Ok(())
    }

    async fn list(
        &self,
        keyword: CustomerSearchParameters,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<CustomerJson>, Error> {
        let mut conn = self.session.get_session().await;
        let offset = page * page_size;

        let query = Query::select()
            .from(Customers::Table)
            .columns([
                Customers::Id,
                Customers::Name,
                Customers::Email,
                Customers::Phone,
                Customers::Remark,
                Customers::CreatedAt,
                Customers::UpdatedAt,
            ])
            .and_where_option(
                keyword
                    .id
                    .as_ref()
                    .map(|e| Expr::tbl(Customers::Table, Customers::Id).eq(*e)),
            )
            .and_where_option(keyword.partial_email.as_ref().map(|e| {
                Expr::tbl(Customers::Table, Customers::Email).like(LikeExpr::str(e.as_str()))
            }))
            .and_where_option(keyword.partial_phone.as_ref().map(|e| {
                Expr::tbl(Customers::Table, Customers::Phone).like(LikeExpr::str(e.as_str()))
            }))
            .and_where_option(keyword.partial_remark.as_ref().map(|e| {
                Expr::tbl(Customers::Table, Customers::Remark).like(LikeExpr::str(e.as_str()))
            }))
            .offset(offset)
            .limit(page_size)
            .to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, CustomerJson>(&query)
            .fetch_all(conn.deref_mut())
            .await
    }

    #[tracing::instrument(
        name = "check if customer is exist in database",
        skip(self, email, phone)
    )]
    async fn check_if_customer_is_exist(
        &self,
        id: &Option<i64>,
        email: &Option<ValidEmail>,
        phone: &Option<ValidPhone>,
    ) -> Result<bool, sqlx::Error> {
        let mut conn = self.session.get_session().await;

        let query = Query::select()
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
            .and_where_option(id.map(|e| Expr::tbl(Customers::Table, Customers::Id).ne(e)))
            .to_string(PostgresQueryBuilder);

        sqlx::query(dbg!(&query))
            .fetch_optional(conn.deref_mut())
            .await
            .map(|row| row.map_or_else(|| false, |e| e.len() > 0))
    }
}
