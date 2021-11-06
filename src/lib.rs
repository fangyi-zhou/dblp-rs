const PUBLICATION_API_ENDPOINT: &str = "https://dblp.org/search/publ/api";
const AUTHOR_API_ENDPOINT: &str = "https://dblp.org/search/author/api";
const VENUE_API_ENDPOINT: &str = "https://dblp.org/search/venue/api";

extern crate anyhow;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use serde::de::DeserializeOwned;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::Deserialize;
use serde_json::Value;
use std::fmt;

async fn make_request(api_endpoint: &str, query_string: &str) -> anyhow::Result<Value> {
    let client = reqwest::Client::new();
    let response = client
        .get(api_endpoint)
        .query(&[("q", query_string), ("format", "json")])
        .send()
        .await?;
    let result = response.json::<Value>().await?;
    // println!("{:?}", result);
    Ok(result["result"]["hits"].to_owned())
}

fn process_hits<T: DeserializeOwned>(hits: Value) -> anyhow::Result<Vec<T>> {
    if hits["@total"] == "0" {
        Ok(vec![])
    } else if let Value::Array(values_json) = &hits["hit"] {
        let values = values_json
            .iter()
            .map(|v| {
                // println!("{:?}", v);
                serde_json::from_value(v["info"].to_owned())
            })
            .collect::<Result<Vec<T>, _>>()?;
        Ok(values)
    } else {
        // TODO: Handle this error gracefully
        panic!()
    }
}

fn deserialise_author_in_publication<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct JsonVisitor;

    impl<'de> serde::de::Visitor<'de> for JsonVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("authors")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let _ = map.next_key::<String>()?;
            let entry = map.next_value::<serde_json::Value>()?;
            if let Value::Array(authors) = entry {
                // When there are multiple authors, the results are in an array
                let author_strs = authors
                    .iter()
                    .map(|v| v["text"].as_str().unwrap().to_owned())
                    .collect();
                Ok(author_strs)
            } else if let Value::Object(author) = entry {
                // When there is a single author, the result is as an object
                Ok(vec![author["text"].as_str().unwrap().to_owned()])
            } else {
                panic!()
            }
        }
    }
    deserializer.deserialize_any(JsonVisitor)
}

fn deserialise_venue_in_publication<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct JsonVisitor;

    impl<'de> serde::de::Visitor<'de> for JsonVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("venues")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut venues = Vec::new();
            while let Some(Value::String(venue)) = seq.next_element()? {
                venues.push(venue);
            }
            Ok(venues)
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E> {
            Ok(vec![s.to_owned()])
        }
    }
    deserializer.deserialize_any(JsonVisitor)
}

#[derive(Deserialize, Debug)]
//#[serde(deny_unknown_fields)]
pub struct Publication {
    #[serde(deserialize_with = "deserialise_author_in_publication")]
    pub authors: Vec<String>,
    pub title: String,
    #[serde(deserialize_with = "deserialise_venue_in_publication")]
    pub venue: Vec<String>,
    pub year: String,
    pub r#type: String,
    pub key: String,
    pub ee: String,
    pub url: String,
    pub access: Option<String>,
    pub publisher: Option<String>,
    pub doi: Option<String>,
    pub pages: Option<String>,
    pub volume: Option<String>,
    pub number: Option<String>,
}

/// Search for a publication, returns a JSON value
/// ```
/// # async fn publication() -> anyhow::Result<()> {
/// use dblp_rs::search_publication;
/// let result = search_publication("The Part-Time Parliament").await;
/// # Ok(()) }
/// ```
pub async fn search_publication(query_string: &str) -> anyhow::Result<Vec<Publication>> {
    let hits = make_request(PUBLICATION_API_ENDPOINT, query_string).await?;
    let pubs = process_hits(hits)?;
    Ok(pubs)
}

fn deserialise_notes_in_author<'de, D>(deserializer: D) -> Result<Vec<(String, String)>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct JsonVisitor;

    impl<'de> serde::de::Visitor<'de> for JsonVisitor {
        type Value = Vec<(String, String)>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("notes")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let _ = map.next_key::<String>()?;
            let entry = map.next_value::<serde_json::Value>()?;
            if let Value::Array(notes) = entry {
                let notes = notes
                    .iter()
                    .map(|v| {
                        (
                            v["@type"].as_str().unwrap().to_owned(),
                            v["text"].as_str().unwrap().to_owned(),
                        )
                    })
                    .collect();
                Ok(notes)
            } else if let Value::Object(note) = entry {
                Ok(vec![(
                    note["@type"].as_str().unwrap().to_owned(),
                    note["text"].as_str().unwrap().to_owned(),
                )])
            } else {
                panic!()
            }
        }
    }
    deserializer.deserialize_any(JsonVisitor)
}

fn deserialise_aliases_in_author<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct JsonVisitor;

    impl<'de> serde::de::Visitor<'de> for JsonVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("aliases")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let _ = map.next_key::<String>()?;
            let entry = map.next_value::<serde_json::Value>()?;
            if let Value::Array(aliases) = entry {
                let aliases = aliases
                    .iter()
                    .map(|v| v.as_str().unwrap().to_owned())
                    .collect();
                Ok(aliases)
            } else if let Value::String(alias) = entry {
                Ok(vec![alias])
            } else {
                panic!()
            }
        }

        fn visit_str<E>(self, s: &str) -> Result<Self::Value, E> {
            Ok(vec![s.to_owned()])
        }
    }
    deserializer.deserialize_any(JsonVisitor)
}

#[derive(Deserialize)]
//#[serde(deny_unknown_fields)]
pub struct Author {
    pub author: String,
    pub url: String,
    #[serde(deserialize_with = "deserialise_notes_in_author", default)]
    pub notes: Vec<(String, String)>,
    #[serde(deserialize_with = "deserialise_aliases_in_author", default)]
    pub aliases: Vec<String>,
}

/// Search for an author, returns a JSON value
/// ```
/// # async fn author() -> anyhow::Result<()> {
/// use dblp_rs::search_author;
/// let result = search_author("Leslie Lamport").await;
/// # Ok(()) }
/// ```
pub async fn search_author(query_string: &str) -> anyhow::Result<Vec<Author>> {
    let hits = make_request(AUTHOR_API_ENDPOINT, query_string).await?;
    let authors = process_hits(hits)?;
    Ok(authors)
}

#[derive(Deserialize)]
//#[serde(deny_unknown_fields)]
pub struct Venue {
    pub venue: String,
    pub acronym: Option<String>,
    pub r#type: String,
    pub url: String,
}

/// Search for a venue, returns a JSON value
/// ```
/// # async fn venue() -> anyhow::Result<()> {
/// use dblp_rs::search_venue;
/// let result = search_venue("TOCS").await;
/// # Ok(()) }
/// ```
pub async fn search_venue(query_string: &str) -> anyhow::Result<Vec<Venue>> {
    let hits = make_request(VENUE_API_ENDPOINT, query_string).await?;
    let venues = process_hits(hits)?;
    Ok(venues)
}

#[cfg(test)]
mod tests {
    extern crate tokio;
    use super::*;

    #[tokio::test]
    async fn integration_test_publication() {
        let result = search_publication("The Part-Time Parliament").await;
        result.unwrap();
    }

    #[tokio::test]
    async fn integration_test_more_publication() {
        let result = search_publication("proceedings").await;
        result.unwrap();
    }

    #[tokio::test]
    async fn integration_test_author() {
        let result = search_author("Leslie Lamport").await;
        result.unwrap();
    }

    #[tokio::test]
    async fn integration_test_more_author() {
        let result = search_author("Hu").await;
        result.unwrap();
    }

    #[tokio::test]
    async fn integration_test_venue() {
        let result = search_venue("TOCS").await;
        result.unwrap();
    }

    #[tokio::test]
    async fn integration_test_more_venue() {
        let result = search_venue("Transactions").await;
        result.unwrap();
    }
}
