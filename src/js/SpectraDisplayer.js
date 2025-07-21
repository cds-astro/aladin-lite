// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU General Public License version 3.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU General Public License as published by
//    the Free Software Foundation, version 3 of the License.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//    GNU General Public License for more details.
//
//    The GNU General Public License is available in COPYING file
//    along with Aladin Lite.
//


/******************************************************************************
 * Aladin Lite project
 *
 * File SpectraDisplayer.js
 *
 *
 * Author: Matthieu Baumann [CDS, matthieu.baumann@astro.unistra.fr]
 *
 *****************************************************************************/

export class SpectraDisplayer {
    constructor(hips, options) {
        let createPlotCanvas = (name) => {
            const canvas = document.createElement("canvas");
            canvas.classList.add(name);
            canvas.width = this.width;
            canvas.height = this.height;
            //canvas.style.pointerEvents = "none";
            canvas.style.position = "absolute";
            canvas.style.left = "50%";
            canvas.style.transform = "translateX(-50%)";
            canvas.style.bottom = "2px";

            this.view.aladinDiv.appendChild(canvas); 
            return canvas;
        };

        this.view = hips.view;

        this.data = undefined;
        this.scaleX = undefined;
        this.scaleY = undefined;
        this.height = options && options.height || 300;
        this.width = options && options.width || 600;
        this.minY = {};
        this.maxY = {};
        this.mouseFreq = undefined;

        // One canvas for the spectra
        this.canvas = createPlotCanvas("spectra-line");
        this.ctx = this.canvas.getContext("2d");
        // One canvas for the mouse hover spectral line
        const canvasCursor = createPlotCanvas('spectra-cursor')
        canvasCursor.style.pointerEvents = "none"
        this.ctxCursor = canvasCursor.getContext("2d");
        // One canvas for text
        const canvasLabels = createPlotCanvas('spectra-labels')
        canvasLabels.style.pointerEvents = "none"
        this.ctxLabels = canvasLabels.getContext("2d");

        let divNode = document.createElement("div");
        divNode.classList.add("aladin-lite-spectra-displayer")
        divNode.appendChild(this.canvas)
        divNode.appendChild(canvasCursor)
        divNode.appendChild(canvasLabels)

        this.divNode = divNode;

        this.view.aladin.aladinDiv.appendChild(divNode);

        this.defineEventListeners()
    }

    defineEventListeners() {
        let lastMouse = { x: 0, y: 0 };
        let isDragging = false;

        let canvas = this.canvas;
        let ctxCursor = this.ctxCursor;

        let self = this;
        canvas.addEventListener('mousedown', (e) => {
            const rect = canvas.getBoundingClientRect();
            const mx = e.clientX - rect.left;
            const my = e.clientY - rect.top;
            
            let v = this.data.values[Math.round(mx / this.scaleX)]

            let len = this.data.values.length;
            let fOrder = this.data.fOrder;

            v = this.height - (v - this.minY[fOrder]) * this.scaleY
            if (my >= v) {
                isDragging = true;
                lastMouse = { x: mx, y: my };
                canvas.style.cursor = 'grabbing';
            } else {
                // check if the click is next to the center bar
                // Draw the vertical line that can be grabed to move the slice
                this.ctx.beginPath();
                this.ctx.moveTo(this.scaleX * len / 2, this.height);
                this.ctx.lineTo(this.scaleX * len / 2, this.height - (this.maxY[fOrder] - this.minY[fOrder]) * this.scaleY);
                this.ctx.strokeStyle = "red";
                this.ctx.lineWidth = 10;

                if (this.ctx.isPointInStroke(mx, my)) {
                    isDragging = true;
                    lastMouse = { x: mx, y: my };
                    canvas.style.cursor = 'grabbing';
                } else {
                    // propagate event to its sibling
                    let paramsEvent = {
                        bubbles: e.bubbles,
                        cancelable: e.cancelable,
                        clientX: e.clientX,
                        clientY: e.clientY,
                        screenX: e.screenX,
                        screenY: e.screenY,
                        ctrlKey: e.ctrlKey,
                        shiftKey: e.shiftKey,
                        altKey: e.altKey,
                        metaKey: e.metaKey,
                        button: e.button,
                        relatedTarget: e.relatedTarget,
                    };
                    const event = new MouseEvent('mousedown', paramsEvent);
                    this.view.catalogCanvas.dispatchEvent(event);
                }
            }
        });
            
        canvas.addEventListener('mousemove', (e) => {
            const rect = canvas.getBoundingClientRect();
            const mx = e.clientX - rect.left;
            const my = e.clientY - rect.top;

            // can be in the spectral area
            let v = this.data.values[Math.round(mx / this.scaleX)]
            let len = this.data.values.length;

            let fOrder = this.data.fOrder;

            v = this.height - (v - this.minY[fOrder]) * this.scaleY
            canvas.style.cursor = 'default';

            this.ctxCursor.clearRect(0, 0, this.width, this.height);
            this.mouseFreq = null;

            if (my >= v) {
                canvas.style.cursor = 'grab';

                ctxCursor.beginPath();
                ctxCursor.moveTo(mx, this.height);
                ctxCursor.lineTo(mx, v);
                ctxCursor.strokeStyle = "yellow";
                ctxCursor.lineWidth = 2;
                ctxCursor.stroke()

                // compute the frequency at that position
                let curFreq = self.hips.getFrequency();
                let curHash = Number(self.view.wasm.freq2hash(self.hips.layer, curFreq));

                let mouseHash = curHash + Math.round((mx - (this.width / 2)) / this.scaleX)
                this.mouseFreq = self.view.wasm.hash2freq(self.hips.layer, BigInt(mouseHash));
            }

            this._redrawLabels()

            if (!isDragging) {
                // Draw the vertical line that can be grabed to move the slice
                this.ctx.beginPath();
                this.ctx.moveTo(this.scaleX * len / 2, this.height);
                this.ctx.lineTo(this.scaleX * len / 2, this.height - (this.maxY[fOrder] - this.minY[fOrder]) * this.scaleY);
                this.ctx.strokeStyle = "red";
                this.ctx.lineWidth = 10;

                if (this.ctx.isPointInStroke(mx, my)) {
                    console.log('Mouse is on stroke!');
                    this.canvas.style.cursor = 'grab';
                }

                return;
            }

            this.mouseFreq = null;
            this.canvas.style.cursor = 'grabbing';

            // is dragged
            const dx = (mx - lastMouse.x) / this.scaleX;
            if (dx != 0) {
                // set the frequency
                let curFreq = self.hips.getFrequency();

                let curHash = Number(self.view.wasm.freq2hash(self.hips.layer, curFreq));
                let nextHash = curHash - Math.round(dx)

                let nextFreq = self.view.wasm.hash2freq(self.hips.layer, BigInt(nextHash));
                self.hips.setFrequency({
                    value: nextFreq,
                    unit: 'Hz'
                })

                const correctedMx = Math.round(dx) * this.scaleX + lastMouse.x;
                lastMouse = { x: correctedMx, y: my };
            }
        });
            
        canvas.addEventListener('mouseup', (e) => {
            isDragging = false;
            canvas.style.cursor = 'default';

            const clickEvent = new MouseEvent('click', {
                bubbles: true,
                cancelable: true,
                clientX: e.clientX,
                clientY: e.clientY
            });
            this.view.catalogCanvas.dispatchEvent(clickEvent);
        });

        canvas.addEventListener('mouseout', (e) => {
            isDragging = false;
        });

        canvas.addEventListener('wheel', (e) => {
            this.ctxCursor.clearRect(0, 0, this.width, this.height);

            const wheelEvent = new WheelEvent('wheel', {
                bubbles: true,
                cancelable: true,
                deltaX: e.deltaX,
                deltaY: e.deltaY,
                deltaMode: e.deltaMode,
                clientX: e.clientX,
                clientY: e.clientY,
                ctrlKey: e.ctrlKey,
                shiftKey: e.shiftKey,
                altKey: e.altKey,
                metaKey: e.metaKey
            });

            this.view.catalogCanvas.dispatchEvent(wheelEvent);
        });
    }

    attachHiPS3D(hips) {
        // remove the callback from the last hips if there is
        if (this.spectraUpdateCallback) {
            window.removeEventListener("spectra", this.spectraUpdateCallback)
        }

        // store new references to the new hips
        this.hips = hips;

        this.spectraUpdateCallback = (event) => {
            this.data = event.detail;
            this._redraw(this.ctx);
        };

        window.addEventListener("spectra", this.spectraUpdateCallback);
    }

    enableInteraction() {
        this.divNode.style.pointerEvents = "auto"
    }

    disableInteraction() {
        this.divNode.style.pointerEvents = "none"
    }

    _redraw() {
        const values = this.data.values;
        let len = values.length;

        // Clear previous drawing
        this.ctx.clearRect(0, 0, this.width, this.height);

        // Find min and max for scaling
        let valuesWithNoNans = values.filter(v=>Number.isFinite(v));

        const fOrder = this.data.fOrder;

        if (Number.isFinite(this.minY[fOrder])) {
            this.minY[fOrder] = Math.min(...valuesWithNoNans, this.minY[fOrder])
        } else {
            this.minY[fOrder] = Math.min(...valuesWithNoNans)
        }
        if (Number.isFinite(this.maxY[fOrder])) {
            this.maxY[fOrder] = Math.max(...valuesWithNoNans, this.maxY[fOrder])
        } else {
            this.maxY[fOrder] = Math.max(...valuesWithNoNans)
        }

        this.scaleX = this.width / (len - 1);
        this.scaleY = (this.maxY[fOrder] - this.minY[fOrder] === 0) ? 1 : this.height / (this.maxY[fOrder] - this.minY[fOrder]);

        this._redrawSpectra(values)

        // Draw the vertical line that can be grabed to move the slice
        this.ctx.beginPath();
        this.ctx.moveTo(this.scaleX * len / 2, this.height);
        this.ctx.lineTo(this.scaleX * len / 2, this.height - (this.maxY[fOrder] - this.minY[fOrder]) * this.scaleY);
        this.ctx.strokeStyle = "red";
        this.ctx.lineWidth = 2;
        this.ctx.stroke();

        this._redrawLabels()
    }

    _redrawLabels() {
        function freq2String(frequencyHz, precisionHz) {
            const units = [
                { unit: "THz", factor: 1e12 },
                { unit: "GHz", factor: 1e9 },
                { unit: "MHz", factor: 1e6 },
                { unit: "kHz", factor: 1e3 },
                { unit: "Hz",  factor: 1 }
            ];
            
            for (const { unit, factor } of units) {
                const value = frequencyHz / factor;
                const precisionInUnit = precisionHz / factor;
            
                if (value >= 1 || unit === "Hz") {
                    // Calculate number of decimal places needed to show the given precision
                    const decimals = Math.max(0, Math.ceil(-Math.log10(precisionInUnit)));
                    return value.toFixed(decimals) + " " + unit;
                }
            }
        }

        // Clear previous drawing
        this.ctxLabels.clearRect(0, 0, this.width, this.height);

        // Draw the min and max frequencies
        this.ctxLabels.font = "20px monospace"; // You can also use "Courier New", "Consolas", etc.
        this.ctxLabels.fillStyle = "lightgreen";
        this.ctxLabels.textBaseline = "middle"; // Vertically centered

        // min window freq
        this.ctxLabels.textAlign = "left"; // Horizontally centered
        this.ctxLabels.fillText(freq2String(this.data.freqMin, this.data.freqStep), 0, this.height - 20);

        // max window freq
        this.ctxLabels.textAlign = "right"; // Horizontally centered
        this.ctxLabels.fillText(freq2String(this.data.freqMax, this.data.freqStep), this.width, this.height - 20);

        // current window freq
        this.ctxLabels.textAlign = "center"; // Horizontally centered
        let fStr; 
        if (this.mouseFreq) {
            this.ctxLabels.fillStyle = "yellow";
            fStr = freq2String(this.mouseFreq, this.data.freqStep);
        } else {
            fStr = freq2String(this.data.freq, this.data.freqStep);
        }
        this.ctxLabels.fillText(fStr, this.width / 2, this.height - 20);
    }

    _redrawSpectra(array) {
        this.ctx.beginPath();
        this.ctx.lineWidth = 4;

        let strokeStyle = "red";
        this.ctx.strokeStyle = strokeStyle

        let fOrder = this.data.fOrder;

        let prevY;
        let i = 0;
        let i1 = array.length;
        while (i <= i1) {
            let y;
            const x = i * this.scaleX;

            const inValidDomain = this.data.freqIdxStart !== undefined && this.data.freqIdxEnd !== undefined && i > this.data.freqIdxStart && i < this.data.freqIdxEnd;

            if (inValidDomain) {
                const tileNotReceived = !Number.isFinite(array[i]);
                if (tileNotReceived) {
                    // color orange
                    if (strokeStyle !== "orange") {
                        this.ctx.lineTo(x, this.height)
                        strokeStyle = "orange"
                        this.ctx.stroke()

                        this.ctx.beginPath();
                        this.ctx.strokeStyle = strokeStyle
                        this.ctx.lineWidth = 4
                    }

                    y = this.height;
                    if (i === 0) {
                        this.ctx.moveTo(x, y);
                    } else {
                        this.ctx.lineTo(x, y);
                    }
                } else {
                    // valid frequency, color green
                    if (strokeStyle !== "lightgreen") {
                        strokeStyle = "lightgreen"
                        this.ctx.stroke()

                        this.ctx.beginPath();
                        this.ctx.strokeStyle = strokeStyle
                        this.ctx.lineWidth = 2
                        this.ctx.moveTo(x - this.scaleX, prevY)
                    }

                    y = this.height - (array[i] - this.minY[fOrder]) * this.scaleY;
                    if (i === 0) {
                        this.ctx.moveTo(x, y);
                    } else {
                        this.ctx.lineTo(x, y);
                    }
                }
            } else {
                // frequency out of the survey coverage => color red
                if (strokeStyle !== "red") {
                    this.ctx.lineTo(x, this.height)
                    this.ctx.stroke()

                    this.ctx.beginPath();
                    strokeStyle = "red"
                    this.ctx.strokeStyle = strokeStyle
                    this.ctx.lineWidth = 4
                }

                y = this.height;
                if (i === 0) {
                    this.ctx.moveTo(x, y);
                } else {
                    this.ctx.lineTo(x, y);
                }
            }

            i++;
            prevY = y;
        }

        this.ctx.stroke();
    }
}
 