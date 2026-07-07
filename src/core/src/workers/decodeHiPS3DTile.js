self.onmessage = async (e) => {
    const { blob, tileDepth, tileSize, hips, cell } = e.data;
    const bitmap = await self.createImageBitmap(blob);

    // Compute tiling layout
    const numCols = Math.floor(bitmap.width / tileSize);

    // See HiPS3D doc
    const numRows = Math.ceil(tileDepth / numCols);

    // Create OffscreenCanvas
    const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
    const context = canvas.getContext("2d");

    // Draw full image once
    context.drawImage(bitmap, 0, 0);

    // Extract full region needed
    const imageData = context.getImageData(
        0,
        0,
        numCols * tileSize,
        numRows * tileSize
    );

    const bytes = imageData.data; // Uint8ClampedArray (RGBA)

    // Allocate output buffer (2 bytes per pixel)
    const decodedBytes = new Uint8Array(
        tileSize * tileSize * tileDepth * 2
    );

    let k = 0;
    let numTilesCropped = 0;

    for (let y = 0; y < numRows; y++) {
        const sy = y * tileSize;

        for (let x = 0; x < numCols; x++) {
            const sx = x * tileSize;

            for (let i = sy; i < sy + tileSize; i++) {
                for (let j = sx; j < sx + tileSize; j++) {

                    const idByte = (j + i * numCols * tileSize) * 4;

                    // Copy R channel
                    decodedBytes[k] = bytes[idByte];

                    // Copy A channel
                    decodedBytes[k + 1] = bytes[idByte + 3];

                    k += 2;
                }
            }

            numTilesCropped++;

            if (numTilesCropped === tileDepth) {
                break;
            }
        }

        if (numTilesCropped === tileDepth) {
            break;
        }
    }

    self.postMessage(
        {
            tileSize,
            tileDepth,
            HiPSCDid: hips,
            cell,
            bytes: decodedBytes,
        },
        [decodedBytes.buffer] // transfer of ownership
    );
};