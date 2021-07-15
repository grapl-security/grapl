. 
| with_entries(
    select(
        # Select keys that match this regex.
        # (note: match also supports N things to match on, makes adding to this subset easy)
        .key|match("grapl-nomad-consul-*")
    )
)
