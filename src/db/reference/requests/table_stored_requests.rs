use crate::db::entity::OndoKey;
use crate::db::entity::{TableStored, TableValue};
use crate::db::reference::TableName;
use crate::db::DbResult;

pub(crate) trait TableStoredRequests {
    fn get_table_stored(&self, cf_name: &str, key: &TableName) -> DbResult<Option<TableStored>>;
}

pub(crate) trait TableStoredIteratorRequests<'a> {
    fn all_values(
        &'a self,
        value_cf_name: &str,
    ) -> DbResult<Box<dyn Iterator<Item = DbResult<TableValue>> + 'a>>;
    fn all_values_with_key_prefix(
        &'a self,
        value_cf_name: &str,
        key_prefix: OndoKey,
    ) -> DbResult<Box<dyn Iterator<Item = DbResult<TableValue>> + 'a>>;
    fn all_values_with_key_range(
        &'a self,
        value_cf_name: &str,
        start_key: OndoKey,
        end_key: OndoKey,
    ) -> DbResult<Box<dyn Iterator<Item = DbResult<TableValue>> + 'a>>;
}

