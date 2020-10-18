use crate::{
    db,
    db::{articles::Article, sources},
    sources::rssatom::SourceData,
    Result,
};
use log;
use uuid::Uuid;

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
        let new_articles = match fetch_new_from_source(&conn, &source) {
            Ok(articles) => articles,
            Err(e) => {
                // Likely communication problems when connecting to the source
                // TODO update the source with `e`
                continue;
            }
        };
        // TODO add new articles
    }
}

fn fetch_new_from_source(
    conn: &db::DbConn,
    source: &sources::Source,
) -> Result<Vec<Article>> {
    let source_data = serde_json::from_value(source.source_data.to_owned())?;
    let mut fetched_articles = fetch_from_source(&source_data, source.id)?;

    match source_data {
        sources::SourceData::RSSAtom(r) => {
            r.unique(source.id, &mut fetched_articles, conn)?
        }
    };
    Ok(fetched_articles)
}

fn fetch_from_source(
    source_data: &sources::SourceData,
    source_id: Uuid,
) -> Result<Vec<Article>> {
    match source_data {
        sources::SourceData::RSSAtom(r) => r.fetch(source_id),
    }
}
