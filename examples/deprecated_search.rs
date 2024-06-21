#![allow(deprecated)]

use anyhow::Result;
use qdrant_client::prelude::*;
use qdrant_client::qdrant::{
    Condition, CreateCollectionBuilder, Distance, Filter, PayloadIncludeSelector, QuantizationType,
    ScalarQuantizationBuilder, SearchParamsBuilder, SearchPointsBuilder, VectorParamsBuilder,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Example of top level client
    // You may also use tonic-generated client from `src/qdrant.rs`
    let config = QdrantClientConfig::from_url("http://localhost:6334");
    let client = QdrantClient::new(Some(config))?;

    let collections_list = client.list_collections().await?;
    dbg!(collections_list);
    // collections_list = ListCollectionsResponse {
    //     collections: [
    //         CollectionDescription {
    //             name: "test",
    //         },
    //     ],
    //     time: 1.78e-6,
    // }

    let collection_name = "test";
    client.delete_collection(collection_name).await?;

    client
        .create_collection(
            &CreateCollectionBuilder::default()
                .collection_name(collection_name)
                .vectors_config(VectorParamsBuilder::new(300, Distance::Cosine))
                .quantization_config(ScalarQuantizationBuilder::new(QuantizationType::Int8))
                .build(),
        )
        .await?;

    let collection_info = client.collection_info(collection_name).await?;
    dbg!(collection_info);

    let payload: Payload = json!(
        {
            "foo": "Bar",
            "bar": 12,
            "baz": {
                "qux": "quux"
            }
        }
    )
    .try_into()
    .unwrap();

    let points = vec![PointStruct::new(0, vec![12.; 10], payload)];
    client
        .upsert_points_blocking(collection_name, None, points, None)
        .await?;

    let search_point_req = SearchPointsBuilder::new(collection_name, [11.; 10], 10)
        .filter(Filter::all([Condition::matches("bar", 12)]))
        .with_payload(PayloadIncludeSelector { fields: vec![] })
        .params(SearchParamsBuilder::default().exact(true))
        .build();

    let search_result = client.search_points(&search_point_req).await?;

    dbg!(&search_result);
    // search_result = SearchResponse {
    //     result: [
    //         ScoredPoint {
    //             id: Some(
    //                 PointId {
    //                     point_id_options: Some(
    //                         Num(
    //                             0,
    //                         ),
    //                     ),
    //                 },
    //             ),
    //             payload: {
    //                 "bar": Value {
    //                     kind: Some(
    //                         IntegerValue(
    //                     12,
    //                     ),
    //                     ),
    //                 },
    //                 "foo": Value {
    //                     kind: Some(
    //                         StringValue(
    //                     "Bar",
    //                     ),
    //                     ),
    //                 },
    //             },
    //             score: 1.0000001,
    //             version: 0,
    //             vectors: None,
    //         },
    //     ],
    //     time: 9.5394e-5,
    // }

    let found_point = search_result.result.into_iter().next().unwrap();
    let mut payload = found_point.payload;
    let baz_payload = payload.remove("baz").unwrap().into_json();
    println!("baz: {}", baz_payload);
    // baz: {"qux":"quux"}

    Ok(())
}