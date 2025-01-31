use crate::db::db_error::{DbError, DbResult};
use crate::db::entity::OndoKey;
use crate::db::entity::TableValue;
use crate::db::reference::requests::TableValueRequests;
use crate::db::reference::TableValueReference;
use crate::db::server::rocks_db_accessor::RocksDbAccessor;
use crate::db::server::source_sink::ondo_serializer::OndoSerializer;
use crate::db::DbError::CfNotFound;
use serde_json::Value;

impl TableValueRequests for RocksDbAccessor {
    fn get_table_value(
        &self,
        cf_name: &str,
        key: &TableValueReference,
    ) -> DbResult<Option<TableValue>> {
        let guarded_db = self.guarded_db();
        let db = RocksDbAccessor::db_read_lock(&guarded_db)?;
        let cf = db.cf_handle(cf_name).ok_or(CfNotFound)?;
        let ondo_key = OndoKey::ondo_serialize(&key.id)?;
        // println!("DEBUG: Fetching table value with key: {:?}", ondo_key);
        let answer = db
            .get_cf(cf, &ondo_key)
            .map_err(|err| DbError::RocksDbError(err))?;
        answer
            .map(|bytes| Value::ondo_deserialize(&bytes))
            .transpose()
    }
}
