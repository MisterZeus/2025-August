use protocols;

drop table if exists `products`;

create table `products` (
    id int not null,
    country char(2) not null,
    title varchar(100) not null,
    description longtext null,
    category_id smallint null,
    price decimal(10,2) not null,
    quantity bigint not null,
    create_dt datetime not null,

    primary key (id)
);

create unique index ix_uq_products_title on `products`(title);

insert into `products` (id, country, title, description, category_id, price, quantity, create_dt)
values
    (1, 'UK', 'laptop', null, 2, 123.4, 3, '2025-01-01'),
    (2, 'CY', 'phone', 'Just a phone desc', 20000, 7.890, 30000, '2025-06-01');

-- create table `sample` (
--     t tinyint not null,
--     i int not null,
--     s longtext not null
-- );
--
-- insert into `sample` (t, i, s)
-- SELECT 1, 2147483647, REPEAT('A', 10000000);