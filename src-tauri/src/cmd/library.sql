create table package
(
    id          integer not null
        constraint package_pk
            primary key autoincrement,
    name        text    not null,
    description text,
    author      text
);

create index package_author_index
    on package (author);

create index package_name_index
    on package (name);

create table sticky
(
    id          integer                            not null
        constraint sticky_pk
            primary key autoincrement,
    hash        text                               not null
        constraint sticky_pk2
            unique,
    create_date datetime default current_timestamp not null,
    modify_date integer  default current_timestamp not null,
    name        text                               not null,
    package     integer                            not null
        constraint sticky_package_id_fk
            references package,
    sensor_id   text
);

create index sticky_create_date_index
    on sticky (create_date);

create index sticky_modify_date_index
    on sticky (modify_date);

create index sticky_sensor_id_index
    on sticky (sensor_id);

create table subscription
(
    id    integer not null
        constraint subscription_pk
            primary key autoincrement
        constraint subscription_package_id_fk
            references package,
    type  integer not null,
    url   text    not null,
    extra text
);

create table tag
(
    id        integer not null
        constraint tag_pk
            primary key autoincrement,
    namespace text    not null,
    value     text    not null
);

create table sticky_tag
(
    sticky integer not null
        constraint sticky_id_fk
            references sticky,
    tag    integer not null
        constraint tag_id_fk
            references tag
);

create index tag_namespace_value_index
    on tag (namespace, value);


INSERT INTO package (id, name, description, author) VALUES (1, 'Inbox', null, null);