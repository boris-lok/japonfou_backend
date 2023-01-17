-- Add migration script here

create table order_items
(
    id          bigint      not null,
    customer_id bigint      not null,
    quantity    smallint    not null,
    status      smallint    not null,
    product_id  bigint      not null,
    created_at  TIMESTAMPTZ not null,
    updated_at  TIMESTAMPTZ,
    deleted_at  TIMESTAMPTZ,
    primary key
)
