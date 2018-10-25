use mysql::{Pool, Transaction};
use failure::Error;
use graph_descriptions::graph_description::*;
use graph_descriptions::graph_description::host::*;
use graph_descriptions::*;
use uuid;

pub fn create_ip_asset_session(conn: &Pool,
                               ip: impl AsRef<str>,
                               asset_id: impl AsRef<str>,
                               create_time: u64) -> Result<(), Error> {
    let skewed_create_time = create_time - (create_time % 10);

    let query = format!(r#"
       INSERT INTO ip_asset_history
          (ip, asset_id, create_time)
          VALUES
              ("{}", "{}", "{}")"#,
                &ip.as_ref(),
                &asset_id.as_ref(),
                skewed_create_time.to_string().as_str());

    let asset_id = asset_id.as_ref();
    conn.prep_exec(&query, &())?;
    Ok(())
}


pub fn create_table(conn: &Pool) {
    info!("Creating ip_asset_history table");
    conn.prep_exec("DROP TABLE IF EXISTS `ip_asset_history`", &());

    conn.prep_exec("CREATE TABLE IF NOT EXISTS ip_asset_history (
                primary_key     SERIAL PRIMARY KEY,
                asset_id        TEXT NOT NULL,
                ip              BLOB NOT NULL,
                create_time     NUMERIC NOT NULL
              )", &()).expect("ip_asset_history::create_table");
}