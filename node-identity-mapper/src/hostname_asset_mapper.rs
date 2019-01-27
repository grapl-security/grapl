use mysql::{Pool, Transaction};
use failure::Error;
use graph_descriptions::graph_description::*;
use graph_descriptions::graph_description::host::*;
use graph_descriptions::*;
use uuid;

pub fn create_hostname_asset_session(conn: &Pool,
                                     hostname: impl AsRef<str>,
                                     asset_id: impl AsRef<str>,
                                     create_time: u64) -> Result<(), Error> {
    info!("Mapping ip {} timestamp {} to asset_id{}",
          hostname.as_ref(), create_time, asset_id.as_ref());

    let query = format!(r#"
       INSERT INTO hostname_asset_history
          (hostname, asset_id, create_time)
          VALUES
              ("{}", "{}", "{}")"#,
                        &hostname.as_ref(),
                        &asset_id.as_ref(),
                        create_time.to_string().as_str());

    conn.prep_exec(&query, &())?;
    Ok(())
}


pub fn create_table(conn: &Pool) {
    info!("Creating ip_asset_history table");
//    conn.prep_exec("DROP TABLE IF EXISTS `hostname_asset_history`", &());

    conn.prep_exec("CREATE TABLE IF NOT EXISTS hostname_asset_history (
                primary_key     SERIAL PRIMARY KEY,
                asset_id        TEXT NOT NULL,
                hostname        BLOB NOT NULL,
                create_time     BIGINT UNSIGNED NOT NULL
              )", &()).expect("ip_asset_history::create_table");
}