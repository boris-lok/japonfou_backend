-- Add migration script here

create table customers
(
    id         bigint       not null,
    name       varchar(256) not null,
    email      varchar(1024),
    phone      varchar(16),
    remark     text,
    created_at timestamp without time zone not null,
    updated_at timestamp without time zone,
    deleted_at timestamp without time zone,
    PRIMARY KEY (id)
);