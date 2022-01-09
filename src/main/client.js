const Net = require('net');

const port = 26642;
const host = '127.0.0.1';

const client = new Net.Socket();

const PING = 0x00;
const PONG = 0x01;
const DATA_BINARY = 0x02;


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

client.connect({ port: port, host: host }, function() {
    // If there is no error, the server has accepted the request and created a new 
    // socket dedicated to us.
    console.log('TCP connection established with the server.');

    // The client can now send data to the server by writing to its socket.
    let text = new TextEncoder().encode("Hello world.");
    
    client.write(buildMessage(0x01, text));
});

let clientState = {
    data: [],
    dataToRead: 0,
    readingData: false
};

let textDecoder = new TextDecoder();

// The client can also receive data from the server by reading from its socket.
client.on('data', function(chunk) {
    var pointer = 0;

    if (!clientState.readingData) {
        const messageType = chunk[0];

        switch (messageType) {
            case PING:
                client.write(Uint8Array.from([PONG]));
                return;
            case PONG:
                return;
            case DATA_BINARY:
                let dataLength = chunk[1] << 24 + chunk[2] << 16 + chunk[3] << 8 + chunk[4];

                clientState.dataToRead = dataLength;

                pointer += 5;
            break;
            default:
                throw "unreachable!";
        }
    }

    for (var i = pointer; i < chunk.length; i++) {
        clientState.data.push(chunk[i]);
    }

    if (clientState.data.length >= clientState.dataToRead) {
        clientState.readingData = false;

        console.log(`Data received from the server: ${textDecoder.decode(Uint8Array.from(clientState.data))}.`);

        clientState.data.length = 0;
    }
});

client.on('end', function() {
    console.log('Requested an end to the TCP connection');
});
