# 3.3.0

* New documentation API (W.I.P) here: https://cds-astro.github.io/aladin-lite/
* New release page here: https://aladin.cds.unistra.fr/AladinLite/doc/release/
* A major UI update:
  1. Some API new classes A.box, A.button
  2. A status bar where the user can enque messages for a specific amount of time (Aladin.appendStatusBarMessage)
* Remove of JQuery and autocompletejs dependencies
* Fix some performances issues, i.e. a bug when resizing the aladin lite view and which launched several parallel requestAnimationFrame.
* Polygon and circular selection (see Aladin class API documentation for how to use it)
* ObsCore and Datalink votable parsing and interpretation. This work is still in progress and made in the frame of the SKA radio mission.

# 3.2.0

* MOC rendering perf enhanced. Possibility to draw only the perimeter of a MOC object (perimeter set to True)
* Many fixes e.g. footprint rendering for all sky projections
* A line/shape webgl rasterizer thanks to the use of the `lyon`crate. MOCs and grid lines are rendered that way. Therefore, it is possible to change the grid lines thickness
* Use of vite for the project management and deployment

# 3.1.0

## What's Changed
* Add message for safari users to enable WebGL2 feature and reload the page by @bmatthieu3 in https://github.com/cds-astro/aladin-lite/pull/54
* Starting fits support by @bmatthieu3 in https://github.com/cds-astro/aladin-lite/pull/70
* display fits images with the drag and drop by @bmatthieu3 
![Kapture 2023-03-23 at 14 34 28](https://user-images.githubusercontent.com/2772384/227264124-8e05a3d8-1565-497f-a118-39fab3c6ed83.gif)
* support `webp` tile format by @bmatthieu3 and @tboch 
* planetary name resolver by @tboch 
* small ui changes and bug fixes by @bmatthieu3
* add codemeta and its validatior action by @ManonMarchand in https://github.com/cds-astro/aladin-lite/pull/66


# 3.0.0

Official release of Aladin Lite v3, [as announced in CDS news](https://cds.unistra.fr/news.php?fn_mode=fullnews&fn_incl=0&fn_id=958).

## What's Changed
* Fix missing tiles issue by @tboch in https://github.com/cds-astro/aladin-lite/pull/18
* Hips catalogue filtering by @tboch in https://github.com/cds-astro/aladin-lite/pull/28
* Make footprint selection easier by @tboch in https://github.com/cds-astro/aladin-lite/pull/19
* Bug fix: enable different colors for multiple polylines in same layer by @tboch in https://github.com/cds-astro/aladin-lite/pull/30
* Method remove to delete individual source from a catalogue layer by @tboch in https://github.com/cds-astro/aladin-lite/pull/37
* Stop animation by @tboch in https://github.com/cds-astro/aladin-lite/pull/40
* Add message for safari users to enable WebGL2 feature and reload the page by @bmatthieu3 in https://github.com/cds-astro/aladin-lite/pull/54
