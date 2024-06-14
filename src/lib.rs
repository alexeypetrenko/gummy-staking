mod pb;

use std::fmt::format;

use anyhow::bail;
use pb::sf::solana::event::v1::Event;
use pb::sf::solana::event::v1::Events;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables as DatabaseChangeTables;
// use substreams_solana::pb::sol;
use anyhow::Result;
use borsh::BorshDeserialize;
use substreams_solana::pb::sf::solana::r#type::v1::Block;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<Events, substreams::errors::Error> {
    let mut events: Vec<Event> = Vec::new();

    let program_id = "6aw4sBovP6yaG1q4y2GpjaQcLZJbBWMJP4aJFsLKxgb3";
    let PREFIX = "Program data: ";
    let start_log_message = format!("Program {program_id} invoke");
    let end_log_message = format!("Program {program_id} success");

    // block.transactions.iter().for_each(|tx|
    for tx in block.transactions.iter() {
        if let Some(meta) = tx.meta.as_ref() {
            let mut in_program = false;

            for log_message in meta.log_messages.iter() {
                if log_message.starts_with(start_log_message.as_str()) {
                    in_program = true;
                } else if log_message.starts_with(end_log_message.as_str()) {
                    in_program = false;
                } else if in_program && log_message.starts_with(PREFIX) {
                    if let Some(message) = log_message.strip_prefix(PREFIX) {
                        let decoded = log_to_event(message);
                        match decoded {
                            Ok(event) => {
                                events.push(Event { text: event });
                            }
                            Err(e) => {
                                let tx_sig = match &tx.transaction {
                                    Some(transaction) => bs58::encode(&transaction.signatures[0]).into_string(),
                                    None => "".to_string(),
                                };
                                let e_with_context =
                                    e.context(format!("Error decoding {message}. Transaction signature: {tx_sig}"));
                                return Err(e_with_context);
                            }
                        }
                    }
                }
            }
        };
    }
    // );

    Ok(Events { events })
}

fn log_to_event(message: &str) -> Result<String> {
    use base64::prelude::*;
    let base64_decoded_message = match BASE64_STANDARD.decode(message) {
        Ok(result) => result,
        Err(error) => {
            return Err(anyhow::Error::from(error).context("Error decoding base64"));
        }
    };
    if base64_decoded_message.len() < 8 {
        // return Err(anyhow!("Decoded message too short"));
        bail!("Decoded message too short");
    }
    let discriminator = &base64_decoded_message[0..8];
    let serialized_event = &base64_decoded_message[8..];

    match discriminator {
        b"\x3e\xcd\xf2\xaf\xf4\xa9\x88\x34\x00" => {
            let event = borsh::from_slice::<Deposit>(serialized_event).unwrap();
            return Ok(format!("{event:?}"));
        }
        _ => {
            bail!("Discriminator does not match known events");
        }
    }
}

#[derive(BorshDeserialize, Debug)]
struct Pubkey([u8; 32]);
#[derive(BorshDeserialize, Debug)]
struct Deposit {
    pub user: Pubkey,
    pub amount: u64,
    pub total_amount: u64,
    pub lock_expires: u32,
    pub referrer: Pubkey,
}
