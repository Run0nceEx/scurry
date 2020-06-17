table! {
    ip_index (ip) {
        ip -> Integer,
    }
}

table! {
    protocols (identifer) {
        identifer -> Varchar,
    }
}

table! {
    providers (provider_name) {
        provider_name -> Varchar,
    }
}

table! {
    score_data (id) {
        id -> Integer,
        parent -> Integer,
        score -> Float,
        weight_val -> Float,
        bias -> Float,
        ts -> Datetime,
        custom_data -> Nullable<Binary>,
    }
}

table! {
    service_data (id) {
        id -> Integer,
        alive_cnt -> Integer,
        dead_cnt -> Integer,
        alive_ts -> Integer,
        check_ts -> Integer,
        created_ts -> Datetime,
        protocol_data -> Nullable<Binary>,
    }
}

table! {
    service_index (id) {
        id -> Integer,
        ip -> Integer,
        port -> Smallint,
        is_online -> Bool,
        provider_name -> Varchar,
        identifer -> Varchar,
    }
}

joinable!(score_data -> service_index (parent));
joinable!(service_data -> service_index (id));
joinable!(service_index -> ip_index (ip));
joinable!(service_index -> protocols (identifer));
joinable!(service_index -> providers (provider_name));

allow_tables_to_appear_in_same_query!(
    ip_index,
    protocols,
    providers,
    score_data,
    service_data,
    service_index,
);
