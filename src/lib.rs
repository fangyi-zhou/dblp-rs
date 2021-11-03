const PUBLICATION_API_ENDPOINT: &str = "https://dblp.org/search/publ/api";
const AUTHOR_API_ENDPOINT: &str = "https://dblp.org/search/author/api";
const VENUE_API_ENDPOINT: &str = "https://dblp.org/search/venue/api";

extern crate anyhow;
extern crate reqwest;
extern crate serde_json;

async fn make_request(api_endpoint: &str, query_string: &str) -> anyhow::Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let response = client
        .get(api_endpoint)
        .query(&[("q", query_string), ("format", "json")])
        .send()
        .await?;
    let result = response.json::<serde_json::Value>().await?;
    //println!("{:?}", result);
    Ok(result)
}

/// Search for a publication, returns a JSON value
/// ```
/// # async fn publication() -> anyhow::Result<()> {
/// use dblp_rs::search_publication;
/// let result = search_publication("The Part-Time Parliament").await;
/// # Ok(()) }
/// ```
pub async fn search_publication(query_string: &str) -> anyhow::Result<serde_json::Value> {
    make_request(PUBLICATION_API_ENDPOINT, query_string).await
}

/// Search for an author, returns a JSON value
/// ```
/// # async fn author() -> anyhow::Result<()> {
/// use dblp_rs::search_author;
/// let result = search_author("Leslie Lamport").await;
/// # Ok(()) }
/// ```
pub async fn search_author(query_string: &str) -> anyhow::Result<serde_json::Value> {
    make_request(AUTHOR_API_ENDPOINT, query_string).await
}

/// Search for a venue, returns a JSON value
/// ```
/// # async fn venue() -> anyhow::Result<()> {
/// use dblp_rs::search_venue;
/// let result = search_venue("TOCS").await;
/// # Ok(()) }
/// ```
pub async fn search_venue(query_string: &str) -> anyhow::Result<serde_json::Value> {
    make_request(VENUE_API_ENDPOINT, query_string).await
}

#[cfg(test)]
mod tests {
    extern crate tokio;
    use super::*;

    #[tokio::test]
    async fn integration_test_publicaion() {
        let result = search_publication("The Part-Time Parliament").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn integration_test_author() {
        let result = search_author("Leslie Lamport").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn integration_test_venue() {
        let result = search_venue("TOCS").await;
        assert!(result.is_ok());
    }
}
