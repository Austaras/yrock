use rocksdb::Options;
use yrocks::YRock;
use yrs::{Doc, GetString, Text, Transact};

#[test]
fn update_encoding() {
    let tempdir = tempfile::Builder::new().prefix("_temp").tempdir().unwrap();
    let path = tempdir.path();
    let mut opts = Options::default();
    opts.create_if_missing(true);

    let db = YRock::new(opts, path).unwrap();

    let doc = Doc::new();

    let text = doc.get_or_insert_text("test");

    let upd = {
        let mut trx = doc.transact_mut();

        text.insert(&mut trx, 0, "abc");
        let len = text.len(&trx);
        text.insert(&mut trx, len, "def");

        trx.encode_update_v1()
    };

    db.store_encoded(b"test", &upd).unwrap();

    let upd = {
        let mut trx = doc.transact_mut();
        let len = text.len(&trx);
        text.insert(&mut trx, len, "111");

        trx.encode_update_v1()
    };

    db.store_encoded(b"test", &upd).unwrap();

    let new_doc = db.get(b"test").unwrap().unwrap();

    let text = new_doc.get_or_insert_text("test");
    let content = text.get_string(&new_doc.transact());

    assert_eq!(content, "abcdef111")
}
