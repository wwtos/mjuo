# Audio backend
The audio backend is written in Rust. This section of the project communicates with the frontend, currently through a TCP stream.

# Crates
There are a few crates, mainly:
1. `ipc`. This takes care of communication with the client.
2. `sound-engine`. This is where a lot of the complicated DSP code is stored. It is separated from `node-engine` in order to help reduce mental overhead. `sound-engine` should just contain complicated algorithms. `node-engine` takes care of the stupid realities of computers and IO management.
3. `node-engine`. The `node-engine` crate contains all of the code related to the actual audio processing, as well as the audio nodes. More details are inside the `node-engine` README.

There is also the main process in `src`. That code is (currently) just ceremonial routing, as well as connecting to the audio sources.
