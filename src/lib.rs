mod events;
mod pb;
mod tables_with_incrementing_key;

use anyhow::Result;
use base64::prelude::*;
use substreams_entity_change::pb::entity::EntityChanges;
use substreams_entity_change::tables::ToValue;
use substreams_solana::pb::sf::solana::r#type::v1::Block;
use tables_with_incrementing_key::TablesWithIncrementingKey;

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
                    tables.log_error("Error decoding base64");
                    continue;
                };
                if base64_decoded_message.len() < 8 {
                    tables.log_error("Decoded message too short");
                    continue;
                }
                let discriminator = &base64_decoded_message[0..8];
                let serialized_event = &base64_decoded_message[8..];

                match discriminator {
                    events::DISCRIMINATOR_DEPOSIT => {
                        let event_name = "Deposit";
                        match borsh::from_slice::<events::Deposit>(serialized_event) {
                            Err(e) => tables.log_error(&format!("Error deserializing event '{event_name}': '{e}'. Log is {message}.")),
                            Ok(event) => {
                                tables
                                    .create_row_with_incrementing_key(&format!("{event_name}Event"))
                                    .set_if_some(
                                        "timestamp",
                                        block.block_time.as_ref().map(|x| x.timestamp),
                                    )
                                    .set("user", event.user.to_string())
                                    .set("amount", event.amount)
                                    .set("total_amount", event.total_amount)
                                    .set("lock_expires", event.lock_expires)
                                    .set("referrer", event.referrer.to_string());
                            }
                        }
                    }
                    events::DISCRIMINATOR_WITHDRAW => {
                        let event_name = "Withdraw";
                        match borsh::from_slice::<events::Withdraw>(serialized_event) {
                            Err(e) => tables.log_error(&format!("Error deserializing event '{event_name}': '{e}'. Log is {message}.")),
                            Ok(event) => {
                                tables
                                    .create_row_with_incrementing_key(&format!("{event_name}Event"))
                                    .set_if_some(
                                        "timestamp",
                                        block.block_time.as_ref().map(|x| x.timestamp),
                                    )
                                    .set("user", event.user.to_string())
                                    .set("total_amount", event.total_amount);
                            }
                        }
                    }
                    events::DISCRIMINATOR_SET_REFERRER => {
                        let event_name = "SetReferrer";
                        match borsh::from_slice::<events::SetReferrer>(serialized_event) {
                            Err(e) => tables.log_error(&format!("Error deserializing event '{event_name}': '{e}'. Log is {message}.")),
                            Ok(event) => {
                                tables
                                .tables
                                    .create_row("Referrer", event.user.to_string())
                                    .set("referrer", event.new_referrer.to_string());
                            }
                        }
                    }
                    events::DISCRIMINATOR_REGISTER_SHORT_REFERRER => {
                        let event_name = "RegisterShortReferrer";
                        match borsh::from_slice::<events::RegisterShortReferrer>(serialized_event) {
                            Err(e) => tables.log_error(&format!("Error deserializing event '{event_name}': '{e}'. Log is {message}.")),
                            Ok(event) => {
                                tables
                                .tables
                                    .create_row("ShortReferrer", String::from_utf8_lossy(event.short.as_ref()))
                                    .set("full", event.full.to_string());
                            }
                        }
                    }
                    events::DISCRIMINATOR_ADMIN_REGISTER_SHORT_REFERRER => {
                        let event_name = "AdminRegisterShortReferrer";
                        match borsh::from_slice::<events::AdminRegisterShortReferrer>(serialized_event) {
                            Err(e) => tables.log_error(&format!("Error deserializing event '{event_name}': '{e}'. Log is {message}.")),
                            Ok(event) => {
                                tables
                                .tables
                                    .create_row("ShortReferrer", String::from_utf8_lossy(event.short.as_ref()))
                                    .set("full", event.full.to_string());
                            }
                        }
                    }
                    events::DISCRIMINATOR_ADMIN_DELETE_SHORT_REFERRER => {
                        let event_name = "AdminDeleteShortReferrer";
                        match borsh::from_slice::<events::AdminDeleteShortReferrer>(serialized_event) {
                            Err(e) => tables.log_error(&format!("Error deserializing event '{event_name}': '{e}'. Log is {message}.")),
                            Ok(event) => {
                                tables
                                .tables
                                    .delete_row("ShortReferrer", String::from_utf8_lossy(event.short.as_ref()));
                            }
                        }
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
            None => self,
        }
    }
}
