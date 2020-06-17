-- Your SQL goes here
-- https://sqlite.org/foreignkeys.html


CREATE TABLE ip_index(
    ip int primary key not null,
    UNIQUE(ip) on CONFLICT REPLACE
);

-- CREATE INDEX ip ON ip_index(ip);


create table protocols(
    identifer varchar(20) PRIMARY key not null,
    UNIQUE(identifier) on CONFLICT REPLACE
);



create table providers(
    provider_name varchar(20) PRIMARY key not null,
    UNIQUE(provider_name) on CONFLICT REPLACE
);

-- CREATE INDEX identifer ON protocols(identifer);

create table service_index(
    id int PRIMARY key AUTOINCREMENT,
    
    ip int references ip_index(ip) on update cascade not null,
    port smallint not null,

    provider_name varchar(20) references providers(provider_name) on update cascade not null,
    identifer int REFERENCES protocols(identifer) on update cascade,
    
    UNIQUE(ip, port) ON CONFLICT REPLACE
    
);


create table service_data(
    id int REFERENCES service_index(id) on update cascade PRIMARY KEY,

    -- ctrs
    alive_cnt int not null,
    dead_cnt int not null,    
    
    -- timestamps
    alive_ts int not null,
    check_ts int not null,
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
);



