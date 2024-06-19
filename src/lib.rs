mod pb;

use std::collections::HashMap;

use substreams_database_change::change::AsString;
use anyhow::Result;
use base64::prelude::*;
use borsh::BorshDeserialize;
use substreams_entity_change::pb::entity::EntityChanges;
use substreams_entity_change::tables::Row;
use substreams_entity_change::tables::Tables;
use substreams_entity_change::tables::ToValue;
use substreams_solana::pb::sf::solana::r#type::v1::Block;
use substreams_solana::pb::sf::solana::r#type::v1::UnixTimestamp;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<EntityChanges, substreams::errors::Error> {
    let mut tables = TablesWithIncrementingKey::new();

    let program_id = "6aw4sBovP6yaG1q4y2GpjaQcLZJbBWMJP4aJFsLKxgb3";
    let log_event_prefix = "Program data: ";
    let start_log_message = format!("Program {program_id} invoke");
    let end_log_message = format!("Program {program_id} success");

    for tx in block.transactions.iter() {
        let Some(transaction) = &tx.transaction else {
            continue;
        };
        let tx_sig = bs58::encode(&transaction.signatures[0]).into_string();
        tables.set_prefix_and_reset_counters(tx_sig);
        let Some(meta) = tx.meta.as_ref() else {
            continue;
        };

        let mut in_program = false;

        for log_message in meta.log_messages.iter() {
            if log_message.starts_with(start_log_message.as_str()) {
                in_program = true;
            } else if log_message.starts_with(end_log_message.as_str()) {
                in_program = false;
            } else if in_program && log_message.starts_with(log_event_prefix) {
                let Some(message) = log_message.strip_prefix(log_event_prefix) else {
                    continue;
                };
                //------------------- Parse message to the event ---------------------------------
                let Ok(base64_decoded_message) = BASE64_STANDARD.decode(message) else {
                    tables.log_error( "Error decoding base64");
                    continue;
                };
                if base64_decoded_message.len() < 8 {
                    tables.log_error("Decoded message too short");
                    continue;
                }
                let discriminator = &base64_decoded_message[0..8];
                let serialized_event = &base64_decoded_message[8..];

                match discriminator {
                    b"\x3e\xcd\xf2\xaf\xf4\xa9\x88\x34" => {
                        let event = borsh::from_slice::<Deposit>(serialized_event).unwrap();
                        tables.create_row_with_incrementing_key("Deposit")
                        .set_if_some("timestamp", block.block_time.as_ref().map(|x|{x.timestamp}))
                        .set("user", event.user.to_string())
                        .set("amount", event.amount)
                        .set("total_amount", event.total_amount)
                        .set("lock_expires", event.lock_expires)
                        .set("referrer", event.referrer.to_string());
                    }
                    _ => {
                        tables.log_error("Discriminator does not match known events");
                    }
                }
            }
        }
    }

    Ok(tables.to_entity_changes())
}

trait SetIfSome {
    fn set_if_some<T: ToValue>(&mut self, name: &str, value: Option<T>) -> &mut Self;
}
impl SetIfSome for substreams_entity_change::tables::Row {
    fn set_if_some<T: ToValue>(&mut self, name: &str, value: Option<T>) -> &mut Self {
        match value {
           Some(value) => self.set(name, value),
           None => self 
        }
    }
}

struct TablesWithIncrementingKey {
    tables: Tables,
    prefix: String,
    counters: HashMap<String, u64>,
}
impl TablesWithIncrementingKey {
    fn new() -> Self {
        TablesWithIncrementingKey {
            tables: Tables::new(),
            prefix: "".to_string(),
            counters: HashMap::new(),
        }
    }
    fn set_prefix_and_reset_counters (&mut self, prefix: String) {
        self.prefix = prefix;
        self.counters.clear();
    } 

    fn create_row_with_incrementing_key(&mut self, table: &str) -> &mut Row {
        let counter = self.counters.entry(table.as_string()).and_modify(|c| {*c+=1}).or_insert(1);
        let key =  format!("{}-{}", self.prefix, counter);

        self.tables.create_row(table, key)
    }

    fn log_error(&mut self, error: &str) {
        self.create_row_with_incrementing_key("Error").set("description", error);
    }

    fn to_entity_changes(self) -> EntityChanges {
        self.tables.to_entity_changes()
    }
}

#[derive(BorshDeserialize, Debug)]
struct Pubkey([u8; 32]);
impl AsRef<[u8]> for Pubkey {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}
impl Pubkey {
    fn to_string(&self) -> String {
        bs58::encode(self.0).into_string()
    }
}
#[derive(BorshDeserialize, Debug)]
struct Deposit {
    pub user: Pubkey,
    pub amount: u64,
    pub total_amount: u64,
    pub lock_expires: u32,
    pub referrer: Pubkey,
}
