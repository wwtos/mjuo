// @ts-ignore
import Net from 'net';
import {createEnumDefinition, EnumInstance} from "../util/enum";

var RawMessage = createEnumDefinition({
    "Ping": null,
    "Pong": null,
    "Data": ["array"],
    "Json": ["object"]
});

const port = 26642;
const host = '127.0.0.1';

const client = new Net.Socket();

enum MessageType {
    PING = 0x00,
    PONG = 0x01,
    DATA_BINARY = 0x02,
    DATA_JSON = 0x03
}

function buildMessage(protocol: MessageType, message: Uint8Array) {
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

class Client {
    data: number[];
    messages: EnumInstance[];
    listeners: {
        [key: string]: Function[]
    };
    dataToRead: number;
    readingData: boolean;
    dataReadingType: MessageType;
    client: Net.Socket;

    constructor (client: Net.Socket) {
        this.data = [];
        this.messages = [];
        this.listeners = {};
        this.dataToRead = 0;
        this.readingData = false;
        this.dataReadingType = MessageType.PING;
        this.client = client;

        let textDecoder = new TextDecoder();

        client.on('data', (chunk) => {
            var pointer = 0;
        
            while (pointer < chunk.length) {
                if (this.readingData) {
                    while (this.data.length < this.dataToRead && pointer < chunk.length) {
                        this.data.push(chunk[pointer]);
        
                        pointer++;
                    }
        
                    if (this.data.length === this.dataToRead) {
                        switch (this.dataReadingType) {
                            case MessageType.DATA_BINARY:
                                this.messages.push(RawMessage.Data([this.data]));
                                this.trigger("message", RawMessage.Data([this.data]));
                            break;
                            case MessageType.DATA_JSON:
                                this.messages.push(RawMessage.Json([JSON.parse(textDecoder.decode(Uint8Array.from(this.data)))]));
                                this.trigger("message", RawMessage.Json([JSON.parse(textDecoder.decode(Uint8Array.from(this.data)))]));
                            break;
                        }
        
                        this.data = [];
                        this.readingData = false;
                    }
        
                    continue;
                }
        
                const messageType = chunk[pointer];
                pointer++;
                let dataLength;
        
                switch (messageType) {
                    case MessageType.PING:
                        this.trigger("message", RawMessage.Ping);
                        pointer++;
                        continue;
                    case MessageType.PONG:
                        this.trigger("message", RawMessage.Pong);
                        pointer++;
                        continue;
                    case MessageType.DATA_BINARY:
                        dataLength = (chunk[pointer] << 24) + (chunk[pointer + 1] << 16) + (chunk[pointer + 2] << 8) + chunk[pointer + 3];
                        this.dataToRead = dataLength;
                        this.readingData = true;
                        this.dataReadingType = MessageType.DATA_BINARY;
        
                        pointer += 4;
        
                        continue;
                    case MessageType.DATA_JSON:
                        dataLength = (chunk[pointer] << 24) + (chunk[pointer + 1] << 16) + (chunk[pointer + 2] << 8) + chunk[pointer + 3];
        
                        this.dataToRead = dataLength;
                        this.readingData = true;
                        this.dataReadingType = MessageType.DATA_JSON;
        
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
    }

    sendJson (json: object) {
        client.write(buildMessage(MessageType.DATA_JSON, new TextEncoder().encode(JSON.stringify(json))));
    }

    on (event: string, listener: Function) {
        if ((this.listeners[event] as any) === undefined) {
            this.listeners[event] = [];
        }
    
        this.listeners[event].push(listener);
    }

    trigger (event: string, value: any) {
        for (var listener of this.listeners[event]) {
            listener(value);
        }
    }
}

export function open () {
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
        
        client.write(buildMessage(MessageType.DATA_JSON, text));
    });

    return new Client(client);
}

export { RawMessage };

// The client can also receive data from the server by reading from its socket.
