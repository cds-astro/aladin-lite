#!/bin/bash

# Lance Chromium avec une page Web permettant de tester la version de aladin dans distrib.latest

cd .. && python3 -m http.server 42195 &

chromium-browser http://0.0.0.0:42195/bug-tracking/test-latest-release.html
