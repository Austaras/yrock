use std::path::Path;

use rocksdb::{DB, MergeOperands, Options};
use yrs::{
    Doc, ReadTxn, StateVector, Transact, Update, merge_updates_v1,
    updates::{decoder::Decode, encoder::Encode},
};

pub struct YRocks {
    inner: DB,
}

fn partial_merge(key: &[u8], prev: Option<&[u8]>, op: &MergeOperands) -> Option<Vec<u8>> {
    if op.len() < 256 {
        return None;
    }

    reencode_merge(key, prev, op)
}

fn reencode_merge(_: &[u8], prev: Option<&[u8]>, op: &MergeOperands) -> Option<Vec<u8>> {
    let iter = prev.into_iter().chain(op.into_iter());

    merge_updates_v1(iter).ok()
}

impl YRocks {
    pub fn new(mut option: Options, path: impl AsRef<Path>) -> Result<Self, rocksdb::Error> {
        option.set_merge_operator("yjs_update_merge", reencode_merge, partial_merge);

        let db = DB::open(&option, path)?;
        Ok(Self { inner: db })
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<yrs::Doc>, rocksdb::Error> {
        let value = self.inner.get(key)?;

        let Some(value) = value else {
            return Ok(None);
        };

        let update = Update::decode_v1(&value).expect("invalid stored data");

        let doc = yrs::Doc::new();

        {
            let mut trx = doc.transact_mut();
            trx.apply_update(update).expect("invalid stored data");
        }

        Ok(Some(doc))
    }

    pub fn set(&self, key: &[u8], doc: &Doc) -> Result<(), rocksdb::Error> {
        let update = doc
            .transact()
            .encode_state_as_update_v1(&StateVector::default());

        self.inner.put(key, update)
    }

    pub fn store_update(&self, key: &[u8], update: Update) -> Result<(), rocksdb::Error> {
        self.inner.merge(key, update.encode_v1())
    }

    pub fn store_encoded(&self, key: &[u8], bin: &[u8]) -> Result<(), rocksdb::Error> {
        self.inner.merge(key, bin)
    }
}
