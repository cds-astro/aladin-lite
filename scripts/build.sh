#!/bin/bash

# Script de concaténation et minification d'Aladin Lite 
#
#
# Le résultat est créé dans le répertoire distrib
# 
# Pré-requis : 
# - uglifyjs version 2 : https://github.com/mishoo/UglifyJS2
#
#

scriptdir="$( cd "$( dirname "$0" )" && pwd )"
srcdir=${scriptdir}/../src

srctmpdir=/tmp/AL-src

# create version number
version=$(date --utc +%F)

distribdir=${scriptdir}/../distrib/${version}

if [ ! -d ${distribdir} ];
then
    mkdir ${distribdir}
fi

distribfile=${distribdir}/aladin.js
distribfileminified=${distribdir}/aladin.min.js

csssrcfile=${srcdir}/css/aladin.css
cssfileminified=${distribdir}/aladin.min.css

srctar=${distribdir}/AladinLiteSrc.tar

uglifyjs="/home/boch/tools/node_modules/.bin/uglifyjs"
lessc="/usr/bin/lessc"

jsfiles=('js/cds.js' 'js/libs/json2.js' 'js/Logger.js' 'js/libs/jquery.mousewheel.js' 'js/libs/RequestAnimationFrame.js' 'js/libs/Stats.js' 'js/libs/healpix.min.js' 'js/libs/astro/astroMath.js' 'js/libs/astro/projection.js' 'js/libs/astro/coo.js' 'js/SimbadPointer.js' 'js/Box.js' 'js/CooConversion.js' 'js/Sesame.js' 'js/HealpixCache.js' 'js/Utils.js' 'js/URLBuilder.js' 'js/MeasurementTable.js' 'js/Color.js' 'js/AladinUtils.js' 'js/ProjectionEnum.js' 'js/CooFrameEnum.js' 'js/HiPSDefinition.js' 'js/Downloader.js' 'js/libs/fits.js' 'js/MOC.js' 'js/CooGrid.js' 'js/Footprint.js' 'js/Popup.js' 'js/Circle.js' 'js/Polyline.js' 'js/Overlay.js' 'js/Source.js' 'js/Catalog.js' 'js/ProgressiveCat.js' 'js/Tile.js' 'js/TileBuffer.js' 'js/ColorMap.js' 'js/HpxKey.js' 'js/HpxImageSurvey.js' 'js/HealpixGrid.js' 'js/Location.js' 'js/View.js' 'js/Aladin.js')

cmd="cat "
for t in "${jsfiles[@]}"
do
    cmd="${cmd} ${srcdir}/$t"
done


# version non minifiée
cmd1="${cmd} | sed -e 's/{ALADIN-LITE-VERSION-NUMBER}/${version}/g' > ${distribfile}"
eval ${cmd1}

# version minifiée
fileList=""
for t in "${jsfiles[@]}"
do
    fileList="${fileList} ${srcdir}/$t"
done
cmd2="${uglifyjs} ${fileList} --comments -c -m |  sed -e 's/{ALADIN-LITE-VERSION-NUMBER}/${version}/g' > ${distribfileminified}"
echo $cmd2
eval ${cmd2}

# traitement des CSS
${lessc} --compress ${csssrcfile} > ${cssfileminified}

cp ${csssrcfile} ${distribdir}

# create AladinLiteSrc.tar.gz
myVersion="AladinLite-${version}"
echo $myVersion
rm -r ${srctmpdir}
mkdir ${srctmpdir}
mkdir ${srctmpdir}/${myVersion}
mkdir ${srctmpdir}/${myVersion}/src
mkdir ${srctmpdir}/${myVersion}/src/js
mkdir ${srctmpdir}/${myVersion}/src/css
for t in "${jsfiles[@]}"
do
    cp ${srcdir}/$t ${srctmpdir}/${myVersion}/src/js
done
cp ${csssrcfile} ${srctmpdir}/${myVersion}/src/css
cp minimal_build.sh ${srctmpdir}/${myVersion}/build.sh
cp COPYING ${srctmpdir}/${myVersion}
tar cvf ${srctar} -C ${srctmpdir} .
gzip -f ${srctar}

# update symbolic link pointing to latest release
latest_symlink=${scriptdir}/../distrib/latest
if [ -d ${latest_symlink} ];
then
    rm ${latest_symlink}
fi

ln -s ${distribdir} ${latest_symlink}
