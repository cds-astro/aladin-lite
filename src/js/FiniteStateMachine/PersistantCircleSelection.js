// SPDX-License-Identifier: LGPL-3.0-or-later*

import { FSM } from "../FiniteStateMachine";
import { Utils } from "../Utils";
import { Circle } from "../shapes/Circle";
/**
 * Persistent interactive circle selector.
 *
 * Interactions:
 *   - Drag the center handle -> move the circle
 *   - Drag the resize handle -> resize the circle
 *
 * The circle remains visible and interactive after mouseup.
 */
export class PersistantCircleSelect extends FSM {
    constructor(options = {}, view) {
        /*
         * `this` cannot be used before super().
         *
         * We therefore capture the instance through `self`.
         * The handlers are created before super() because the FSM
         * expects them in the transitions object, but they don't
         * access `self` until they are actually executed.
         */
        let self;

        // ================================================================
        // FSM handlers
        // ================================================================
        const idle = () => {
            self.interaction = null;
        };

        const start = (e) => {
            const coo = Utils.relMouseCoords(e);
            console.log(coo)
            const lonlat = self.view.aladin.pix2world(coo.x, coo.y);
            if (lonlat) {}

            let interaction = self._getInteraction(coo);
            self.interaction = interaction;

            if (interaction?.type === "move") {
                self._startMove(lonlat);

                self.dispatch("moving", e);
            } else if (interaction?.type === "resize") {
                self._startResize(lonlat, interaction.handle);

                self.dispatch("resizing", e);
            } else {
                self.dispatch("idle");
            }
        };

        const moving = (e) => {
            const coo = Utils.relMouseCoords(e);
            const lonlat = self.view.aladin.pix2world(coo.x, coo.y);

            self._move(lonlat);
            self.movingCallback && self.movingCallback(self.getSelection());

            self.view.requestRedraw();
        };

        const resizing = (e) => {
            const coo = Utils.relMouseCoords(e);
            const lonlat = self.view.aladin.pix2world(coo.x, coo.y);

            self._resize(lonlat);
            self.resizeCallback && self.resizeCallback(self.getSelection());

            self.view.requestRedraw();
        };

        const mouseup = () => {
            console.log("mouseup circle")

            self._finishGesture();

            self._emitSelection();

            self.dispatch("idle");

            self.view.requestRedraw();
        };

        const mouseout = () => {
            /*
             * Keep the original behavior from your implementation:
             * if the pointer leaves while dragging, finish the gesture.
             */
            if (self.dragStart) {
                mouseup();
            }
        };

        let canvas = document.createElement("canvas");

        /*
         * FSM construction.
         *
         * No `this` is accessed here.
         */
        super({
            state: "idle",
            transitions: {
                idle: {
                    start,
                },

                start: {
                    idle,
                    moving,
                    resizing,
                },

                moving: {
                    mousemove: moving,
                    mouseup,
                    mouseout,
                    idle,
                },

                resizing: {
                    mousemove: resizing,
                    mouseup,
                    mouseout,
                    idle,
                },
            },
            view,
            canvas,
        });

        /*
         * From this point onward `this` is valid.
         */
        self = this;

        // ================================================================
        // Configuration
        // ================================================================

        this.options = {
            minRadius: 0.0,
            maxRadius: 2.0,

            radius: 0.1,

            // ------------------------------------------------------------
            // Handle appearance
            // ------------------------------------------------------------

            handleRadius: 5,
            handleHoverRadius: 7,

            handleLineWidth: 2,
            handleHoverLineWidth: 4,

            // Hit area can be larger than the visual handle.
            handleHitRadius: 10,

            // ------------------------------------------------------------

            ...options,
        };

        this.view = view;
        this.prevCursor = view.getCursor();

        // ================================================================
        // Persistent geometry
        // ================================================================


        let centerRaDec = (options && options.centerRaDec) || [view.viewCenter.ra, view.viewCenter.dec];
        let radius = Math.min(Math.max(this.options.radius, this.options.minRadius), this.options.maxRadius); 

        this.shape = new Circle(centerRaDec, radius, this.options);

        // ================================================================
        // Temporary gesture state
        // ================================================================

        this.dragStart = null;

        // Which handle is currently hovered?
        //
        // null
        // "move"
        // "resize"
        this.hoveredHandle = null;

        // ================================================================
        // Callback
        // ================================================================

        // A final callback
        this.callback = options.callback;
        // A callback executed when resizing
        this.resizeCallback = options.resizeCallback;
        // A callback executed when moving
        this.movingCallback = options.movingCallback;

        // ================================================================
        // Dedicated transparent overlay
        // ================================================================

        this.canvas = canvas;
        this.canvas.className = "circle-selector-overlay";

        this.ctx = this.canvas.getContext("2d");

        /*
         * Position the canvas over the Aladin viewport.
         */
        this.canvas.style.position = "absolute";
        this.canvas.style.left = "0";
        this.canvas.style.top = "0";

        this.interaction = null;

        /*
         * Important:
         *
         * The overlay is visual only.
         * Aladin's existing event system continues to receive
         * the mouse events.
         */
        this.canvas.style.pointerEvents = "none";

        const container = view.viewDiv;

        container.appendChild(this.canvas);

        // Keep track of canvas dimensions in CSS pixels.
        this.width = 0;
        this.height = 0;

        this._resizeCanvas();

        /*
         * Redraw if the viewport changes size.
         */
        this._resizeObserver = new ResizeObserver(() => {
            this._resizeCanvas();
            //this.view.requestRedraw();
        });

        this._resizeObserver.observe(container);
    }

    // ================================================================
    // Canvas
    // ================================================================

    _resizeCanvas() {
        const rect = this.view.aladin.aladinDiv.getBoundingClientRect();

        const dpr = window.devicePixelRatio || 1;

        this.width = rect.width;
        this.height = rect.height;

        this.canvas.width = Math.round(rect.width * dpr);
        this.canvas.height = Math.round(rect.height * dpr);

        this.canvas.style.width = `${rect.width}px`;
        this.canvas.style.height = `${rect.height}px`;

        /*
         * Draw using CSS-pixel coordinates while retaining
         * high-DPI rendering.
         */
        this.ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    }

    // ================================================================
    // Geometry
    // ================================================================

    /**
     * Return the position of the center/move handle.
     */
    _getCenterHandle() {
        if (!this.shape.center)
            return null;

        return {
            x: this.shape.center.x,
            y: this.shape.center.y,
        };
    }

    /**
     * Return the position of the resize handle.
     *
     * The handle is placed at -45 degrees.
     */
    _getResizeHandles() {
        if (!this.shape.center || !Utils.isNumber(this.shape.radius) || this.shape.isTooSmall)
            return null

        return {
            top: {
                x: this.shape.center.x,
                y: this.shape.center.y - this.shape.radius,
            },

            right: {
                x: this.shape.center.x + this.shape.radius,
                y: this.shape.center.y,
            },

            bottom: {
                x: this.shape.center.x,
                y: this.shape.center.y + this.shape.radius,
            },

            left: {
                x: this.shape.center.x - this.shape.radius,
                y: this.shape.center.y,
            },
        };
    }

    /**
     * Euclidean distance between two points.
     */
    _distance(a, b) {
        const dx = a.x - b.x;
        const dy = a.y - b.y;

        return Math.sqrt(dx * dx + dy * dy);
    }

    /**
     * Determine which part of the circle was clicked/hovered.
     *
     * Returns:
     *
     *   "move"
     *   "resize"
     *   null
     */
    _getInteraction(coo) {
        /*
         * Check resize handles first.
         */
        const handles = this._getResizeHandles();
        if (handles) {
            for (const [name, handle] of Object.entries(handles)) {
                if (this._distance(coo, handle) <= this.options.handleHitRadius) {
                    return {
                        type: "resize",
                        handle: name,
                    };
                }
            }
        }
        

        /*
         * Center = move.
         */
        const centerHandle = this._getCenterHandle();
        if (centerHandle) {
            if (this._distance(coo, centerHandle) <= this.options.handleHitRadius) {
                return {
                    type: "move",
                };
            }
        }

        return null;
    }

    // ================================================================
    // Move
    // ================================================================

    _startMove(coo) {
        this.dragStart = coo;
    }

    _move(lonlat) {
        this.shape.setCenter(lonlat)
    }

    // ================================================================
    // Resize
    // ================================================================

    _startResize(lonlat, handle) {
        this.dragStart = {
            lon: lonlat[0],
            lat: lonlat[1],
        };

        this.resizeHandle = handle;
    }

    _resize(lonlat) {
        let radius = this.view.wasm.angularDist(lonlat[0], lonlat[1], this.shape.centerRaDec[0], this.shape.centerRaDec[1]);

        radius = Math.max(this.options.minRadius, radius);
        radius = Math.min(this.options.maxRadius, radius);

        this.shape.setRadius(radius);
    }

    // ================================================================
    // Gesture management
    // ================================================================

    _finishGesture() {
        this.dragStart = null;
        this.resizeHandle = null;
    }

    // ================================================================
    // Drawing
    // ================================================================

    draw() {
        const ctx = this.ctx;

        ctx.clearRect(0, 0, this.width, this.height);
        // ============================================================
        // Circle
        // ============================================================
        this.shape.draw(ctx, this.view, false, true)

        this.prevCursor = this.view.getCursor();

        if (this._getInteraction(this.view.xy)) {
            this.view.setCursor('pointer');

            this.shape.hover()
        } else {
            //this.view.setCursor(this.prevCursor);

            this.shape.unhover()
        }

        // ============================================================
        // Handles
        // ============================================================
        console.log("too small", this.shape.isTooSmall)
        if (!this.shape.isTooSmall)
            this._drawResizeHandles(ctx, this.shape.color);

        this._drawCenterHandle(ctx, this.shape.color);
    }

    // ================================================================
    // Center handle
    // ================================================================

    _drawCenterHandle(ctx, color) {
        const handle = this._getCenterHandle();

        const hovered = this.interaction?.type === "move";

        const radius = hovered
            ? this.options.handleHoverRadius
            : this.options.handleRadius;

        const lineWidth = hovered
            ? this.options.handleHoverLineWidth
            : this.options.handleLineWidth;

        ctx.beginPath();

        ctx.arc(handle.x, handle.y, radius, 0, 2 * Math.PI);

        ctx.fillStyle = `${color}66`;

        ctx.fill();

        ctx.strokeStyle = color;

        ctx.lineWidth = lineWidth;

        ctx.stroke();
    }

    // ================================================================
    // Resize handle
    // ================================================================

    _drawResizeHandles(ctx, color) {
        const handles = this._getResizeHandles();

        for (const [_, handle] of Object.entries(handles)) {
            const hovered = this.interaction?.type === "resize";
;
            const radius = hovered
                ? this.options.handleHoverRadius
                : this.options.handleRadius;

            const lineWidth = hovered
                ? this.options.handleHoverLineWidth
                : this.options.handleLineWidth;

            ctx.beginPath();

            ctx.arc(handle.x, handle.y, radius, 0, 2 * Math.PI);

            /*
             * Same transparent green fill.
             */
            ctx.fillStyle = `${color}66`;

            ctx.fill();

            /*
             * Green contour.
             *
             * The contour becomes thicker when hovered.
             */
            ctx.strokeStyle = color;

            ctx.lineWidth = lineWidth;

            ctx.stroke();
        }
    }

    // ================================================================
    // Selection object
    // ================================================================

    getSelection() {
        const ra = this.shape.centerRaDec[0];
        const dec = this.shape.centerRaDec[1];
        const r = this.shape.radiusDegrees;
        const x = this.shape.center.x;
        const y = this.shape.center.y;

        return {
            x,
            y,
            ra,
            dec,
            r,
        };
    }

    // ================================================================
    // Selection callback
    // ================================================================

    _emitSelection() {
        const selection = this.getSelection();

        if (typeof this.callback === "function") {
            this.callback(selection);
        }
    }

    // ================================================================
    // Public API
    // ================================================================

    setRadius(radius) {
        radius = Math.max(
            this.options.minRadius,
            Math.min(this.options.maxRadius, radius),
        );

        this.shape.setRadius(radius);

        this.view.requestRedraw();
    }

    setCenter(ra, dec) {
        this.shape.setCenter([ra, dec]);
    }

    getCenter() {
        return {
            ra: this.shape.centerRaDec[0],
            dec: this.shape.centerRaDec[1],
        };
    }

    getScreenCenter() {
        return {
            x: this.shape.center.x,
            y: this.shape.center.y,
        };
    }

    getRadius() {
        return this.shape.radiusDegrees;
    }

    isInteracting() {
        //console.trace("interaction", this._getInteraction(this.view.xy), this.interaction)
        return this._getInteraction(this.view.xy) !== null || (this.interaction !== null && this.interaction !== undefined);
    }

    /**
     * Explicitly resize the overlay.
     *
     * Useful if the Aladin viewport is resized manually.
     */
    resize() {
        this._resizeCanvas();

        this.view.requestRedraw();
    }

    /**
     * Clean up the selector.
     */
    destroy() {
        if (this._resizeObserver) {
            this._resizeObserver.disconnect();

            this._resizeObserver = null;
        }

        if (this.canvas) {
            this.canvas.remove();
        }

        this.view.setCursor("default");

        this._finishGesture();
    }

    setColor(color) {
        this.shape.setColor(color);
        this.options = { ...this.options, color };
    }
}
