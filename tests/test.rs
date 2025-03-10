use yrocks::YRock;
use yrs::{Doc, GetString, Text, Transact, Update, updates::decoder::Decode};

#[test]
fn update_encoding() {
    let doc = Doc::new();

    let text = doc.get_or_insert_text("test");

    let upd = {
        let mut trx = doc.transact_mut();

        text.insert(&mut trx, 0, "abc");
        let len = text.len(&trx);
        text.insert(&mut trx, len, "def");

        trx.encode_update_v1()
    };

    let new_doc = Doc::new();

    {
        let mut trx = new_doc.transact_mut();

        trx.apply_update(Update::decode_v1(&upd).unwrap()).unwrap();
    }

    let text = new_doc.get_or_insert_text("test");
    let content = text.get_string(&new_doc.transact());

    assert_eq!(content, "abcdef")
}
