// SPDX-License-Identifier: LGPL-3.0-or-later
// Copyright 2013 - UDS/CNRS
// The Aladin Lite program is distributed under the terms
// of the GNU Lesser General Public License version 3
// or (at your option) any later version.
//
// This file is part of Aladin Lite.
//
//    Aladin Lite is free software: you can redistribute it and/or modify
//    it under the terms of the GNU Lesser General Public License as published by
//    the Free Software Foundation, either version 3 of the License, or
//    (at your option) any later version.
//
//    Aladin Lite is distributed in the hope that it will be useful,
//    but WITHOUT ANY WARRANTY; without even the implied warranty of
//    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//    GNU Lesser General Public License for more details.
//
//    You should have received a copy of the GNU Lesser General Public License
//    along with Aladin Lite. If not, see <https://www.gnu.org/licenses/>.
//

import { ActionButton } from "./gui/Widgets/ActionButton";
import { Input } from "./gui/Widgets/Input";
import HomeIconUrl from '../../assets/icons/maximize.svg';
import SpectraIconUrl from '../../assets/icons/freq.svg';
import { Utils } from "./Utils";
import { Aladin } from "./Aladin";
import { DOMElement } from "./gui/Widgets/Widget";
/******************************************************************************
 * Aladin Lite project
 *
 * File SpectraDisplayer.js
 *
 *
 * Author: Matthieu Baumann [CDS, matthieu.baumann@astro.unistra.fr]
 *
 *****************************************************************************/

export class SpectraDisplayer extends DOMElement {
    static UNIT = {
        FREQUENCY: {
            label: "f",
            units: [
                { unit: "THz", factor: 1e12 },
                { unit: "GHz", factor: 1e9 },
                { unit: "MHz", factor: 1e6 },
                { unit: "kHz", factor: 1e3 },
                { unit: "Hz",  factor: 1 }
            ]
        },
        WAVELENGTH: {
            label: "λ",
            units: [
                { unit: "km", factor: 1e3 },
                { unit: "m", factor: 1 },
                { unit: "mm", factor: 1e-3 },
                { unit: "μm", factor: 1e-6 },
                { unit: "nm", factor: 1e-9 },
                { unit: "Å", factor: 1e-10 },
                { unit: "pm", factor: 1e-12 },
            ]
        },
        VELOCITY: {
            label: "v",
            units: [
                { unit: "km/s", factor: 1e3 },
                { unit: "m/s", factor: 1 },
                { unit: "mm/s", factor: 1e-3 },
                { unit: "μm/s", factor: 1e-6 },
                { unit: "nm/s", factor: 1e-9 },
                { unit: "pm/s", factor: 1e-12 },
            ]
        },
        convertFrequency: function(freq, options) {
            const SPEED_OF_LIGHT = 299792458.0;

            let unit = options && options.unit;

            let value;
            if (unit === SpectraDisplayer.UNIT.WAVELENGTH) {
                value = SPEED_OF_LIGHT / freq;
            } else if (unit === SpectraDisplayer.UNIT.VELOCITY) {
                // A velocity is given in "m/s"
                const restFreq = options && options.restFreq;
                if (!restFreq) {
                    throw 'When giving a velocity, a rest frequency must be given as well for computing the frequency to query the HiPS'
                }

                value = SPEED_OF_LIGHT * (1.0 - freq / restFreq)
            } else {
                // unit is "Hz"
                value = freq;
            }

            return value;
        },
        convertPrecisionFrequency: function(prec, freq, options) {
            const SPEED_OF_LIGHT = 299792458.0;

            let unit = options && options.unit;

            let value;
            if (unit === SpectraDisplayer.UNIT.WAVELENGTH) {
                value = SPEED_OF_LIGHT * prec / (freq * freq);
            } else if (unit === SpectraDisplayer.UNIT.VELOCITY) {
                // A velocity is given in "m/s"
                const restFreq = options && options.restFreq;
                if (!restFreq) {
                    throw 'When giving a velocity, a rest frequency must be given as well for computing the frequency to query the HiPS'
                }

                value = prec * SPEED_OF_LIGHT / restFreq
            } else {
                // unit is "Hz"
                value = prec;
            }

            return value;
        }
    };

    constructor(view, options) {
        super()

        let createPlotCanvas = (name) => {
            const canvas = document.createElement("canvas");
            canvas.classList.add(name);
            canvas.width = this.width;
            canvas.height = this.height;
            canvas.style.position = "absolute"
            canvas.style.top = 0;
            canvas.style.left = 0;

            this.view.aladinDiv.appendChild(canvas); 
            return canvas;
        };

        this.view = view;

        this.data = undefined;
        this.scaleX = undefined;
        this.scaleY = undefined;
        this.height = options && options.height || 300;
        this.width = options && options.width || 600;
        this.minY = undefined;
        this.maxY = undefined;
        this.mouseFreq = undefined;
        this.enabled = true;

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

        let self = this;
        // a selector for choosing the unit
        let unitSelector = new Input({
            label: "Unit:",
            name: "unit selector",
            value: "f",
            type: 'select',
            classList: ['aladin-spectra-unit-selector'],
            options: [
                SpectraDisplayer.UNIT.FREQUENCY.label,
                SpectraDisplayer.UNIT.WAVELENGTH.label,
                SpectraDisplayer.UNIT.VELOCITY.label,
            ],
            tooltip: {
                content: `Unit: ${SpectraDisplayer.UNIT.FREQUENCY.label}, ${SpectraDisplayer.UNIT.WAVELENGTH.label} and ${SpectraDisplayer.UNIT.VELOCITY.label}`,
                position: {direction: "right"}
            },
            change: (e) => {
                let label = e.target.value;

                if (label === SpectraDisplayer.UNIT.FREQUENCY.label) {
                    self.unit = SpectraDisplayer.UNIT.FREQUENCY
                } else if (label === SpectraDisplayer.UNIT.WAVELENGTH.label) {
                    self.unit = SpectraDisplayer.UNIT.WAVELENGTH
                } else {
                    self.unit = SpectraDisplayer.UNIT.VELOCITY
                }

                self._redrawLabels();
            },
        })

        let autoCenterBtn = new ActionButton({
            size: 'small',
            icon: {
                monochrome: true,
                url: HomeIconUrl
            },
            tooltip: {
                content: "Scale for data",
                position: {direction: "right"}
            },
            classList: ['aladin-spectra-home'],
            action(e) {
                self.resetScale()
                self._redraw(self.ctx);
            }
        })
        let extractionBtn = new ActionButton({
            size: 'small',
            icon: {
                monochrome: true,
                url: SpectraIconUrl
            },
            tooltip: {
                content: "Extract the spectra under the cursor",
                position: {direction: "right"}
            },
            classList: ['aladin-spectra-extraction'],
            action(e) {
                // TODO
            }
        })

        this.unit = SpectraDisplayer.UNIT.FREQUENCY;

        let divNode = document.createElement("div");
        divNode.style.position = "absolute";
        divNode.style.left = "50%";
        divNode.style.transform = "translateX(-50%)";
        divNode.style.bottom = "2px";
        divNode.style.width = this.width + "px";
        divNode.style.height = this.height + "px";
        divNode.classList.add("aladin-lite-spectra-displayer")

        divNode.appendChild(this.canvas)
        divNode.appendChild(canvasCursor)
        divNode.appendChild(canvasLabels)
        divNode.appendChild(unitSelector.element())
        divNode.appendChild(autoCenterBtn.element())
        //divNode.appendChild(extractionBtn.element())

        this.divNode = divNode;

        this.view.aladin.aladinDiv.appendChild(divNode);

        this.defineEventListeners()
        this.hips3DList = new Map();
    }

    defineEventListeners() {
        this.lastMouse = { x: 0, y: 0 };
        this.isDragging = false;

        let canvas = this.canvas;
        let ctxCursor = this.ctxCursor;

        let self = this;

        let lastClickTime = 0;
        const DOUBLE_CLICK_DELAY = 300; // most operating systems uses duration between 250ms and 500ms by default.

        let mouseDownTime = 0;
        let mouseDownPos = { x: 0, y: 0 };
        const CLICK_TIME_THRESHOLD = 250;  // ms
        const CLICK_MOVE_THRESHOLD = 5;    // pixels


        Utils.on(canvas, 'mousedown touchstart', (e) => {
            mouseDownTime = Date.now();
            mouseDownPos = Utils.relMouseCoords(e);

            const mx = mouseDownPos.x;
            const my = mouseDownPos.y;
            let v = this.data.values[Math.round(mx / this.scaleX)]

            let len = this.data.values.length;

            v = this.height - (v - this.minY) * this.scaleY
            if (my >= v) {
                this.lastMouse = { x: mx, y: my };
                canvas.style.cursor = 'grabbing';
            } else {
                // check if the click is next to the center bar
                // Draw the vertical line that can be grabed to move the slice

                this.ctx.beginPath();
                this.ctx.lineWidth = 30;

                this.ctx.moveTo(this.scaleX * len / 2, this.height);
                this.ctx.lineTo(this.scaleX * len / 2, this.height - (this.maxY - this.minY) * this.scaleY);
                this.ctx.strokeStyle = Aladin.DEFAULT_OPTIONS.reticleColor;

                if (this.ctx.isPointInStroke(mx, my)) {
                    this.lastMouse = { x: mx, y: my };
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
                        changedTouches: e.changedTouches,
                        targetTouches: e.targetTouches,
                        relatedTarget: e.relatedTarget,
                    };
                    let event;
                    if (e.type === "mousedown") {
                        event = new MouseEvent("mousedown", paramsEvent);
                    } else {
                        this.disableInteraction();
                        event = new TouchEvent("touchstart", paramsEvent)
                    }
                    // Track timing to simulate dblclick
                    const now = Date.now();
                    if (now - lastClickTime < DOUBLE_CLICK_DELAY) {
                        const dblClickEvent = new MouseEvent('dblclick', {
                            bubbles: true,
                            cancelable: true,
                            clientX: e.clientX,
                            clientY: e.clientY
                        });

                        this.view.catalogCanvas.dispatchEvent(dblClickEvent);
                        lastClickTime = 0; // reset
                    } else {
                        lastClickTime = now;
                    }

                    this.view.catalogCanvas.dispatchEvent(event);
                }
            }
        });
            
        Utils.on(canvas, 'mousemove touchmove', (e) => {
            if (!this.enabled) {
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
                    changedTouches: e.changedTouches,
                    targetTouches: e.targetTouches,
                    relatedTarget: e.relatedTarget,
                };

                let touchEvent = new TouchEvent("touchmove", paramsEvent);
                this.view.catalogCanvas.dispatchEvent(touchEvent);
                return;
            }

            let mouseXY = Utils.relMouseCoords(e)
            const mx = mouseXY.x;
            const my = mouseXY.y;         

            // can be in the spectral area
            let v = this.data.values[Math.round(mx / this.scaleX)]
            let len = this.data.values.length;

            v = this.height - (v - this.minY) * this.scaleY
            if (!this.isDragging) {
                this.isDragging = canvas.style.cursor === 'grabbing';
            }

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

            if (!this.isDragging) {
                // Draw the vertical line that can be grabed to move the slice
                this.ctx.beginPath();
                this.ctx.moveTo(this.scaleX * len / 2, this.height);
                this.ctx.lineTo(this.scaleX * len / 2, this.height - (this.maxY - this.minY) * this.scaleY);
                this.ctx.strokeStyle = "red";
                this.ctx.lineWidth = 30;

                if (this.ctx.isPointInStroke(mx, my)) {
                    this.canvas.style.cursor = 'grab';
                }

                return;
            }

            
            this.mouseFreq = null;

            // is dragged
            let dx = (mx - this.lastMouse.x) / this.scaleX;
            if (dx != 0) {
                // Set the frequency

                // look where we are in the freq range
                let j = Utils.binarySearch(self.data.freqs, self.data.freq);
                let df, f;
                if (j > 0 && j < self.data.freqs.length - 1) {
                    df = (self.data.freqs[j + 1] - self.data.freqs[j - 1]) * 0.5;
                    f = self.data.freq - dx * df;
                } else if (j == 0) {
                    df = self.data.freqs[1] - self.data.freqs[0]
                    f = self.data.freqs[0] - dx * df;
                } else {
                    df = self.data.freqs[self.data.freqs.length - 1] - self.data.freqs[self.data.freqs.length - 2];
                    f = self.data.freqs[self.data.freqs.length - 1] - dx * df;
                }

                self.hips.setFrequency({
                    value: f,
                    unit: 'Hz'
                })

                this.lastMouse = { x: mx, y: my };
            }
        });

        Utils.on(canvas, 'mouseup touchend', (e) => {
            if (!this.enabled) {
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
                    changedTouches: e.changedTouches,
                    targetTouches: e.targetTouches,
                    relatedTarget: e.relatedTarget,
                };

                let touchEvent = new TouchEvent("touchend", paramsEvent);
                this.view.catalogCanvas.dispatchEvent(touchEvent);
                return;
            }

            this.isDragging = false;
            canvas.style.cursor = 'default';

            let mouseXY = Utils.relMouseCoords(e);

            const timeDiff = Date.now() - mouseDownTime;
            const dx = mouseXY.x - mouseDownPos.x;
            const dy = mouseXY.y - mouseDownPos.y;

            const dist = Math.sqrt(dx * dx + dy * dy);

            if (timeDiff < CLICK_TIME_THRESHOLD && dist < CLICK_MOVE_THRESHOLD) {
                // Custom click detected
                const rect = canvas.getBoundingClientRect();
                const mx = mouseXY.x;
                const my = mouseXY.y;
                let v = this.data.values[Math.round(mx / this.scaleX)]
                v = this.height - (v - this.minY) * this.scaleY

                if (my >= v) {
                    let dx = (mx - rect.width * 0.5) / this.scaleX;
                    if (dx != 0) {
                        // Set the frequency

                        // look where we are in the freq range
                        let j = Utils.binarySearch(self.data.freqs, self.data.freq);
                        let df, f;
                        if (j > 0 && j < self.data.freqs.length - 1) {
                            df = (self.data.freqs[j + 1] - self.data.freqs[j - 1]) * 0.5;
                            f = self.data.freq + dx * df;
                        } else if (j == 0) {
                            df = self.data.freqs[1] - self.data.freqs[0]
                            f = self.data.freqs[0] + dx * df;
                        } else {
                            df = self.data.freqs[self.data.freqs.length - 1] - self.data.freqs[self.data.freqs.length - 2];
                            f = self.data.freqs[self.data.freqs.length - 1] + dx * df;
                        }

                        self.hips.setFrequency({
                            value: f,
                            unit: 'Hz'
                        })
                    }

                    this.lastMouse = { x: mx, y: my };
                }

                //this.ctxCursor.clearRect(0, 0, this.width, this.height);
                this.mouseFreq = null;
            }


            if (e.type !== "touchend") {
                const clickEvent = new MouseEvent('click', {
                    bubbles: true,
                    cancelable: true,
                    clientX: e.clientX,
                    clientY: e.clientY
                });
                this.view.catalogCanvas.dispatchEvent(clickEvent);
            }
        });

        Utils.on(canvas, 'mouseout touchcancel', (e) => {
            this.isDragging = false;
        });

        Utils.on(canvas, 'wheel', (e) => {
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

    _hide() {
        if (this.isHidden) {
            return;
        }

        this.divNode.style.display = "none";
        this.isHidden = true;
    }

    _show() {
        if (!this.isHidden) {
            return;
        }

        this.divNode.style.display = "block";
        this.isHidden = false;
    }

    attachHiPS3D(hips) {
        // remove the callback from the last hips if there is
        if (this.spectraUpdateCallback) {
            window.removeEventListener("spectra", this.spectraUpdateCallback)
        }

        // store new references to the new hips
        this.hips = hips;

        if (hips) {
            this.spectraUpdateCallback = (event) => {
                let data = event.detail;
                if (data.layer === this.hips.layer) {
                    this.data = data;
                    console.log(data)
                    this._redraw(this.ctx);
                }
            };

            window.addEventListener("spectra", this.spectraUpdateCallback);

            this.resetScale();
            this._show()
        }
    }

    // When changing the HiPS format, a scale reset is necessary
    resetScale() {
        this.minY = undefined;
        this.maxY = undefined;
    }

    enableInteraction() {
        this.enabled = true;
        this.divNode.style.pointerEvents = "auto"
    }

    disableInteraction() {
        this.enabled = false;
        this.divNode.style.pointerEvents = "none"
    }

    _redraw() {
        const values = this.data.values;
        let len = values.length;

        // Clear previous drawing
        this.ctx.clearRect(0, 0, this.width, this.height);

        // Find min and max for scaling
        let valuesWithNoNans = values.filter(v=>Number.isFinite(v));

        if (Number.isFinite(this.minY)) {
            this.minY = Math.min(...valuesWithNoNans, this.minY)
        } else {
            this.minY = Math.min(...valuesWithNoNans)
        }
        if (Number.isFinite(this.maxY)) {
            this.maxY = Math.max(...valuesWithNoNans, this.maxY)
        } else {
            this.maxY = Math.max(...valuesWithNoNans)
        }

        this.scaleX = this.width / (len - 1);
        this.scaleY = (this.maxY - this.minY === 0) ? 1 : this.height / (this.maxY - this.minY);

        this._redrawSpectra(values)

        // Draw the vertical line that can be grabed to move the slice
        this.ctx.beginPath();
        this.ctx.moveTo(this.scaleX * len / 2, this.height);
        this.ctx.lineTo(this.scaleX * len / 2, this.height - (this.maxY - this.minY) * this.scaleY);
        this.ctx.strokeStyle = Aladin.DEFAULT_OPTIONS.reticleColor;
        this.ctx.lineWidth = 2;
        this.ctx.stroke();

        this.ctxCursor.clearRect(0, 0, this.width, this.height);
        this.ctxCursor.beginPath();
        this.ctxCursor.moveTo(this.lastMouse.x, this.height);
        let v = this.data.values[Math.round(this.lastMouse.x / this.scaleX)]
        v = this.height - (v - this.minY) * this.scaleY
        this.ctxCursor.lineTo(this.lastMouse.x, v);
        this.ctxCursor.strokeStyle = "yellow";
        this.ctxCursor.lineWidth = 2;
        this.ctxCursor.stroke()

        this._redrawLabels()
    }

    _redrawLabels() {
        let self = this;
        let spectraValue2String = (freq, precision) => {
            let units = self.unit.units;

            let x = SpectraDisplayer.UNIT.convertFrequency(
                freq,
                {
                    unit: self.unit,
                    restFreq: self.hips.obsRestFreq
                }
            )

            let dx = SpectraDisplayer.UNIT.convertPrecisionFrequency(
                precision,
                freq,
                {
                    unit: self.unit,
                    restFreq: self.hips.obsRestFreq
                }
            )

            for (const { unit, factor } of units) {
                const value = x / factor;
                const precisionInUnit = dx / factor;
            
                if (Math.abs(value) >= 1 || unit === units[units.length - 1].unit) {
                    // Calculate number of decimal places needed to show the given precision
                    const decimals = Math.max(0, Math.ceil(-Math.log10(precisionInUnit)));
                    return value.toFixed(decimals) + " " + unit;
                }
            }
        }

        // Clear previous drawing
        this.ctxLabels.clearRect(0, 0, this.width, this.height);

        let drawLabel = (ctx, str, x, y, strokeStyle, font, fillStyle) => {
            ctx.strokeStyle = strokeStyle;       // contour color
            ctx.strokeText(str, x, y);

            ctx.fillStyle = fillStyle;
            ctx.font = font
            ctx.fillText(str, x, y);
        };

        // Draw the min and max frequencies
        this.ctxLabels.font = "20px monospace"; // You can also use "Courier New", "Consolas", etc.
        this.ctxLabels.fillStyle = "lightgreen";
        this.ctxLabels.textBaseline = "middle"; // Vertically centered

        // min window freq
        this.ctxLabels.textAlign = "left"; // Horizontally centered
        drawLabel(
            this.ctxLabels,
            spectraValue2String(this.data.freqMin, this.data.freqs[1] - this.data.freqs[0]),
            0,
            this.height - 20,
            'black',
            '20px monospace',
            'lightgreen'
        )

        // max window freq
        this.ctxLabels.textAlign = "right"; // Horizontally centered
        drawLabel(
            this.ctxLabels,
            spectraValue2String(this.data.freqMax, this.data.freqs[this.data.freqs.length - 1] - this.data.freqs[this.data.freqs.length - 2]),
            this.width,
            this.height - 20,
            'black',
            '20px monospace',
            'lightgreen'
        )

        // current window freq
        this.ctxLabels.textAlign = "center"; // Horizontally centered
        let str, fillStyle; 
        if (!this.isDragging && this.mouseFreq) {
            fillStyle = "yellow";
            str = spectraValue2String(this.mouseFreq, this.data.freqStep);
        } else {
            fillStyle = Aladin.DEFAULT_OPTIONS.reticleColor;
            str = spectraValue2String(this.data.freq, this.data.freqStep);
        }
        drawLabel(
            this.ctxLabels,
            str,
            this.width / 2,
            this.height - 20,
            'black',
            '20px monospace',
            fillStyle
        )
    }

    _redrawSpectra(array) {
        this.ctx.beginPath();
        this.ctx.lineWidth = 4;

        let strokeStyle = "red";
        this.ctx.strokeStyle = strokeStyle

        let prevY;
        let i = 0;
        let i1 = array.length;

        while (i < i1) {
            let y;
            let x = i * this.scaleX;

            const inValidDomain = this.data.freqIdxStart !== undefined && this.data.freqIdxEnd !== undefined && i >= this.data.freqIdxStart && i <= this.data.freqIdxEnd;

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

                    y = this.height - (array[i] - this.minY) * this.scaleY;

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
 