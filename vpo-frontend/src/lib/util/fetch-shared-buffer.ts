export async function fetchSharedBuffer(input: RequestInfo | URL, init?: RequestInit | undefined) {
    const response = await fetch(input, init);

    if (!response.body) return;

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