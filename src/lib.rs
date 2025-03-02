use std::path::PathBuf;

use rocksdb::{DB, MergeOperands, Options};
use yrs::{
    Update,
    updates::{decoder::Decode, encoder::Encode},
};

pub struct YRock {
    inner: DB,
}

fn reencode_merge(_: &[u8], prev: Option<&[u8]>, op: &MergeOperands) -> Option<Vec<u8>> {
    let mut update = if let Some(prev) = prev {
        Update::decode_v1(prev).unwrap()
    } else {
        Update::new()
    };

    for op in op {
        let next_update = Update::decode_v1(op).unwrap();

        update.merge(next_update);
    }

    Some(update.encode_v1())
}

impl YRock {
    pub fn new(mut option: Options, path: &PathBuf) -> Result<Self, rocksdb::Error> {
        option.set_merge_operator_associative("yjs_update_merge", reencode_merge);

        let db = DB::open(&option, path)?;
        Ok(Self { inner: db })
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Update>, rocksdb::Error> {
        let value = self.inner.get(key)?;

        let Some(value) = value else {
            return Ok(None);
        };

        let update = Update::decode_v1(&value).expect("invalid stored data");

        Ok(Some(update))
    }

    pub fn store_update(&self, key: &[u8], update: Update) -> Result<(), rocksdb::Error> {
        self.inner.merge(key, &update.encode_v1())
    }
}
