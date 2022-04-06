# Mason-Jones Unit Orchestra
This is a node-based synthesizer/sequencer, designed specifically for developing pipe organs.

## To run
_note, I have only tested this with linux. If you want to get it running on another OS, you'll need to implement a MIDI and audio backend in `vpo-backend/sound-engine/src/backend`_

There's two parts to this code: The frontend and the backend.

### Backend
To get the backend running, just navigate to it and use `cargo run`

### Frontend
To run the frontend, first install all the packages with `npm install`.
Next, run `npm run watch-main` to watch the electron main process code.
Finally, run `npm run start` to start the frontend.
