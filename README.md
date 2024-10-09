# [Aladin Lite](https://aladin.u-strasbg.fr/AladinLite)

**An astronomical HiPS visualizer in the browser** <img src="aladin-logo.png" alt="Aladin Lite logo" width="220">

Aladin Lite is a Web application which enables HiPS visualization from the browser. It is developed at [CDS, Strasbourg astronomical data center](http://cds.unistra.fr/).

See [A&A 578, A114 (2015)](https://arxiv.org/abs/1505.02291) and [IVOA HiPS Recommendation](http://ivoa.net/documents/HiPS/index.html) for more details about the HiPS standard.

Aladin Lite is built to be easily embeddable in any web page. It powers astronomical portals like [ESASky](https://sky.esa.int/), [ESO Science Archive portal](http://archive.eso.org/scienceportal/) and [ALMA Portal](https://almascience.eso.org/asax/).

More details on [Aladin Lite documentation page](http://aladin.u-strasbg.fr/AladinLite/doc/).
A new [API technical documentation](https://cds-astro.github.io/aladin-lite/) is now available.

[![Run tests](https://github.com/cds-astro/aladin-lite/actions/workflows/test.yml/badge.svg)](https://github.com/cds-astro/aladin-lite/actions/workflows/test.yml)
[![API Documentation](https://img.shields.io/badge/API-documentation-blue.svg)](https://cds-astro.github.io/aladin-lite)
[![Releases page](https://img.shields.io/badge/Releases-forge-yellow.svg)](https://aladin.cds.unistra.fr/AladinLite/doc/release/)

Aladin Lite is available [at this link](https://aladin.u-strasbg.fr/AladinLite).

## Running & editable JS examples

<!-- Examples -->
<table><tbody>
<tr><td>Basic Aladin Lite setup</td><td>Load SIMBAD & NED catalog data</td><td>Load a FITS image</td></tr>
    <tr><td align="left"><a href="https://aladin.cds.unistra.fr/AladinLite/doc/API/examples/AL-in-responsive-div/"><img height="200" src="https://github.com/cds-astro/aladin-lite/blob/develop/assets/vignettes/Basic%20Aladin%20Lite%20instanciation.png?raw=true"></img></a></td>
        <td align="center"><a href="https://aladin.cds.unistra.fr/AladinLite/doc/API/examples/easy-access-simbad-ned/"><img height="200" src="https://github.com/cds-astro/aladin-lite/blob/develop/assets/vignettes/Load%20SIMBAD%20&%20NED%20catalogs%20data.png?raw=true"></img></a></td>
        <td align="right"><a href="https://aladin.cds.unistra.fr/AladinLite/doc/API/examples/load-FITS-image-URL/"><img height="200" src="https://github.com/cds-astro/aladin-lite/blob/develop/assets/vignettes/Load%20a%20FITS%20image.png?raw=true"></img></a></td></tr><tr>
        <td>American Astronomical Society 225<br/>example</td><td>Display proper motion vectors</td><td>Visualization of Mars</td></tr><tr>
            <td align="left"><a href="https://aladin.cds.unistra.fr/AladinLite/doc/API/examples/AAS225/"><img height="200" src="https://github.com/cds-astro/aladin-lite/blob/develop/assets/vignettes/American%20Astronomical%20Society%20225%20demonstration.png?raw=true"></img></a></td>
            <td align="center"><a href="https://aladin.cds.unistra.fr/AladinLite/doc/API/examples/show-proper-motions/"><img height="200" src="https://github.com/cds-astro/aladin-lite/blob/develop/assets/vignettes/Display%20proper%20motions.png?raw=true"></img></a></td>
            <td align="right"><a href="https://aladin.cds.unistra.fr/AladinLite/doc/API/examples/mars-visualisation/"><img height="200" src="https://github.com/cds-astro/aladin-lite/blob/develop/assets/vignettes/Visualization%20of%20Mars.png?raw=true"></img></a></td></tr></tbody></table>
<!-- Examples -->

## Releases

A [release page](https://aladin.cds.unistra.fr/AladinLite/doc/release/) keeps track of all the current and previous builds.
A ``release`` and ``beta`` versions, regularly updated are available. The ``beta`` version is usually more advanced than the ``release`` one but features more error prone and not production-ready code.

> [!TIP]
> If you are working on a project that uses Aladin Lite, prefer working with a fixed version. Every build is tagged with a version number and accessible with:
>
> ```https://aladin.cds.unistra.fr/AladinLite/api/v3/<version>/aladin.js```

## Documentation

There is a new API documentation available [here](https://cds-astro.github.io/aladin-lite).
Editable examples showing the API can also be found [here](https://aladin.cds.unistra.fr/AladinLite/doc/API/examples/).

## Embed it into your projects

You can embed Aladin Lite it into your webpages in two ways

### The vanilla way

Please include [the javascript script of Aladin Lite v3](https://aladin.cds.unistra.fr/AladinLite/api/v3/latest/aladin.js) into your project. API differences from the v2 are minimal, here is a snippet of code you can use to embed it into your webpages:

```html
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

### NPM deployment

A [NPM package](https://www.npmjs.com/package/aladin-lite) is deployed and maintained. It is used by [ipyaladin](https://github.com/cds-astro/ipyaladin), a jupyter widget allowing to run aladin lite in a notebook.

```npm i aladin-lite```

Aladin Lite can be imported with:

```html
<script type="module">
    import A from 'aladin-lite';
    // your code...
</script>
```

## New features

* [X] Rust/WebGL2 new rendering engine
* [X] Remove jQuery dep
* [ ] UI dev, better support for smartphones
* [X] FITS images support
* [X] WCS parsing, displaying an (JPEG/PNG) image in aladin lite view
* [X] Display customized shapes (e.g. proper motions) from astronomical catalog data
* [X] AVM tags parsing support
* [X] Easy sharing of current « view »
* [ ] All VOTable serializations
* [ ] FITS tables
* [X] Creating HiPS instance from an URL
* [X] Local HiPS loading 
* [X] Multiple mirrors handling for HiPS tile retrival
* [ ] HiPS cube

## Licence

Aladin Lite is currently licensed under GPL v3.0

If you think this license might prevent you from using Aladin Lite in your pages/application/portal, please open an issue or [contact us](mailto:cds-question@unistra.fr)

## Contribution guidelines

There are several ways to contribute to Aladin Lite:

- **report a bug**: anyone is welcome to open an issue to report a bug. Please make sure first the issue does not exist yet. Be as specific as possible, and provide if possible detailed instructions about how to reproduce the problem.

- **suggest a new feature**: if you feel something is missing, check first if a similar feature request has not already been submitted in the open issues. If not, open a new issue, and give a detailed explanation of the feature you wish.

- **develop new features/provide code fixing bugs**. As open development is a new thing for us, we will in a first time only take into consideration code contribution (_i.e._ Pull Requests) from our close partners.
In any case, please get in touch before starting a major update or rewrite.

### Building steps

First you need to install the dependencies from the package.json
Please run:

```sh
npm install
```

After that you are supposed to have the Rust toolchain installed
to compile the core project into WebAssembly.
Follow the steps from the Rust official website [here](https://www.rust-lang.org/learn/get-started)
You will also need [wasm-pack](https://rustwasm.github.io/wasm-pack/), a tool helping compiling rust into a proper .wasm file.

Once it's installed you will need to switch to the nightly rust version:

```sh
rustup default nightly
```

Then you can build the project:

```sh
npm run build
```

> [!WARNING]
> **If you are experimenting Rust compiling issues:**
> - Make sure you have your **wasm-pack** version updated. To do so:
> ```sh
> cargo install wasm-pack --version ~0.12
> ```
> - Make sure you are using the rust **nightly** toolchain
> ```sh
> rustup default nightly
> ```
> - Remove your `src/core/Cargo.lock` file and `src/core/target` directory -- this ensures that you'd escape any bad compilation state:
> ```sh
> git clean -di
> ```
> -  then recompile with `npm run build`.

It will generate the aladin lite compiled code into a `dist/` directory located at the root of the repository. This directory contains two javascript files. `aladin.umd.cjs` follows the UMD module export convention and it is the one you need to use for your project.

### Testing guidelines

#### Run the examples

A bunch of examples are located into the `examples` directory.
To run them, start a localhost server:

```sh
npm run serve
```

#### Rust tests

These can be executed separately from the JS part:

* Compile the Rust code:

```sh
cd src/core
cargo check --features webgl2
```

* Run the tests:

```sh
cargo test --features webgl2
```

#### Snapshot comparisons

We use [playwright](https://playwright.dev/) for snapshot comparison testing. Only ground truth snapshots have been generated for MacOS/Darwin architecture.
First install playwright:

```sh
npx playwright install
```

Run the tests, advises are given for opening the UI mode or for generating your own ground truth snapshots.

```sh
npm run test:playwright
```
