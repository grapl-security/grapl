use crate::models::{get_image_name, strip_file_zone_identifier, utc_to_epoch};
use graph_descriptions::file::FileState;
use graph_descriptions::graph_description::*;
use graph_descriptions::node::NodeT;
use graph_descriptions::process::ProcessState;
use sysmon::FileCreateEvent;

/// Creates a subgrqph describing a `FileCreateEvent`
///
/// The subgraph generation for a `FileCreateEvent` includes the following:
/// * A creator `Process` node - denotes the process that created the file
/// * A subject `File` node - the file that is created as part of this event
pub fn generate_file_create_subgraph(
    file_create: &FileCreateEvent,
) -> Result<Graph, failure::Error> {
    let timestamp = utc_to_epoch(&file_create.event_data.creation_utc_time)?;
    let mut graph = Graph::new(timestamp);

    let creator = ProcessBuilder::default()
        .asset_id(file_create.system.computer.computer.clone())
        .state(ProcessState::Existing)
        .process_id(file_create.event_data.process_id)
        .process_name(get_image_name(&file_create.event_data.image.clone()).unwrap())
        .last_seen_timestamp(timestamp)
        //        .created_timestamp(file_create.event_data.process_guid.get_creation_timestamp())
        .build()
        .map_err(|err| failure::err_msg(err))?;

    let file = FileBuilder::default()
        .asset_id(file_create.system.computer.computer.clone())
        .state(FileState::Created)
        .file_path(strip_file_zone_identifier(
            &file_create.event_data.target_filename,
        ))
        .created_timestamp(timestamp)
        .build()
        .map_err(|err| failure::err_msg(err))?;

    graph.add_edge(
        "created_files",
        creator.clone_node_key(),
        file.clone_node_key(),
    );
    graph.add_node(creator);
    graph.add_node(file);

    Ok(graph)
}
