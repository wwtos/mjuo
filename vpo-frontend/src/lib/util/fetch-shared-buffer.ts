async function parseAudio(context: AudioContext, response: Response) {
    const audioData = await context.decodeAudioData(await response.arrayBuffer());
    const bufferOut = new SharedArrayBuffer(audioData.length * 4);
    const tmpBuffer = new Float32Array(audioData.length);

    audioData.copyFromChannel(tmpBuffer, 0);

    (new Float32Array(bufferOut)).set(tmpBuffer);

    return bufferOut;
}

export async function fetchSharedBuffer(context: AudioContext, input: string, init?: RequestInit | undefined) {
    const response = await fetch(input, init);

    const resource = input.split("/").pop() ?? "";
    const extension = resource.substring(resource.indexOf(".") + 1);

    if (!response.body) return;

    if (extension === "wav" || extension === "ogg") {
        return await parseAudio(context, response);
    }

    const parts = [];
    let totalLength = 0;

    const reader = response.body.getReader();

    while (true) {
        const {done, value: chunk} = await reader.read();

        if (done) break;

        parts.push(chunk);
        totalLength += chunk.byteLength;
    }

    const sab = new SharedArrayBuffer(totalLength);
    const u8 = new Uint8Array(sab);

    let offset = 0;
    for (const buffer of parts) {
        u8.set(buffer, offset);
        offset += buffer.byteLength;
    }

    return sab;
}