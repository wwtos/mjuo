# Node lifecycle

### On creation

1. `new_variant` called with node name as string (calls the node's `new` or `default` generally)
2. `Node::init` called on newly created node
3. If `Node::init` returned a `NodeRow` with `InnerGraph`:
   1. `Node::get_child_graph_socket_list` called to get what sockets the inner graph should expose
   2. `Node::init_graph` called with the new graph

### On tick

1. `Node::accept_stream_input`, `Node::accept_midi_input`, and `Node::accept_value_input` are called, based on what input sockets were registered
2. `Node::process` is called
3. `Node::get_stream_output`, `Node::get_midi_output`, and `Node::get_value_output` are called an arbitrary number of times, depending on the traverser's implementation

### On property change

1. `Node::init` is called. Engine will update properties if `Node::init` set the `did_rows_change` flag

### On child graph change

1. `Node::init_graph` is called with new graph
