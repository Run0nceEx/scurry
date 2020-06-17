// `diesel setup`
// mod schema

mod models;
mod schema;
mod actors;

use diesel::Connection;
use diesel::MysqlConnection;

pub fn establish_connection(database_url: String) -> MysqlConnection {
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}



