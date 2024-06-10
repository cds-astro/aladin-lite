# Customization

This is a guide for users wanting to customize the apparence of Aladin Lite user interface.

## CSS class names

There are distincts CSS class names for users wanting to personnalize the default elements. These classes are listed below:

* `aladin-stack-control` targets the stack opener button
* `aladin-fullScreen-control` targets the fullscreen control button
* `aladin-simbadPointer-control` targets the Simbad pointer control button
* `aladin-grid-control`  targets the coordinate grid trigger button
* `aladin-settings-control` targets the settings menu opener button
* `aladin-share-control` targets the share menu opener button
* `aladin-projection-control` targets the projection selector button
* `aladin-stack-box` targets the stack box
* `aladin-status-bar` targets the status bar frame
* `aladin-cooFrame` targets the frame selector element
* `aladin-location` targets the search bar and location information element
* `aladin-fov` targets the field of view information display element

This example changes the position of the Aladin Lite search bar to the very top-left of Aladin Lite and it disables the frame.

```js
<!doctype html>
<html>
    <head>
        <meta name="viewport" content="width=device-width, height=device-height, maximum-scale=1.0, initial-scale=1.0, user-scalable=no">
    </head>
<body>

<div id="aladin-lite-div" style="width: 768px; height: 512px"></div>

<script type="module">
    import A from '../src/js/A.js';
    let aladin;
    A.init.then(() => {
        aladin = A.aladin(
            '#aladin-lite-div',
            {
                survey: 'P/allWISE/color', // set initial image survey
                projection: 'AIT', // set a projection
                fov: 1.5, // initial field of view in degrees
                target: 'NGC 2175', // initial target
                cooFrame: 'icrs', // set galactic frame
                reticleColor: '#ff89ff', // change reticle color
                reticleSize: 64, // change reticle size
                showContextMenu: true,
            }
        );
    });
</script>
<style>
    .aladin-location {
        position: absolute;
        left: 0.2rem;
        top: 0.2rem;
    }

    .aladin-cooFrame {
        display: none;
    }
</style>
</body>
</html>
```
