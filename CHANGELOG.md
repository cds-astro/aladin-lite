# Changelogs

## 3.4.3-beta

* [bugfix] zoom control buttons

## 3.4.2-beta

* [impr] Improve smartphone support by setting media queries + a better logic for deploying the contextual menu sub options.
* [impr] Improve `WCS` view export with 3rd euler rotation encoding: <https://github.com/cds-astro/aladin-lite/issues/170>. Still some cases are to be handled like: crval on the equator or cylindrical with a galactic frame rotation.
* [fixed] Change `RADECSYS` to `RADESYS` for `Aladin#getViewWCS` to follow fits standard deprecation
* [feat] Add new method `Aladin#getViewImageBuffer` to get the current view as a PNG buffer
* [feat] New line rasterizer using GL instancing. This enhances the rendering speed of MOCs.

## 3.3.3

* [feat] UI: add HiPS basic filter that filters the `hipsList` given
* [feat] New `hipsList` option parameter when instancing a new Aladin object.
* [feat] Zoom smoothing using hermite cubic interpolation functions
* [feat] shape option of Catalog and ProgressiveCat accepts a function returning a Footprint. This allow user to
         associate a footprint to a specific source
* [feat] Hover color support by @pmatsson and @bmatthieu3 in <https://github.com/cds-astro/aladin-lite/pull/145>

## 3.3.2

* [fixed] do not allow to query the properties several times for an imageHiPS
* [fixed] Detecting raytracing rendering mode. Adapt the rendering mode in function of the fov value and the projection used. Some projections do have more distortions with wide FoVs so it is better to use the raytracing rendering mode when fov >= smaller FoV threshold.

## 3.3.0

* [fixed] multiple calls to setImageSurvey with the same survey object led to strange behaviour.
* [perf] Display the first tile received instantly with no blending. Should enhance the slow reported in issue #88.
* [fixed] A.on('select') (debugged from ipyaladin)  
* [fixed] Simbad pointer in galactical frame, cone search of simbad/vizier cats/other cone search services in galactical frame and MOC creation from selection in galactical frame => there is now a new `frame` optional param to Aladin.pix2world. If not given, the coo returned are in the frame of the view.
* [doc] Add doc for image survey definition
* [deprecation] A.createImageSurvey/A.newImageSurvey are now deprecated (but still in the API). Please use `A.imageHiPS` instead by providing a valid url or CDS ID conformed to <https://aladin.cds.unistra.fr/hips/list>
* [refac] Simplify the instanciation of an imageHiPS/ imageFITS. Add a `A.imageHiPS` method for defining a HiPS object
* [fixed] At initialisation, giving a fov > 180 was clamped back to 180 even if we specify allsky projection (i.e. accepting fov > 180). This is now fixed.
* [fixed] MeasurementTable now display the full cell values (no ellipsis anymore)
* [fixed] aladin.on('select') has been implemented. Callback is triggered on a circle and rect selections for not on polygonal selection.
* [fixed] the cooFrame UI selector is updated if the user calls `aladin.setFrame`
* [fixed] `reticleColor` and `reticleSize` options in the public API 
* Restore setFoVRange
* Add CSS class for positioning the UI elements as the user wants. See the API doc aladin options for the class names to use.
* [style] The default grid color is now `rgb(178, 50, 178)` to fit the classic Aladin color palette
* [feat] The object of grid options `gridOptions` is now available in the public API
* [fixed] The parameters `gridColor` and `gridOpacity`, `gridOptions.showLabels` now work as expected
* New documentation API (W.I.P) here: <https://cds-astro.github.io/aladin-lite/>
* New release page here: <https://aladin.cds.unistra.fr/AladinLite/doc/release/>
* A major UI update by @bmatthieu3
  1. Some API new classes A.box, A.button
  2. A status bar where the user can enque messages for a specific amount of time (Aladin.addStatusBarMessage)
* Remove of JQuery and autocompletejs dependencies by @bmatthieu3
* Fix some performances issues, i.e. a bug when resizing the aladin lite view and which launched several parallel requestAnimationFrame by @bmatthieu3
* Polygon and circular selection (see Aladin class API documentation for how to use it)
* ObsCore and Datalink votable parsing and interpretation. This work is still in progress and made in the frame of the SKA radio mission by @bmatthieu3 in <https://github.com/cds-astro/aladin-lite/pull/116>
* SODA service query window formular by @bmatthieu3 in <https://github.com/cds-astro/aladin-lite/pull/116>
* read only catalog option by @szpetny in <https://github.com/cds-astro/aladin-lite/pull/117>
* Small changed regarding drawing a footprint by @szpetny in <https://github.com/cds-astro/aladin-lite/pull/118>
* Object and footprint click/hover events expose mouse coordinates by @szpetny in <https://github.com/cds-astro/aladin-lite/pull/121>
* A proposal of a new feature - fill the polygon with a color by @szpetny in <https://github.com/cds-astro/aladin-lite/pull/122>
* update getViewWCS to adapt to projection by @ManonMarchand in <https://github.com/cds-astro/aladin-lite/pull/119>
* New SAMP support by @bmatthieu3 in <https://github.com/cds-astro/aladin-lite/pull/128>
* A possibility to create Coo and Footprint objects by @szpetny in <https://github.com/cds-astro/aladin-lite/pull/130>
* new method aladin.getFrame() that returns the name of the current coordinate system
* `getViewWCS` now adapts to the `cooFrame` and the `projection`
* `getFov` is no longer capped at 180°
* bugfix `setProjection` now also updates for 'PAR' and 'SFL' projections

## 3.2.0

* MOC rendering perf enhanced. Possibility to draw only the perimeter of a MOC object (perimeter set to True)
* Many fixes e.g. footprint rendering for all sky projections
* A line/shape webgl rasterizer thanks to the use of the `lyon`crate. MOCs and grid lines are rendered that way. Therefore, it is possible to change the grid lines thickness
* Use of vite for the project management and deployment

## 3.1.0

* Add message for safari users to enable WebGL2 feature and reload the page by @bmatthieu3 in <https://github.com/cds-astro/aladin-lite/pull/54>
* Starting fits support by @bmatthieu3 in <https://github.com/cds-astro/aladin-lite/pull/70>
* display fits images with the drag and drop by @bmatthieu3
![Kapture 2023-03-23 at 14 34 28](https://user-images.githubusercontent.com/2772384/227264124-8e05a3d8-1565-497f-a118-39fab3c6ed83.gif)
* support `webp` tile format by @bmatthieu3 and @tboch
* planetary name resolver by @tboch
* small ui changes and bug fixes by @bmatthieu3
* add codemeta and its validatior action by @ManonMarchand in <https://github.com/cds-astro/aladin-lite/pull/66>

## 3.0.0

Official release of Aladin Lite v3, [as announced in CDS news](https://cds.unistra.fr/news.php?fn_mode=fullnews&fn_incl=0&fn_id=958).

* Fix missing tiles issue by @tboch in <https://github.com/cds-astro/aladin-lite/pull/18>
* Hips catalogue filtering by @tboch in <https://github.com/cds-astro/aladin-lite/pull/28>
* Make footprint selection easier by @tboch in <https://github.com/cds-astro/aladin-lite/pull/19>
* Bug fix: enable different colors for multiple polylines in same layer by @tboch in <https://github.com/cds-astro/aladin-lite/pull/30>
* Method remove to delete individual source from a catalogue layer by @tboch in <https://github.com/cds-astro/aladin-lite/pull/37>
* Stop animation by @tboch in <https://github.com/cds-astro/aladin-lite/pull/40>
* Add message for safari users to enable WebGL2 feature and reload the page by @bmatthieu3 in <https://github.com/cds-astro/aladin-lite/pull/54>

## 2.x.x

### 2020-08

* polyline improvements (by @imbasimba)

### 2020-07

* new method stopAnimation

### 2020-06

* new method in Catalog layer to *remove* individual *Source* objects

### 2019-10

* displayFITS can now take a base64 data URL as input

### 2019-05-03

* https URLs for Simbad pointer

### 2019-04-30

* empty Downloader queue when changing displayed HiPs

### 2019-02-06

* bug fix: MOC cells at order 0 were not displayed

### 2019-01

* add method getViewWCS

### 2018-10-30

* bug fix affichage MOC order>11

### 2018-09-24

* bug fix in VOTable parsing: CDATA text was always blank in <TD></TD>

### 2018-09-18

* drawing algorith improved: no more flickering when zooming in

### 2018-09-17

* improvement on mobile device: pinch zoom works, panning works better

### 2018-08-30

* URL generated for a VizieR cone search now take into account the option 'limit', as to limit the size of the retrieved VOTable

### 2018-06-11

* Bug fix for rectangular selection of sources (aladin.on('select', ...  )

### 2018-05-16

* Add variable View.CALLBACKS_THROTTLE_TIME_MS to control minimal time between two callbacks

### 2018-05-14

* HTTPS support for Logger

### 2018-04-20

* partially fix the all-sky view (the cells borders were visible) --> delta in method HpxImageSurvey.drawOneTile2
* add option showAllskyRing
* all-sky is shown at orders 3 and 4

### 2018-04

* Add method zoomToFoV (zoom with animation)
* doc : ajout Tour navigator library dans Plugins

### 2018-01-09

* Add option simbadPointer

### 2018-01-08

* Add option realFullscreen

### 2017-12-20

* Catalog.onClick can now also be a function

### 2017-12-14

* add function udpateShape for Catalog and ProgressiveCatalog object
* ajout shape 'circle' pour les catalogues
* bug fix: les cats progressifs ne s'affichaient pas quand on était zoomé et qu'on ne bougeait pas

### 2017-12-13

* support Circle when generating footprints from STC-S descriptions

### 2017-11-30

* improve positionChanged listener: no more called when clicking on an object
* improve object shown when clicking ; it's really the closest one now

### 2017-11-24

* add dragging attribute to positionChanged listener callback param

### 2017-10-09

* add listener for 'mouseMove' event

### 2017-09-28

* MOC display is way faster when panning
* Sesame bug fix when used in a local file web page

### 2017-09-27

* bug fix : MOC display with norder>9 (thanks to Raffaele D'Abrusco)
* improve MOC display speed

### 2017-09-21

* add listener for 'click' event
* dimensions can be specified for getViewDataURL method

### 2017-09-08

* gotoObject: can now take a success callback function
* improve AITOFF display

### 2017-09-07

* improvement: MOCs, catalogs and overlays have now different logos in the "stack"

### 2017-09-06

* fix: export PNG was not working any longer in latest version of Chrome, as top frame navigation to data URL was no longer allowed

### 2017-08-28

* add J2000d option for frame selection

### 2017-08-25

* all graphical overlays (footprints, MOCs) in addition to catalogs are now visible and can be shown/hidden from the control panel
* add skyFraction method to MOC class
* fix: when going full screen, background is now all white (no more HTML elements visible in the background)
* style: layer labels in Overlay layers panel are rounded at both ends

### 2017-08-24

* add adaptativeDisplay option for MOCs
* try first to load MOC through given URL, and only if it fails, try through proxy (allows to load local/not publicly accessible MOCs)

### July 2017

* bug fix MOC
* add method adjustFovForObject
* add listeners on positionChanged zoomChanged
* fix pour Andre Moitinho pour que Sesame fonctionne en HTTPS

### February 2017

* shape parameter when creating a catalogue can now be a custom draw function
* bug fix, when superimposing a HiPS over a HiPS with a different coordinate frame

### January 2017

* added method to query SkyBot: A.catalogFromSkyBot


### September 2016

* support of HiPS catalog (new format)

### June 2016

* fix astrometry offset by subdividing HEALPix cells too distorted
* new method A.MOCFromJSON to create a MOC from the JSON serialization

### March 2016

* added raField and decField options when creating catalogue

### December 2015

* added cubehelix color map
* added option in A.catalogFromURL to bypass proxy
* access to Simbad and VizieR data (catalogFromVizieR and catalogFromSimbad) are now done directly, without the proxy

### Novembre 2015

* ajout méthodes getShareURL et getEmbedCode

### Octobre 2015

* ajout méthodes accès facile à Simbad, NED et VizieR
* ajout onClick comme option de Catalog: 'showTable' ou 'showPopup'
* nouvelle version de jquery mousewheel (ça déconnait un peu sous Mac avec la précedente)

### Décembre 2014

* ajout displayLabel pour afficher un label d'un overlay Catalog
* ajout option lineWidth pour objet A.graphicOverlay
* bug fix : la liste déroulante est mise à jour correctement quand on change de frame programmatiquement

### Novembre 2014

* ajout nouvelles formes pour les sources

### 23 octobre 2014

* bug fix méthode on('objectClicked') était appelée de manière intempestive 
quand la souris quittait le canvas

### 21 octobre 2014

* ajout option shape (plus ou square) pour les catalogues

### 20 octobre 2014

* amélioration on objectClicked et objectHivered. On envoie une valeur nulle pour signifier qu'on quitte l'objet
* ajout méthode pour dessiner un cercle (A.circle)

### 16 septembre 2014

* bug fix pour nouvelle version de Firefox. Aladin lite freezait. On ne crée plus les textures individuelles pour le allsky

### 24 avril 2014

New in the API:
* getSize
* getFov
* world2pix
* pix2world
* getFovCorners

### Novembre 2013

* ajout catalogues progressifs
* ajout option pour fullScreen mode au démarrage
* ajout méthode setFOVRange
* polyfill pour Function.prototype.bind (pour Firefox < 4)

### Septembre 2013

* color maps

### Août 2013

* ajout page exemple full-screen.html
* ajout bouton maximize in fullscreen
* CSS dans fichier séparé
* image réticule "cachée"
* options pour personnaliser le réticule
* revamped UI for layers
* export PNG (nécessite support CORS)

### Juillet 2013

* ajout boutons zoom
* ajout sélection d'objets

### Fin 2013

* ajout catalogue progressif
* ajout on select, objectClicked, objectHovered
