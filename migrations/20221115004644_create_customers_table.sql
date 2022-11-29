-- Add migration script here

create table customers
(
    id         bigint       not null,
    name       varchar(256) not null,
    email      varchar(1024),
    phone      varchar(32),
    remark     text,
    created_at TIMESTAMPTZ  not null,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    PRIMARY KEY (id)
);