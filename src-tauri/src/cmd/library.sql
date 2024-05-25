create table main.package
(
    id          integer not null
        constraint package_pk
            primary key autoincrement,
    name        text    not null,
    description text,
    author      text
);

create index main.package_author_index
    on main.package (author);

create index main.package_name_index
    on main.package (name);

create table main.sticky
(
    id          integer                            not null
        constraint sticky_pk
            primary key autoincrement,
    filename    text,
    create_date datetime default current_timestamp not null,
    modify_date datetime default current_timestamp not null,
    name        text                               not null,
    package     integer                            not null
        constraint sticky_package_id_fk
            references main.package,
    sensor_id   TEXT,
    width       integer  default -1                not null,
    height      integer  default -1                not null,
    type        TEXT                               not null,
    constraint check_sticky_has_file
        check (type != 'PIC' OR (type = 'PIC' AND filename IS NOT NULL))
);

create index main.sticky_create_date_index
    on main.sticky (create_date);

create index main.sticky_modify_date_index
    on main.sticky (modify_date);

create index main.sticky_sensor_id_index
    on main.sticky (sensor_id);

create table main.subscription
(
    id    integer not null
        constraint subscription_pk
            primary key autoincrement
        constraint subscription_package_id_fk
            references main.package,
    type  integer not null,
    url   text    not null,
    extra text
);

create table main.tag
(
    id        integer not null
        constraint tag_pk
            primary key autoincrement,
    namespace text    not null,
    value     text    not null
);

create table main.sticky_tag
(
    sticky integer not null
        constraint sticky_id_fk
            references main.sticky,
    tag    integer not null
        constraint tag_id_fk
            references main.tag,
    constraint sticky_tag_pk
        primary key (tag, sticky)
);

create index main.tag_namespace_value_index
    on main.tag (namespace, value);

create table main.version
(
    version_code integer not null
        constraint version_pk
            primary key
);

