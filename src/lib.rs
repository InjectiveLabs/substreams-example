mod pb;

#[substreams::handlers::map]
fn map_transfer(blk: pb::cosmos::Block) -> Result<pb::cosmos::ResponseBeginBlock, substreams::errors::Error> {
    let events: Vec<pb::cosmos::Event> = blk.result_begin_block
        .unwrap()
        .events
        .into_iter()
        .filter(|event| event.event_type == "transfer")
        .collect();
    Ok(pb::cosmos::ResponseBeginBlock {events})
}