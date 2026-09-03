//! Regression test pinning schema derivation over a heterogeneous collection.
//!
//! `Document::union` is not a pure lattice join: it carries a Jaccard index whose
//! `num_unions` counter (against `MAX_NUM_DOC_UNIONS`) decides when a schema becomes
//! unstable, and `derive_schema_for_partition` stops early once it is. The derived
//! schema therefore depends on *how* the per-document schemas are associated, not
//! just on which documents were seen.
//!
//! Every other test in this suite uses one or two document shapes, where the
//! accumulated schema stays the same size as any single document's schema and the
//! association order cannot be observed. This test uses documents that each
//! contribute unique field names, so the accumulated schema grows and a change to
//! the association order shows up as a different derived schema.

use crate::{derive_schema_for_collection, internal_integration_tests::create_mdb_client};
use bson::{Document, doc};
use std::collections::BTreeSet;

const DB_NAME: &str = "heterogeneous_regression";
const COLL_NAME: &str = "unique_fields_per_document";

const NUM_DOCS: i64 = 200;
const FIELDS_PER_DOC: i64 = 5;

/// Derivation stops after the first batch of PARTITION_DOCS_PER_ITERATION documents,
/// because unioning that many dissimilar shapes drives the schema unstable.
const DOCS_BEFORE_UNSTABLE: i64 = 20;

#[cfg(feature = "integration")]
#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn heterogeneous_documents_derive_a_stable_prefix() {
    let client = create_mdb_client().await;
    let coll = client.database(DB_NAME).collection::<Document>(COLL_NAME);
    coll.drop().await.unwrap();
    coll.insert_many((0..NUM_DOCS).map(|i| {
        let mut doc = doc! {"_id": i};
        for j in 0..FIELDS_PER_DOC {
            doc.insert(format!("u{i}_{j}"), i as i32);
        }
        doc
    }))
    .await
    .unwrap();

    let service = crate::data_service::MongoDbDataService::new(client);
    let derived = derive_schema_for_collection(&service, DB_NAME, COLL_NAME, None)
        .await
        .unwrap();

    coll.drop().await.unwrap();

    assert!(
        derived.is_unstable(),
        "unioning {DOCS_BEFORE_UNSTABLE} dissimilar documents should mark the schema unstable"
    );

    let expected = std::iter::once("_id".to_string())
        .chain(
            (0..DOCS_BEFORE_UNSTABLE)
                .flat_map(|i| (0..FIELDS_PER_DOC).map(move |j| format!("u{i}_{j}"))),
        )
        .collect::<BTreeSet<_>>();

    let as_bson = Document::try_from(derived).unwrap();
    let actual = as_bson
        .get_document("$jsonSchema")
        .unwrap()
        .get_document("properties")
        .unwrap()
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();

    assert_eq!(
        expected, actual,
        "derived properties differ; if this changed, check whether the order in which \
         per-document schemas are unioned was altered"
    );
}
