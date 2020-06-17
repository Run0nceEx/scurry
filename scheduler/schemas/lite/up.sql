-- Your SQL goes here
-- https://sqlite.org/foreignkeys.html


--create database if not exists pxdb-temp;
--use pxdb-temp;

create database if not exists pxdb;
use pxdb;

CREATE TABLE ip_index(
    ip int primary key not null,
    UNIQUE(ip) ON CONFLICT ABORT
);

create table protocols(
    identifer varchar(20) primary key not null,
    UNIQUE(identifer) ON CONFLICT ABORT
);

create table providers(
    provider_name varchar(20) primary key not null,
    UNIQUE(provider_name) ON CONFLICT ABORT
);


create table service_index(
    id int PRIMARY key not null,
    
    ip int references ip_index(ip) on update cascade not null,
    port smallint not null,

    is_online BOOLEAN NOT NULL CHECK (is_online IN (0,1)),  -- "real" Boolean storage (1 or 0 (not null))
    provider_name varchar(20) references providers(provider_name) on update cascade on delete ignore not null,
    identifer varchar(20) REFERENCES protocols(identifer) on update cascade,
    
    UNIQUE(ip, port) ON CONFLICT FAIL
);

CREATE INDEX services on service_index(ip, port, identifer);


create table service_data(
    id int REFERENCES service_index(id) on update cascade PRIMARY KEY not null,

    -- ctrs
    alive_cnt int not null,
    dead_cnt int not null,
    
    -- timestamps
    alive_ts int not null, -- last time alive ts
    check_ts int not null, -- check at this instant

    created_ts int DEFAULT CURRENT_TIMESTAMP not null,

    -- protocol data
    protocol_data binary
);


-- this is basically extension fields for my plugins

create table score_data(
    id int PRIMARY KEY AUTOINCREMENT,
    parent int REFERENCES service_index(id) on update cascade not null,
    
    score float not null,
    weight_val float not null,
    bias float not null,

    ts int DEFAULT CURRENT_TIMESTAMP not null,
    custom_data binary
) AUTOINCREMENT = 1;

