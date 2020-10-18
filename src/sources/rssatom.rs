use crate::{
    db::articles::{Article, ArticleSource},
    schema::articles,
    Result,
};
use atom_syndication;
use diesel::prelude::*;
use log;
use rfc822_sanitizer;
use rss;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{error::Error, fmt, io::BufReader};
use uuid::Uuid;

/// Methods specific to a kind of source (ex: RSS)
pub trait SourceData {
    // Pull available articles from a source.
    fn fetch(&self, source_id: Uuid) -> Result<Vec<Article>>;
    // Remove articles from a list that already exist in the db.
    fn unique(
        &self,
        source_id: Uuid,
        articles: &mut Vec<Article>,
        conn: &PgConnection,
    ) -> Result<()>;
}

#[derive(Debug)]
pub struct RSSFetchError {
    rss_error: rss::Error,
    atom_error: atom_syndication::Error,
}

impl fmt::Display for RSSFetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RSS: ({}) Atom: ({})", self.rss_error, self.atom_error)
    }
}

impl Error for RSSFetchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.rss_error.source()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RSSAtom {
    url: String,
}

impl SourceData for RSSAtom {
    fn fetch(&self, source_id: Uuid) -> Result<Vec<Article>> {
        let resp = reqwest::blocking::get(&self.url).and_then(|r| r.text())?;

        let rss_err =
            match rss::Channel::read_from(BufReader::new(resp.as_bytes())) {
                Err(e) => e,
                Ok(channel) => {
                    return Ok(channel
                        .into_items()
                        .into_iter()
                        .map(|item| {
                            RSSAtom::rss_item_to_article(&item, source_id)
                        })
                        .collect())
                }
            };

        let atom_err = match atom_syndication::Feed::read_from(BufReader::new(
            resp.as_bytes(),
        )) {
            Err(e) => e,
            Ok(feed) => {
                return Ok(feed
                    .entries
                    .into_iter()
                    .map(|entry| {
                        RSSAtom::atom_entry_to_article(&entry, source_id)
                    })
                    .collect())
            }
        };

        Err(Box::new(RSSFetchError {
            rss_error: rss_err,
            atom_error: atom_err,
        }))
    }

    fn unique(
        &self,
        source_id: Uuid,
        articles: &mut Vec<Article>,
        conn: &PgConnection,
    ) -> Result<()> {
        // http://www.詹姆斯.com/blog/2006/08/rss-dup-detection
        // Using this heirarchy: GUID -> link -> Title -> Desc -> Content
        //  Pick at least 2 that aren't empty, and filter for those.

        let mut indexes_to_keep: Vec<bool> = Vec::new();

        for article in articles.iter() {
            let mut query = articles::table
                .select(articles::id)
                .filter(articles::source.eq(&source_id))
                .into_boxed();
            let mut filters = 0;
            if let Some(id) = &article.id_from_source {
                query = query.filter(articles::id_from_source.eq(id));
                filters += 1;
            };

            let deserialized_links: std::result::Result<
                Vec<String>,
                serde_json::Error,
            > = serde_json::from_value(article.links.to_owned());
            if let Ok(links) = deserialized_links {
                if links.len() >= 1 {
                    query = query
                        .filter(articles::links.eq(article.links.to_owned()));
                    filters += 1;
                }
            };

            if filters < 2 {
                if let Some(title) = &article.title {
                    query = query.filter(articles::title.eq(title));
                    filters += 1;
                };
            }

            if filters < 2 {
                if let Some(summary) = &article.summary {
                    query = query.filter(articles::summary.eq(summary));
                    filters += 1;
                };
            }

            if filters < 2 {
                query = query
                    .filter(articles::content.eq(article.content.to_owned()));
            }

            let similar_articles: Vec<Uuid> = query.load(conn)?;
            // Matches 1 or more(?) articles, don't return it
            indexes_to_keep.push(similar_articles.len() == 0);
        }

        let mut i = 0;
        articles.retain(|_| (indexes_to_keep[i], i += 1).0);

        Ok(())
    }
}

impl RSSAtom {
    fn rss_item_to_article(item: &rss::Item, source_id: Uuid) -> Article {
        let ts = item
            .pub_date()
            .map(|s| rfc822_sanitizer::parse_from_rfc2822_with_fallback(s).ok())
            .flatten()
            .map(|datetime| time::Timespec {
                sec: datetime.timestamp(),
                nsec: 0,
            });
        // Did we get a date, but not a result?
        match (item.pub_date(), ts) {
            (Some(date), None) => log::debug!(
                "Could not parse from source {} as date: {}",
                source_id,
                date
            ),
            _ => (),
        };

        let source_info = match item
            .source()
            .map(|source| RSSAtom::rss_source_to_article_source(source))
            .map(|source| serde_json::to_value(source))
        {
            Some(Ok(v)) => v,
            Some(Err(e)) => {
                log::warn!("Couldn't serialize source: {}", e);
                serde_json::json!([])
            }
            None => serde_json::json!([]),
        };

        Article {
            id: Uuid::new_v4(),
            title: item.title().map(|s| s.to_string()),
            published: ts,
            source_info: source_info,
            summary: None,
            content: serde_json::json!({"value": item.content().map(|s| s.to_string())}),
            rights: None,
            links: serde_json::to_value(opt_to_vector(
                item.link().map(|s| s.to_string()),
            ))
            .unwrap(),
            authors: serde_json::to_value(opt_to_vector(
                item.author().map(|s| s.to_string()),
            ))
            .unwrap(),
            // name -> term
            // domain -> scheme
            categories: item
                .categories()
                .into_iter()
                .map(|c| c.name().to_string())
                .collect(),
            comments_url: item.comments().map(|s| s.to_string()),
            // TODO serialize Extension
            extensions: serde_json::to_value(item.extensions())
                .unwrap_or(serde_json::json!({})),
            source: source_id,
            id_from_source: item
                .guid()
                .and_then(|guid| Some(guid.value().to_string())),
        }
    }

    fn atom_entry_to_article(
        entry: &atom_syndication::Entry,
        source_id: Uuid,
    ) -> Article {
        Article {
            id: Uuid::new_v4(),
            title: Some(entry.title().to_string()),
            published: entry.published().map(|datetime| time::Timespec {
                sec: datetime.timestamp(),
                nsec: 0,
            }),
            // TODO serialize Source
            source_info: serde_json::to_value(opt_to_vector(entry.source()))
                .unwrap(),
            summary: entry.summary().map(|s| s.to_string()),
            content: serde_json::to_value(entry.content())
                .unwrap_or(serde_json::json!({})),
            rights: entry.rights().map(|s| s.to_string()),
            links: serde_json::to_value(&entry.links)
                .unwrap_or(serde_json::json!([])),
            authors: serde_json::to_value(&entry.authors)
                .unwrap_or(serde_json::json!([])),
            categories: serde_json::to_value(&entry.categories)
                .unwrap_or(serde_json::json!([])),
            comments_url: None,
            extensions: serde_json::to_value(entry.extensions())
                .unwrap_or(serde_json::json!({})),
            source: source_id,
            id_from_source: Some(entry.id.to_owned()),
        }
    }

    fn rss_source_to_article_source(source: &rss::Source) -> ArticleSource {
        ArticleSource {
            title: source.title().map(|s| s.to_string()),
            links: vec![source.url().to_string()],
        }
    }
}

fn opt_to_vector<T>(o: Option<T>) -> Vec<T> {
    o.into_iter().collect::<Vec<T>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fetch_example_rss() {
        let source_id = Uuid::new_v4();
        let good_rss = RSSAtom{url: "http://rssfeeds.democratandchronicle.com/Democratandchronicle/news".to_string()};
        let articles = good_rss.fetch(source_id).unwrap();
        for a in &articles {
            assert!(a.source == source_id);
            println!("{:#?}", a);
        }
    }

    #[test]
    fn fetch_bad_rss() {
        let source_id = Uuid::new_v4();
        let bad_rss = RSSAtom {
            url: "http://example.com".to_string(),
        };
        bad_rss.fetch(source_id).unwrap_err();
    }
}
