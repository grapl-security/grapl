use postgres::Connection;

use failure::Error;
use graph_descriptions::graph_description::*;
use graph_descriptions::graph_description::host::*;
use graph_descriptions::*;


use uuid;

pub fn create_ip_asset_session(conn: &Connection,
                               ip: impl AsRef<str>,
                               asset_id: impl AsRef<str>,
                               create_time: u64) -> Result<(), Error> {

    // TODO: Escape stuff
    let query = r"
       INSERT INTO ip_asset_history
          (session_id, ip, create_time)
          VALUES
              ($0, $1, $2, $3)";

    let asset_id = asset_id.as_ref();
    conn.execute(&query, &[&asset_id,
        &ip.as_ref(), &(create_time as i64)])?;
    Ok(())
}