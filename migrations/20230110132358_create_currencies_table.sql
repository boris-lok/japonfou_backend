-- Add migration script here

create table currencies
(
    id   smallint   not null,
    name varchar(8) not null,
    primary key (id)
);

insert into currencies (id, name)
values (446, 'MOP'),
       (344, 'HKD');
