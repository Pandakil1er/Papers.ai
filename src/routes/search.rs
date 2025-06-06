use actix_web::{get, web, HttpResponse, Responder};
use elasticsearch::{Elasticsearch, SearchParts};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageSearchResult {
    pub uuid: String,
    pub name: String,
    pub summary: String,
}

#[get("/search")]
pub async fn search(
    es: web::Data<Elasticsearch>,
    web::Query(query): web::Query<SearchQuery>,
) -> impl Responder {
    let search_body = json!({
        "size": 50,
        "_source": ["uuid", "name", "summary"],
        "query": {
            "multi_match": {
                "query": query.q,
                "fields": ["name^2", "summary", "keywords"],
                "fuzziness": "AUTO",
                "operator": "and"
            }
        }
    });

    match es
        .search(SearchParts::Index(&["papers"]))
        .body(search_body)
        .send()
        .await
    {
        Ok(res) => {
            // Parse the JSON response body
            let response_body = res.json::<Value>().await.unwrap_or_default();
            let hits = response_body["hits"]["hits"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|hit| {
                    // Extract the `_source` object as a Value
                    let source: Value = hit["_source"].clone();
                    // Deserialize into ImageSearchResult
                    serde_json::from_value::<ImageSearchResult>(source).ok()
                })
                .collect::<Vec<_>>();

            HttpResponse::Ok().json(hits)
        }
        Err(err) => {
            HttpResponse::InternalServerError().body(format!("Elasticsearch error: {}", err))
        }
    }
}
