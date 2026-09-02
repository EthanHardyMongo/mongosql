//! Regression tests for documents whose field names contain a `.`.
//!
//! A document with a dotted field name never matches a `$jsonSchema` derived from
//! itself: the `required` keyword resolves `"a.b"` as the path `a` -> `b`, which is
//! absent, so the constraint can never be satisfied. Such a document is therefore
//! always returned by the exclusion query in `derive_schema_for_partition`, and the
//! derivation loop can spin forever on it.

use crate::{derive_schema_for_collection, internal_integration_tests::create_mdb_client};
use bson::{Document, doc};
use std::time::Duration;

const DB_NAME: &str = "dotted_field_regression";

/// Deriving a schema should never take anywhere near this long for a handful of
/// tiny documents; exceeding it means the derivation loop is not terminating.
const TIMEOUT: Duration = Duration::from_secs(30);

#[allow(clippy::unwrap_used)]
async fn assert_derivation_terminates(coll_name: &str, docs: Vec<Document>) {
    let client = create_mdb_client().await;
    let db = client.database(DB_NAME);
    let coll = db.collection::<Document>(coll_name);
    coll.drop().await.unwrap();
    coll.insert_many(docs).await.unwrap();

    let service = crate::data_service::MongoDbDataService::new(client);
    let derived = tokio::time::timeout(
        TIMEOUT,
        derive_schema_for_collection(&service, DB_NAME, coll_name, None),
    )
    .await;

    coll.drop().await.unwrap();

    match derived {
        Err(_) => panic!(
            "derive_schema_for_collection did not terminate within {TIMEOUT:?} for `{DB_NAME}.{coll_name}`"
        ),
        Ok(Err(err)) => panic!("unexpected error: {err:?}"),
        Ok(Ok(_)) => {}
    }
}

macro_rules! test_derivation_terminates {
    ($test_name:ident, docs = $docs:expr) => {
        #[cfg(feature = "integration")]
        #[tokio::test]
        async fn $test_name() {
            super::derive_schema_dotted_fields::assert_derivation_terminates(
                stringify!($test_name),
                $docs,
            )
            .await
        }
    };
}

// A collection holding a single document is the guaranteed instance of the bug: the
// document is the only member of its batch, so it is never added to `ignored_ids`,
// and the partition minimum never advances past it.
test_derivation_terminates!(
    single_dotted_document,
    docs = vec![doc! {"_id": 0, "a.b": 1}]
);

test_derivation_terminates!(
    single_nested_dotted_document,
    docs = vec![doc! {"_id": 0, "a": {"b.c": 1}}]
);

test_derivation_terminates!(
    single_dotted_document_in_array,
    docs = vec![doc! {"_id": 0, "a": [{"b.c": 1}]}]
);

// More generally, the loop fails to terminate whenever the trailing batch of a
// partition holds exactly one document that cannot match its own schema, i.e. when
// the number of such documents is congruent to 1 modulo PARTITION_DOCS_PER_ITERATION.
test_derivation_terminates!(
    dotted_documents_one_over_a_full_batch,
    docs = (0..21).map(|i| doc! {"_id": i, "a.b": i}).collect()
);

test_derivation_terminates!(
    dotted_documents_two_over_a_full_batch,
    docs = (0..41).map(|i| doc! {"_id": i, "a.b": i}).collect()
);

// Controls: neither of these should ever have been affected.
test_derivation_terminates!(
    dotted_documents_filling_whole_batches,
    docs = (0..40).map(|i| doc! {"_id": i, "a.b": i}).collect()
);

test_derivation_terminates!(
    single_document_without_dotted_fields,
    docs = vec![doc! {"_id": 0, "a": 1}]
);
