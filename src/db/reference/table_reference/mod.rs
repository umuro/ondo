//table_reference.rs
use super::{CfNameMaker, DomainReference, Effect, Effects};
use crate::db::reference::requests::{
    DomainStoredRequests, TableStoredIteratorRequests, TableStoredRequests,
};
use crate::db::{
    entity::{table_value::TableValue, OndoKey, Table, TableStored},
    DbError, DbResult,
};
use serde::{Deserialize, Serialize};

pub(crate) mod stored;
use stored::*;

pub(crate) trait TableReferenceTrait {
    fn get_table(&self, requests: &dyn TableStoredRequests) -> DbResult<Option<Table>>;
    fn put_table(&self, table: &Table, requests: &dyn TableStoredRequests) -> DbResult<Effects>;
    fn post_table(
        &self,
        table: &Table,
        requests: &dyn TableStoredRequests,
        parent_requests: &dyn DomainStoredRequests,
    ) -> DbResult<Effects>;
    fn delete_table(
        &self,
        requests: &dyn TableStoredRequests,
        parent_requests: &dyn DomainStoredRequests,
    ) -> DbResult<Effects>;
    fn list_index_names(&self, requests: &dyn TableStoredRequests) -> DbResult<Vec<String>>;
    fn all_values<'a>(
        &self,
        requests: &'a dyn TableStoredIteratorRequests<'a>,
    ) -> DbResult<Box<dyn Iterator<Item = DbResult<TableValue>> + 'a>>;
    fn all_values_with_key_prefix<'a>(
        &self,
        key_prefix: OndoKey,
        requests: &'a dyn TableStoredIteratorRequests<'a>,
    ) -> DbResult<Box<dyn Iterator<Item = DbResult<TableValue>> + 'a>>;
    fn all_values_with_key_range<'a>(
        &self,
        start_key: OndoKey,
        end_key: OndoKey,
        requests: &'a dyn TableStoredIteratorRequests<'a>,
    ) -> DbResult<Box<dyn Iterator<Item = DbResult<TableValue>> + 'a>>;
}
// FIXME use factory instead of iterator requests.

pub type TableName = String;
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TableReference {
    pub domain_reference: DomainReference,
    pub table_name: TableName,
}

impl TableReference {
    pub fn build(domain_name: &str, table_name: &str) -> Self {
        TableReference {
            domain_reference: DomainReference::build(domain_name),
            table_name: table_name.to_string(),
        }
    }
    pub fn new(domain_reference: DomainReference, table_name: &str) -> Self {
        TableReference {
            domain_reference: domain_reference,
            table_name: table_name.to_owned(),
        }
    }

    pub fn to_domain_reference(&self) -> DomainReference {
        self.domain_reference.clone()
    }
}

impl TableReferenceTrait for TableReference {
    fn all_values<'a>(
        &self,
        requests: &'a dyn TableStoredIteratorRequests<'a>,
    ) -> DbResult<Box<dyn Iterator<Item = DbResult<TableValue>> + 'a>> {
        self.all_values_(requests)
    }

    fn all_values_with_key_prefix<'a>(
        &self,
        key_prefix: OndoKey,
        requests: &'a dyn TableStoredIteratorRequests<'a>,
    ) -> DbResult<Box<dyn Iterator<Item = DbResult<TableValue>> + 'a>> {
        self.all_values_with_key_prefix_(key_prefix, requests)
    }
    fn all_values_with_key_range<'a>(
        &self,
        start_key: OndoKey,
        end_key: OndoKey,
        requests: &'a dyn TableStoredIteratorRequests<'a>,
    ) -> DbResult<Box<dyn Iterator<Item = DbResult<TableValue>> + 'a>> {
        self.all_values_with_key_range_(start_key, end_key, requests)
    }
    fn get_table(&self, requests: &dyn TableStoredRequests) -> DbResult<Option<Table>> {
        self.get_table_stored(requests)
            .map(|opt| opt.map(|table_stored| table_stored.table))
    }

    fn put_table(&self, table: &Table, requests: &dyn TableStoredRequests) -> DbResult<Effects> {
        let stored_opt = self.get_table_stored(requests)?;
        let stored = stored_opt.ok_or(DbError::TableNotInitialized)?;
        let mut new_stored = stored.clone();
        new_stored.table = (*table).clone();
        self.put_table_stored(&new_stored)
    }

    fn post_table(
        &self,
        table: &Table,
        requests: &dyn TableStoredRequests,
        parent_requests: &dyn DomainStoredRequests,
    ) -> DbResult<Effects> {
        let stored_opt = self.get_table_stored(requests)?;
        match stored_opt {
            Some(_) => {
                // Data already exists, return AlreadyExists error
                Err(DbError::AlreadyExists)
            }
            None => {
                let new_stored = TableStored {
                    table: (*table).clone(),
                    indexes: Default::default(),
                };
                self.post_table_stored(&new_stored, parent_requests)
            }
        }
    }

    fn delete_table(
        &self,
        requests: &dyn TableStoredRequests,
        parent_requests: &dyn DomainStoredRequests,
    ) -> DbResult<Effects> {
        self.delete_table_stored(requests, parent_requests)
    }

    fn list_index_names(&self, requests: &dyn TableStoredRequests) -> DbResult<Vec<String>> {
        self.list_index_names_(requests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::reference::domain_reference::stored::tests::{
        create_domain_stored, MockDomainStoredTestRequests,
    };
    use crate::db::reference::table_reference::stored::tests::{
        create_table, create_table_ref, create_table_stored, MockTableStoredTestRequests,
    };

    mod table_reference_trait {
        use super::*;

        #[test]
        fn test_get_table_failure() {
            let mut mock = MockTableStoredTestRequests::new();
            let ref_trait = create_table_ref();
            mock.expect_get_table_stored().returning(|_, _| Ok(None));

            assert_eq!(
                ref_trait.get_table(&mock),
                Ok(None),
                "get_table should return Ok(None) if the key does not exist"
            );
        }

        #[test]
        fn test_put_table_failure() {
            let mut mock = MockTableStoredTestRequests::new();
            let ref_trait = create_table_ref();
            let table = create_table();

            mock.expect_get_table_stored()
                .returning(move |_, _| Ok(None));

            assert_eq!(
                ref_trait.put_table(&table, &mock).unwrap_err(),
                DbError::TableNotInitialized,
                "put_table should return DbError::TableNotInitialized if the key is not found"
            );
        }

        #[test]
        fn test_post_table_failure() {
            let mut mock = MockTableStoredTestRequests::new();
            let ref_trait = create_table_ref();
            let table = create_table();

            mock.expect_get_table_stored()
                .returning(move |_, _| Ok(Some(create_table_stored())));

            let mut parent_mock = MockDomainStoredTestRequests::new();
            parent_mock
                .expect_get_domain_stored()
                .returning(move |_, _| Ok(Some(create_domain_stored())));

            assert_eq!(
                ref_trait
                    .post_table(&table, &mock, &parent_mock)
                    .unwrap_err(),
                DbError::AlreadyExists,
                "post_table should return DbError::AlreadyExists if the key already exists"
            );
        }
    }
}

// TODO:TEST ID Indexing, test all_values_with_key_prefix, test all_values_with_key_range
