use crate::{
    db,
    db::{articles::Article, sources},
    sources::rssatom::Fetch,
    Result,
};
use log;

pub fn fetch_new_from_all_sources(pool: &mut db::Pool) {
    let conn = match pool.get() {
        Ok(conn) => db::DbConn(conn),
        Err(e) => {
            log::error!("{}", e);
            return;
        }
    };
    let sources = match sources::all(&*conn) {
        Ok(srcs) => srcs,
        Err(e) => {
            log::error!("{}", e);
            return;
        }
    };
    for source in sources {
        let _ = fetch_new_from_source(&conn, &source);
    }
}

fn fetch_new_from_source(
    conn: &db::DbConn,
    source: &sources::Source,
) -> Result<Vec<Article>> {
    fetch_from_source(conn, source)
}

fn fetch_from_source(
    conn: &db::DbConn,
    source: &sources::Source,
) -> Result<Vec<Article>> {
    match serde_json::from_value(source.source_data.to_owned())? {
        sources::SourceData::RSSAtom(r) => r.fetch(source.id),
    }
}
