# Aladin Lite v3

**An astronomical HiPS visualizer in the browser** <img src="aladin-logo.png" alt="Aladin Lite logo" width="220">

Aladin Lite is a Web application which enables HiPS visualization from the browser. It is developed at [CDS, Strasbourg astronomical data center](http://cds.unistra.fr/).

See [A&A 578, A114 (2015)](https://arxiv.org/abs/1505.02291) and [IVOA HiPS Recommendation](http://ivoa.net/documents/HiPS/index.html) for more details about the HiPS standard.

Aladin Lite is built to be easily embeddable in any web page. It powers astronomical portals like [ESASky](https://sky.esa.int/), [ESO Science Archive portal](http://archive.eso.org/scienceportal/) and [ALMA Portal](https://almascience.eso.org/asax/).

More details on [Aladin Lite documentation page](http://aladin.u-strasbg.fr/AladinLite/doc/).

# How to test it ?

Aladin Lite v3 is out! Please play with [Aladin Lite v3 at this link](https://aladin.u-strasbg.fr/AladinLite).

## Embed it into your projects

You can embed Aladin Lite it into your webpages in two ways

### The vanilla way

Please include [the javascript script of Aladin Lite v3](https://aladin.cds.unistra.fr/AladinLite/api/v3/latest/aladin.js) into your project. API differences from the v2 are minimal, here is a snippet of code you can use to embed it into your webpages:

```js
<!doctype html>
<html>
<head>
    <!-- Mandatory when setting up Aladin Lite v3 for a smartphones/tablet usage -->
    <meta name="viewport" content="width=device-width, height=device-height, initial-scale=1.0, user-scalable=no">
</head>
<body>

<div id="aladin-lite-div" style="width: 500px; height: 400px"></div>
<script type="text/javascript" src="https://aladin.cds.unistra.fr/AladinLite/api/v3/latest/aladin.js" charset="utf-8"></script>

<script type="text/javascript">
    let aladin;
    A.init.then(() => {
        aladin = A.aladin('#aladin-lite-div', {fov: 360, projection: "AIT", cooFrame: 'equatorial', showCooGridControl: true, showSimbadPointerControl: true, showCooGrid: true});
    });
</script>

</body>
</html>
```

### Using the aladin lite NPM package

First, install it with npm:

```npm i aladin-lite```

Second, you can use it that way:

```js
<!doctype html>
<html>
<head>
    <!-- Mandatory when setting up Aladin Lite v3 for a smartphones/tablet usage -->
    <meta name="viewport" content="width=device-width, height=device-height, initial-scale=1.0, user-scalable=no">
</head>
<body>

<div id="aladin-lite-div" style="width: 500px; height: 400px"></div>

<script type="module">
    import A from 'aladin-lite';

    A.init.then(() => {
        let aladin = A.aladin('#aladin-lite-div', {fov: 360, projection: "AIT", cooFrame: 'equatorial', showCooGridControl: true, showSimbadPointerControl: true, showCooGrid: true});
    });
</script>

</body>
</html>
```

## Goals of v3

- Rust/WebGL new core integration

- Remove jQuery dep

- UI dev, better support for smartphones

- FITS images support

- easy sharing of current « view »

- support of all VOTable serializations (using votable.js?)

- support of FITS tables?

- creating HiPS instance from an URL

- multiple mirrors handling for HiPS tile retrival

## Source code

Source code is available in the ``src`` directory.

## Licence

Aladin Lite is currently licensed under GPL v3.0

If you think this license might prevent you from using Aladin Lite in your pages/application/portal, please open an issue or [contact us](mailto:cds-question@unistra.fr)

## Contributing

There are several ways to contribute to Aladin Lite:

- **report a bug**: anyone is welcome to open an issue to report a bug. Please make sure first the issue does not exist yet. Be as specific as possible, and provide if possible detailed instructions about how to reproduce the problem.

- **suggest a new feature**: if you feel something is missing, check first if a similar feature request has not already been submitted in the open issues. If not, open a new issue, and give a detailed explanation of the feature you wish.

- **develop new features/provide code fixing bugs**. As open development is a new thing for us, we will in a first time only take into consideration code contribution (_i.e._ Pull Requests) from our close partners.
In any case, please get in touch before starting a major update or rewrite.

### Building the application steps

First you need to install the dependencies from the package.json
Please run:

```bash
npm install
```

After that you are supposed to have the Rust toolchain installed
to compile the core project into WebAssembly.
Follow the steps from the Rust official website [here](https://www.rust-lang.org/learn/get-started)
You will also need [wasm-pack](https://rustwasm.github.io/wasm-pack/), a tool helping compiling rust into a proper .wasm file.

Once it's installed you can only build the project:
```bash
npm run build
```

Or build it and launch a localhost server (usually starting on port 8080 but it can be another one if 8080 is occupied):
```bash
npm run serve
```

For just compiling the rust core from the root location (it is faster to do so)

```bash
cd src/core
cargo check --features webgl2
```

and run the tests

```bash
cd src/core
cargo test --features webgl2
```

To generate the Rust backend API documentation

```bash
cd src/core
cargo doc --no-deps --open
```