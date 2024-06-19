
use std::collections::HashMap;
use substreams_entity_change::tables::Row;
use substreams_entity_change::tables::Tables;
use substreams_entity_change::pb::entity::EntityChanges;

pub struct TablesWithIncrementingKey {
    tables: Tables,
    prefix: String,
    counters: HashMap<String, u64>,
}

impl TablesWithIncrementingKey {
    pub fn new() -> Self {
        TablesWithIncrementingKey {
            tables: Tables::new(),
            prefix: "".to_string(),
            counters: HashMap::new(),
        }
    }
    pub fn set_prefix_and_reset_counters (&mut self, prefix: String) {
        self.prefix = prefix;
        self.counters.clear();
    } 

    pub fn create_row_with_incrementing_key(&mut self, table: &str) -> &mut Row {
        let counter = self.counters.entry(table.to_string()).and_modify(|c| {*c+=1}).or_insert(1);
        let key =  format!("{}-{}", self.prefix, counter);

        self.tables.create_row(table, key)
    }

    pub fn log_error(&mut self, error: &str) {
        self.create_row_with_incrementing_key("Error").set("description", error);
    }

    pub fn to_entity_changes(self) -> EntityChanges {
        self.tables.to_entity_changes()
    }
}