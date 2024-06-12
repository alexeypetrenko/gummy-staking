mod pb;

use pb::sf::solana::event::v1::Event;
use substreams_database_change::pb::database::DatabaseChanges;
use substreams_database_change::tables::Tables as DatabaseChangeTables;
// use substreams_solana::pb::sol;
use substreams_solana::pb::sf::solana::r#type::v1::Block;

#[substreams::handlers::map]
fn map_events(block: Block) -> Result<Event, substreams::errors::Error> {
    let mut block_height: Option<u64> = None;
    if let Some(v) = block.block_height.as_ref() {
        block_height = Some(v.block_height)
    }
    

    Ok(Event {
        text: "321323 text".to_string()
    })
}
