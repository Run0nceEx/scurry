-- Your SQL goes here
-- https://sqlite.org/foreignkeys.html

use pxdb;

CREATE TABLE ip_index(
    ip int not null,
    PRIMARY KEY(IP), 
    UNIQUE(ip)
);

create table protocols(
    identifer varchar(20) not null,
    PRIMARY KEY(identifer),
    UNIQUE(identifer)
);

create table providers(
    provider_name varchar(20) not null,
    PRIMARY KEY(provider_name),
    UNIQUE(provider_name)
);

create table service_index(
    id int not null AUTO_INCREMENT,
    
    ip int not null,
    port smallint not null,

    is_online BOOLEAN DEFAULT FALSE NOT NULL,
    provider_name varchar(20) not null,
    
    identifer varchar(20),

    PRIMARY KEY(id),
    FOREIGN KEY(identifer) REFERENCES protocols(identifer),
    FOREIGN KEY(provider_name) REFERENCES providers(provider_name),
    FOREIGN KEY(ip) REFERENCES ip_index(ip),
    UNIQUE(ip, port)
);

CREATE INDEX services on service_index(ip, port, identifer);


create table service_data(
    id int not null,

    -- ctrs
    alive_cnt int not null,
    dead_cnt int not null,
    
    -- timestamps
    alive_ts int not null,
    check_ts int not null,
    
    created_ts DATETIME DEFAULT CURRENT_TIMESTAMP not null,

    -- protocol data
    protocol_data binary,

    FOREIGN KEY(id) REFERENCES service_index(id),
    PRIMARY KEY(id)
);


-- this is basically extension fields for my plugins

create table score_data(
    id int not null,
    parent int not null,
    
    score float not null,
    weight_val float not null,
    bias float not null,

    ts DATETIME DEFAULT CURRENT_TIMESTAMP not null,
    custom_data binary,

    PRIMARY KEY(id),
    FOREIGN KEY(parent) REFERENCES service_index(id)
);

