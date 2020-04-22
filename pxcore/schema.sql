
/* Main table to gather from */
create table if not exists SERVICE_INDEX(
    id int primary key,

    /* ip address in u64 format */
    atoi bigint not null,
    port tinyint not null,
    unique(atoi, port) on CONFLICT REPLACE

    /* protocol identifier */
    protocol varchar(32) not null,

    /* where was this obtained? */
    provider varchar(32) not null,

    /* Indexes */
    analytic_id int not null,
    last_log_id int not null
);


/* Keep track of the failure rate of a service*/
create table if not exists SERVICE_ANALYTICS(
    id int primary key,
    /* Successful scans */
    positive int default 0,
    /* Failed scans */
    negative int default 0,
    created CURRENT_TIMESTAMP,
);


/* Mine log of all Services */
create table if not exists SERVICE_LOG(
    id int primary key,

    provider varchar(32) not null,

    /* Index's id */
    index_id int not null,

    /* Scan Time stamp */
    ts CURRENT_TIMESTAMP not null,
    result bool not null
);

