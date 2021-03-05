# Aladin Lite v3

**An astronomical HiPS visualizer in the browser** <img src="aladin-logo.png" alt="Aladin Lite logo" width="220">

Aladin Lite is a Web application which enables HiPS visualization from the browser. It is developed at [CDS, Strasbourg astronomical data center](http://cds.unistra.fr/).

See [A&A 578, A114 (2015)](https://arxiv.org/abs/1505.02291) and [IVOA HiPS Recommendation](http://ivoa.net/documents/HiPS/index.html) for more details about the HiPS standard.

Aladin Lite is built to be easily embeddable in any web page. It powers astronomical portals like [ESASky](https://almascience.eso.org/asax/), [ESO Science Archive portal](http://archive.eso.org/scienceportal/) and [ALMA Portal](https://almascience.eso.org/asax/).

More details on [Aladin Lite documentation page](http://aladin.u-strasbg.fr/AladinLite/doc/).

This repo contains the Aladin Lite v3 source code and specifically the code of its new WebGL core written in Rust.

## How to test it ?

You can test it [here](https://bmatthieu3.github.io/hips_webgl_renderer/test_moc_moll.html)!

For Safari users only: make sure to enable WebGL2 experimental feature and refresh the page once it is done. You can find it in the Developer Menu > Experimental Features > WebGL2.
Safari will soon [enable WebGL2 by default](https://developer.apple.com/safari/technology-preview/release-notes/).

Do not hesitate to give a feedback either by sending a mail to:

- baumannmatthieu0@gmail.com
- thomas.boch@astro.unistra.fr

or simply by posting an issue in this repo.

## Goals of v3

- Rust/WebGL new core integration

- Remove jQuery dep

- UI dev, using VueJS, better support for smartphones

- package the core and its API as a WASM npm package

- FITS images support

- easy sharing of current « view »

- support of all VOTable serializations (using votable.js?)

- support of FITS tables?

- creating HiPS instance from an URL

- multiple mirrors handling for HiPS tile retrival

## Source code

Source code is available in the ``src`` directory.
Precisely, the core is implemented in Rust and can be found in ``src/core``.

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


Once it's installed you can only build the project:
```bash
npm run build
```

Or build it and launch a localhost server (usually starting on port 8080 but it can be another one if 8080 is occupied):
```bash
npm run serve
```

If you just want to check the Rust code without compiling, from the root location:

```bash
cd src/core
cargo check
```

and run the tests

```bash
cd src/core
cargo test
```

To generate the Rust backend API documentation

```bash
cd src/core
cargo doc --no-deps --open
```
