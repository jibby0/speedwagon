use crate::{db, db::sources};
use log;

pub fn fetch_from_all_sources(pool: &mut db::Pool) {
    let conn = match pool.get() {
        Ok(conn) => db::DbConn(conn),
        Err(e) => {
            log::error!("{}", e);
            return;
        }
    };
    let _sources = match sources::all(&*conn) {
        Ok(srcs) => srcs,
        Err(e) => {
            log::error!("{}", e);
            return;
        }
    };
}
