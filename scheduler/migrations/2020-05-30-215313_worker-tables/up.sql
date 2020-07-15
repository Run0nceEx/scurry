use pxdb;

create table Providers(
    provider_name TINYTEXT not null,
    PRIMARY KEY(provider_name),

    UNIQUE(provider_name)
);

create table ProtocolID(
    identifer TINYTEXT not null,
    rarity SMALLINT not null,

    PRIMARY KEY(identifer),
    UNIQUE(identifer)
);

create table ProtocolProbe(
    identifer TINYTEXT not null,

    regex TEXT not null,
    cpe_flags TEXT not null,
    probe_data binary not null,


    PRIMARY KEY(identifer),
    identifer REFERENCES ProtocolID(identifer),
);


create table ServiceIndex(
    id int not null AUTO_INCREMENT,
    
    ip int not null,
    port smallint not null,

    is_online BOOLEAN DEFAULT FALSE NOT NULL,
    provider_name TINYTEXT not null,
    
    identifer TINYTEXT,

    PRIMARY KEY(id),
    FOREIGN KEY(identifer) REFERENCES ProtocolID(identifer),
    FOREIGN KEY(provider_name) REFERENCES providers(provider_name),
    UNIQUE(ip, port)
);


CREATE INDEX services on service_index(ip, port, identifer);

-- Service data, including fails, blocks, etc
create table ServiceData(
    id int not null,

    -- ctrs
    alive_cnt int DEFAULT 0 not null,
    dead_cnt int DEFAULT 0 not null,
    
    -- timestamps
    alive_ts DATETIME,
    check_ts DATETIME DEFAULT CURRENT_TIMESTAMP not null,
    created_ts DATETIME DEFAULT CURRENT_TIMESTAMP not null,


    blocked boolean DEFAULT 0 not null,
    blocked_ts DATETIME,

    -- protocol data
    -- protocol_data binary,

    CPE TEXT DEFAULT "CPE://" not null,
    
    PRIMARY KEY(id),
    FOREIGN KEY(id) REFERENCES service_index(id),
    
);


-- this is basically extension fields for my plugins


-- internal bias data for look ups
create table ScoreData(
    id int not null AUTO_INCREMENT,
    parent int not null,
    
    score float DEFAULT 0.0 not null,
    weight_val float DEFAULT 0.0 not null,
    bias float DEFAULT 0.0 not null,

    ts DATETIME DEFAULT CURRENT_TIMESTAMP not null,

    PRIMARY KEY(id),
    FOREIGN KEY(parent) REFERENCES service_index(id)
);


-- Port frequency graph, this is for clients to save and update occasionally them selves to
create table PortFrequencyGraph(
    id int not null AUTO_INCREMENT,
    protocol TINYTEXT,
    transport_layer tinytext not null,
    
    port int not null,
    frequency float not null,

    PRIMARY KEY(id)
);


create table Latency(
    id int not null AUTO_INCREMENT,
    parent int not null,
    ts DATETIME default CURRENT_TIMESTAMP not null,

    ms float not null,

    PRIMARY KEY(id),
    
    parent REFERENCES service_index(id)
);


create table SpeedTest(
    id int not null AUTO_INCREMENT,
    parent int not null,
    
    ts DATETIME default CURRENT_TIMESTAMP not null,

    mbps float not null,

    PRIMARY KEY(id),
    parent REFERENCES service_index(id)
);

