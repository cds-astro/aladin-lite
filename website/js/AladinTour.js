
AladinTour = (function() {

    // constructor
    // @param aladin: reference to Aladin Lite instance
    AladinTour = function(aladin) {
        this.aladin = aladin;
    };

    // dict of predefined visits and associated parameters,
    // indexed by the visit name
    var PREDEFINED_VISITS = {
        'simbad-popular': {
            label: 'Simbad most popular objects',
            url: ''
        },
        'gaia-dr2-closest': {
            label: 'Gaia DR2 closest stars',
            url: ''
        },
        'gaia-dr2-brightest': {
            label: 'Gaia DR2 closest stars',
            url: ''
        },
        'galaxies': {
            label: 'A selection of galaxies',
            url: ''
        },
        'messier': {
            label: 'Messier objects',
            url: ''
        },
    };

    AladinTour.prototype = {
        
        setVisit: function(visitName) {

        }

    };

    return AladinTour;

})();

