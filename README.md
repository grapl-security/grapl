# grapl
Graph platform for Detection, Forensics, and Incident Response


Grapl aims to describe a network, and actions taking place on that network,
as a graph. By doing so it will make querying for interconnected
events efficient and easily expressable.

As an example, one can write signatures for behaviors like: 
* Process with `image_name` "word.exe" executes
* "word" executes a child process not on our whitelisted

Further, we can automatically expand signature hits out to scope
our engagement.

Given the `word` and `payload` children we can recursively
add subsequent children, find the files read by word, etc.

![word_macro_graph](https://github.com/insanitybit/grapl/blob/master/images/word_macro_graph.png)


### Features

Grapl consists primarily of:

1. Parsers to turn logs into subgraphs, and infra to merge those subgraphs into a master graph
2. Analyzers to query the master graph in realtime
3. Engagements to interact with the output of analyzers