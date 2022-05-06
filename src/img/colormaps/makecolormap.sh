#!/bin/bash
rm -rf colormaps.png

for colormap_filename in `ls *.png | sort -V`; do
	convert "$colormap_filename" -crop 256x1+0+0 +repage "$colormap_filename"s
done

convert -append *.pngs colormaps.png

rm -rf *.pngs
