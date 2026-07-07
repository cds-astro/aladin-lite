self.onmessage = async (e) => {
    const { blob, tileSize, hips, cell } = e.data;

    // createImageBitmap is async, you need to await it
    const bitmap = await self.createImageBitmap(blob);

    // Transfer the bitmap directly — no need to extract bytes,
    // ImageBitmap can be transferred zero-copy and uploaded
    // directly via texImage2D(bitmap) on the main thread
    self.postMessage(
        {
            tileSize,
            hips,
            cell,
            bitmap,
        },
        [bitmap] // transfer ownership, zero-copy
    );
};