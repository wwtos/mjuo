# node-engine

# Node lifecycle

See `NODE_LIFECYCLD.md`.

# Design notes

One thing I wanted to explore the possibility of was per-sample traversal. There are no buffers, just a method accepting in a single value. At most, there will be a delay of one sample. I'm not sure how long I can continue in this way, but it isn't really far enough along to accurately judge whether I'll need to change to buffered audio later on. Due to this, I don't know if I can do much with multi-threading. I do plan on having a few nodes running on different threads -- namely the VST host node, and the sample playback node. They'll periodically stream the next buffer to the node. Due to this I need _fast_ nodes. I've opted to enable static-analysis as much as possible. Enums are statically dispatched, socket types are keyed, and most of them are statically-named enums (with the option for dynamic sockets).

Another thing I wanted to do with this is allow for an arbitrary (and dynamic) number of I/Os. Whenever a node is connected, edited, or modified in some way, it is reinitialized. At that point it can return a new list of I/Os, and the node engine will adjust everything accordingly.

I opted for a multi output, single input model. One node's output can be connected to multiple other nodes, but it can only have a single input. It removes a lot of ambiguity over how a node will process multiple inputs. It would need to be made explicit with something like a mixer node for mixing audio, or a latest node, for passing through the latest value.

### Memory/graph layout

I very intentionally designed the graph with memory layout in mind. It uses a generational-based graph layout in a vec. I've tried to avoid using pointers whenever possible in node code to preserve the linearity of memory. Heck, I use an enum inside of the graph, not a pointer to a node. Nodes are statically dispatched, allowing the compiler to use even more optimizations. I need to migrate code over still, but I want to incorperate `SmallVec` into more parts of the code, again, to preserve memory linearity.

### Scripting

In my mind, scripting is a _must_. Scripting feels like the #1 missing feature of audio engines. At some point it's just too combersome to use a bunch of tiny nodes or hacks. Even worse, a feature you want can't be implemented at all. In my mind it is _imperative_ for this codebase to be scripting-friendly. I opted to use Rhai, as it's fast enough, but more importantly it has a lot of runtime safety guarantees (very important for running external code). Scripts should be able to modulate sound as much as they
should be able to modulate values/midi.

# Inspirations

- Blender's node editor
- rete.js
