use std::sync::Arc;

use rocksdb::Options;
use tempfile::TempDir;
use yrocks::YRocks;
use yrs::{Doc, GetString, Text, Transact};

fn init() -> (YRocks, TempDir) {
    let tempdir = tempfile::Builder::new().prefix("_temp").tempdir().unwrap();
    let path = tempdir.path();
    let mut opts = Options::default();
    opts.create_if_missing(true);

    (YRocks::new(opts, path).unwrap(), tempdir)
}

static DOC_NAME: &[u8] = b"test";

static TEXT_NAME: &str = "test";

#[test]
fn insert_twice() {
    let (db, dir) = init();

    let doc = Doc::new();

    let text = doc.get_or_insert_text(TEXT_NAME);

    let upd = {
        let mut trx = doc.transact_mut();

        text.insert(&mut trx, 0, "abc");
        let len = text.len(&trx);
        text.insert(&mut trx, len, "def");

        trx.encode_update_v1()
    };

    db.store_encoded(DOC_NAME, &upd).unwrap();

    let upd = {
        let mut trx = doc.transact_mut();
        let len = text.len(&trx);
        text.insert(&mut trx, len, "111");

        trx.encode_update_v1()
    };

    db.store_encoded(DOC_NAME, &upd).unwrap();

    let new_doc = db.get(DOC_NAME).unwrap().unwrap();

    let text = new_doc.get_or_insert_text(TEXT_NAME);
    let content = text.get_string(&new_doc.transact());

    assert_eq!(content, "abcdef111");

    drop(dir)
}

#[test]
fn incremental_updates() {
    let (db, dir) = init();
    let db = Arc::new(db);

    // store document updates
    {
        let db = db.clone();
        let doc = Doc::new();
        let text = doc.get_or_insert_text(TEXT_NAME);

        let _sub = doc.observe_update_v1(move |_, u| {
            db.store_encoded(DOC_NAME, &u.update).unwrap();
        });
        // generate 3 updates
        text.push(&mut doc.transact_mut(), "a");
        text.push(&mut doc.transact_mut(), "b");
        text.push(&mut doc.transact_mut(), "c");
    }

    // load document
    {
        let doc = db.get(DOC_NAME).unwrap().unwrap();
        let text = doc.get_or_insert_text(TEXT_NAME);
        let txn = doc.transact();

        assert_eq!(text.get_string(&txn), "abc");
    }

    drop(dir)
}
