import Net from 'net';
import {createEnumDefinition} from "../util/enum";

var RawMessage = createEnumDefinition({
    "Ping": null,
    "Pong": null,
    "Data": ["array"],
    "Json": ["object"]
});

const port = 26642;
const host = '127.0.0.1';

const client = new Net.Socket();

const PING = 0x00;
const PONG = 0x01;
const DATA_BINARY = 0x02;
const DATA_JSON = 0x03;


function buildMessage(protocol, message) {
    let len = message.length;

    let len_binary = new Uint8Array(4);
    len_binary[0] = (len >> 24) & 255;
    len_binary[1] = (len >> 16) & 255;
    len_binary[2] = (len >> 8) & 255;
    len_binary[3] = (len) & 255;

    let final_message = new Uint8Array(5 + len);
    final_message.set([protocol]);
    final_message.set(len_binary, 1);
    final_message.set(message, 5);

    return final_message;
}

client.on("error", () => {
    console.error("unable to connect to server");
});

function open () {
    client.connect({ port: port, host: host }, function() {
        // If there is no error, the server has accepted the request and created a new 
        // socket dedicated to us.
        console.log('TCP connection established with the server.');

        // The client can now send data to the server by writing to its socket.
        let text = new TextEncoder().encode(JSON.stringify({
            "foo": "bar",
            "baz": {
                "la": [1, false, "apple"]
            }
        }));
        
        client.write(buildMessage(DATA_JSON, text));
    });
}


let clientState = {
    data: [],
    messages: [],
    listeners: {},
    dataToRead: 0,
    readingData: false,
    dataReadingType: 0
};

clientState.on = function(event, listener) {
    if (!this.listeners[event]) {
        this.listeners[event] = [];
    }

    this.listeners[event].push(listener);
};

clientState.trigger = function(event, value) {
    for (var listener of this.listeners[event]) {
        listener(value);
    }
};

let textDecoder = new TextDecoder();

// The client can also receive data from the server by reading from its socket.
client.on('data', function(chunk) {
    var pointer = 0;

    while (pointer < chunk.length) {
        if (clientState.readingData) {
            while (clientState.data.length < clientState.dataToRead && pointer < chunk.length) {
                clientState.data.push(chunk[pointer]);

                pointer++;
            }

            if (clientState.data.length === clientState.dataToRead) {
                switch (clientState.dataReadingType) {
                    case DATA_BINARY:
                        clientState.trigger("message", RawMessage.Data([data]));
                    break;
                    case DATA_JSON:
                        clientState.trigger("message", RawMessage.Json([JSON.parse(textDecoder.decode(Uint8Array.from(clientState.data)))]));
                    break;
                }

                clientState.data = [];
                clientState.readingData = false;
            }

            continue;
        }

        const messageType = chunk[pointer];
        pointer++;
        let dataLength;

        switch (messageType) {
            case PING:
                clientState.trigger("message", RawMessage.Ping);
                pointer++;
                continue;
            case PONG:
                clientState.trigger("message", RawMessage.Pong);
                pointer++;
                continue;
            case DATA_BINARY:
                dataLength = (chunk[pointer] << 24) + (chunk[pointer + 1] << 16) + (chunk[pointer + 2] << 8) + chunk[pointer + 3];
                clientState.dataToRead = dataLength;
                clientState.readingData = true;
                clientState.dataReadingType = DATA_BINARY;

                pointer += 4;

                continue;
            case DATA_JSON:
                dataLength = (chunk[pointer] << 24) + (chunk[pointer + 1] << 16) + (chunk[pointer + 2] << 8) + chunk[pointer + 3];

                clientState.dataToRead = dataLength;
                clientState.readingData = true;
                clientState.dataReadingType = DATA_JSON;

                pointer += 4;
                continue;
            default:
                throw "unreachable!";
        }
    }
});

client.on('end', function() {
    console.log('Requested an end to the TCP connection');
});

export default {
    on: clientState.on.bind(clientState),
    open
}
