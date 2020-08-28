use crate::db;
use crate::db::sources;

pub fn fetch_from_all_sources(pool: &mut db::Pool) {
    let conn = match pool.get() {
        Ok(conn) => db::DbConn(conn),
        Err(e) => return
    };
    sources::all(&*conn);
}
