use crate::{
    db,
    db::{articles, articles::Article, sources},
    sources::rssatom::SourceData,
    Result,
};

const MAX_FETCH_ERRORS: usize = 10;

pub fn fetch_new_from_all_sources(pool: &mut db::Pool) -> Result<()> {
    let conn = db::DbConn(pool.get()?);
    let mut sources = sources::get_for_fetch(&*conn)?;

    for source in &mut sources {
        let new_articles = match fetch_new_from_source(&conn, &source) {
            Ok(articles) => articles,
            Err(e) => {
                // Likely communication problems when connecting to the source
                source.fetch_errors.push(format!(
                    "{}: {}",
                    time::at_utc(source.last_fetch_started.0).rfc822(),
                    e
                ));
                // Only keep the latest errors
                if source.fetch_errors.len() > MAX_FETCH_ERRORS {
                    source.fetch_errors.drain(
                        0..(source.fetch_errors.len() - MAX_FETCH_ERRORS),
                    );
                }
                // TODO only update fetch_errors
                sources::update(source, &conn)?;
                continue;
            }
        };

        for article in new_articles {
            articles::insert(article, &conn)?;
        }
        source.last_successful_fetch = source.last_fetch_started;
        source.fetching = false;
        // TODO only update last_successful_fetch, & fetching
        sources::update(source, &conn)?;
    }

    Ok(())
}

fn fetch_new_from_source(
    conn: &db::DbConn,
    source: &sources::Source,
) -> Result<Vec<Article>> {
    let source_data = serde_json::from_value(source.source_data.to_owned())?;
    let mut fetched_articles = fetch_from_source(&source_data)?;

    match source_data {
        sources::SourceData::RSSAtom(r) => {
            r.unique(&mut fetched_articles, conn)?
        }
    };
    Ok(fetched_articles)
}

fn fetch_from_source(
    source_data: &sources::SourceData,
) -> Result<Vec<Article>> {
    match source_data {
        sources::SourceData::RSSAtom(r) => r.fetch(),
    }
}
