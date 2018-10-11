var aladin = A.aladin('#aladin-lite-div', {target: '18 38 5.286 -06 25 27.88', fov: 30});
aladin.setFrame('galactic');

aladin.setBaseImageLayer(aladin.createImageSurvey("PS1 z", "PS1 z", "http://alasky.u-strasbg.fr/Pan-STARRS/DR1/z", "equatorial", 11, {imgFormat: 'jpg', additionalParams: 'cmap=Greys_r'}));

function updateHiPS() {
    var stretch = $('#stretch').val();
    var cmap = $('#cmap').val();
    var min_cut = $('#min_cut').val();
    var max_cut = $('#max_cut').val();
    aladin.setBaseImageLayer(aladin.createImageSurvey("PS1 z", "PS1 z", "http://alasky.u-strasbg.fr/Pan-STARRS/DR1/z", "equatorial", 11, {imgFormat: 'jpg', additionalParams: 'stretch=' + stretch + '&cmap=' + cmap + '&min_cut=' + min_cut + '&max_cut=' + max_cut}));

}

var cmaps = ['Accent', 'Accent_r', 'Blues', 'Blues_r', 'BrBG', 'BrBG_r', 'BuGn', 'BuGn_r', 'BuPu', 'BuPu_r', 'CMRmap', 'CMRmap_r', 'Dark2', 'Dark2_r', 'GnBu', 'GnBu_r', 'Greens', 'Greens_r', 'Greys', 'Greys_r', 'OrRd', 'OrRd_r', 'Oranges', 'Oranges_r', 'PRGn', 'PRGn_r', 'Paired', 'Paired_r', 'Pastel1', 'Pastel1_r', 'Pastel2', 'Pastel2_r', 'PiYG', 'PiYG_r', 'PuBu', 'PuBuGn', 'PuBuGn_r', 'PuBu_r', 'PuOr', 'PuOr_r', 'PuRd', 'PuRd_r', 'Purples', 'Purples_r', 'RdBu', 'RdBu_r', 'RdGy', 'RdGy_r', 'RdPu', 'RdPu_r', 'RdYlBu', 'RdYlBu_r', 'RdYlGn', 'RdYlGn_r', 'Reds', 'Reds_r', 'Set1', 'Set1_r', 'Set2', 'Set2_r', 'Set3', 'Set3_r', 'Spectral', 'Spectral_r', 'Wistia', 'Wistia_r', 'YlGn', 'YlGnBu', 'YlGnBu_r', 'YlGn_r', 'YlOrBr', 'YlOrBr_r', 'YlOrRd', 'YlOrRd_r', 'afmhot', 'afmhot_r', 'autumn', 'autumn_r', 'binary', 'binary_r', 'bone', 'bone_r', 'brg', 'brg_r', 'bwr', 'bwr_r', 'cool', 'cool_r', 'coolwarm', 'coolwarm_r', 'copper', 'copper_r', 'cubehelix', 'cubehelix_r', 'flag', 'flag_r', 'gist_earth', 'gist_earth_r', 'gist_gray', 'gist_gray_r', 'gist_heat', 'gist_heat_r', 'gist_ncar', 'gist_ncar_r', 'gist_rainbow', 'gist_rainbow_r', 'gist_stern', 'gist_stern_r', 'gist_yarg', 'gist_yarg_r', 'gnuplot', 'gnuplot2', 'gnuplot2_r', 'gnuplot_r', 'gray', 'gray_r', 'hot', 'hot_r', 'hsv', 'hsv_r', 'inferno', 'inferno_r', 'jet', 'jet_r', 'magma', 'magma_r', 'nipy_spectral', 'nipy_spectral_r', 'ocean', 'ocean_r', 'pink', 'pink_r', 'plasma', 'plasma_r', 'prism', 'prism_r', 'rainbow', 'rainbow_r', 'seismic', 'seismic_r', 'spectral', 'spectral_r', 'spring', 'spring_r', 'summer', 'summer_r', 'terrain', 'terrain_r', 'viridis', 'viridis_r', 'winter', 'winter_r'];
$('#cmap').append('<option value="random">random</option>');
for (var k=0; k<cmaps.length; k++) {
    $('#cmap').append('<option value="' + cmaps[k] + '">' + cmaps[k] + '</option>');
}
$('#cmap').val('Greys_r');

$('#stretch, #cmap, #min_cut, #max_cut').on('change', function() {
    updateHiPS();
});
