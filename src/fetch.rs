use crate::{
    db,
    db::{articles::Article, sources},
    sources::rssatom::Fetch,
    Result,
};
use log;
use std::cmp::Ordering;

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
    let mut articles = fetch_from_source(source)?;

    // Figure out what articles we already have.
    //   http://www.詹姆斯.com/blog/2006/08/rss-dup-detection

    // Try to select articles >= the date of the oldest article,
    //   if the articles have dates
    articles.sort_unstable_by(|a, b| match (a.published, b.published) {
        (Some(apub), Some(bpub)) => apub.cmp(&bpub),
        // Some(_).cmp(&None) -> Ordering::Greater. Reverse it so `None`s go to
        // the end.
        _ => a.published.cmp(&b.published).reverse(),
    });

    let query = match articles.get(0).and_then(|a| a.published) {
        // TODO
        Some(p) => (),
        None => {
            // Select last len(Vec<Article>) from the DB w/ this source
            //   * 2, just in case the server messed up
            //
            // Articles should probably have a "retrieved" date, for that
            // purpose
        }
    };

    // TODO It's silly to pull large columns (like `content`) out of the DB when
    // doing queries like this.  Should content should go in another table?
    // Should `ArticleMetadata` or something be created out of  this query?
    //
    Ok(articles) //stub
}

fn fetch_from_source(source: &sources::Source) -> Result<Vec<Article>> {
    match serde_json::from_value(source.source_data.to_owned())? {
        sources::SourceData::RSSAtom(r) => r.fetch(source.id),
    }
}
