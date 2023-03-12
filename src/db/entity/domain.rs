//domain.rs
use super::reference::DomainReference;
use::std::collections::HashMap;

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct Domain {
    pub id: DomainReference
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub(crate) struct DomainStored {
    pub domain: Domain,
    pub tables: HashMap<String, ()>
}
