use crate::repositories::{Customers, Products};
use chrono::Utc;
use sea_query::{Expr, JoinType, PostgresQueryBuilder, Query};
use sqlx::Error;

use crate::routes::{NewOrderItem, OrderItemJson};
use crate::utils::PostgresSession;

#[derive(sea_query::Iden)]
pub(crate) enum OrderItems {
    Table,
    Id,
    CustomerId,
    ProductId,
    Quantity,
    Status,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

#[async_trait::async_trait]
pub trait OrderItemRepo {
    async fn get(&self, id: i64) -> Result<Option<OrderItemJson>, Error>;

    async fn create(&self, new_order_item: NewOrderItem) -> Result<i64, Error>;
}

#[derive(Clone, Debug)]
pub struct PostgresOrderItemRepo {
    session: PostgresSession,
}

impl PostgresOrderItemRepo {
    pub fn new(session: PostgresSession) -> Self {
        Self { session }
    }
}

#[async_trait::async_trait]
impl OrderItemRepo for PostgresOrderItemRepo {
    #[tracing::instrument(name = "Get the order item from the database", skip(self))]
    async fn get(&self, id: i64) -> Result<Option<OrderItemJson>, Error> {
        let mut conn = self.session.get_session().await;

        let order_item_cols = vec![
            (OrderItems::Table, OrderItems::Id),
            (OrderItems::Table, OrderItems::Quantity),
            (OrderItems::Table, OrderItems::Status),
            (OrderItems::Table, OrderItems::CreatedAt),
            (OrderItems::Table, OrderItems::UpdatedAt),
            (OrderItems::Table, OrderItems::DeletedAt),
            (OrderItems::Table, OrderItems::CustomerId),
            (OrderItems::Table, OrderItems::ProductId),
        ];

        let customer_cols = vec![
            (Customers::Table, Customers::Name),
            (Customers::Table, Customers::CreatedAt),
        ];

        let product_cols = vec![
            (Products::Table, Products::Name),
            (Products::Table, Products::Currency),
            (Products::Table, Products::Price),
            (Products::Table, Products::CreatedAt),
        ];

        let query = Query::select()
            .columns(order_item_cols)
            .columns(customer_cols)
            .columns(product_cols)
            .from(OrderItems::Table)
            .join(
                JoinType::InnerJoin,
                Products::Table,
                Expr::col((OrderItems::Table, OrderItems::ProductId))
                    .equals((Products::Table, Products::Id)),
            )
            .join(
                JoinType::InnerJoin,
                Customers::Table,
                Expr::col((OrderItems::Table, OrderItems::CustomerId))
                    .equals((Customers::Table, Customers::Id)),
            )
            .and_where(Expr::col((OrderItems::Table, OrderItems::Id)).eq(id))
            .to_string(PostgresQueryBuilder);

        sqlx::query_as::<_, OrderItemJson>(query.as_str())
            .fetch_optional(conn.as_mut())
            .await
    }

    #[tracing::instrument(name = "Save a new order item into database", skip(self))]
    async fn create(&self, new_order_item: NewOrderItem) -> Result<i64, Error> {
        use sqlx::Row;

        let mut conn = self.session.get_session().await;

        let query = Query::insert()
            .into_table(OrderItems::Table)
            .columns(vec![
                OrderItems::Id,
                OrderItems::CustomerId,
                OrderItems::ProductId,
                OrderItems::Quantity,
                OrderItems::Status,
                OrderItems::CreatedAt,
            ])
            .values_panic(vec![
                new_order_item.id.into(),
                new_order_item.customer_id.0.into(),
                new_order_item.product_id.0.into(),
                new_order_item.quantity.0.into(),
                new_order_item.status.0.into(),
                Utc::now().into(),
            ])
            .returning(Query::returning().column(OrderItems::Id))
            .to_string(PostgresQueryBuilder);

        let res = sqlx::query(dbg!(&query))
            .fetch_one(conn.as_mut())
            .await?;

        Ok(res.get(0))
    }
}
