-- Add migration script here

create table products
(
    id         bigint         not null,
    name       varchar(256)   not null,
    currency   smallint       not null,
    price      decimal(12, 2) not null,
    created_at TIMESTAMPTZ    not null,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    primary key (id)
)
