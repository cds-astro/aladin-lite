#!/bin/bash

for filename in *.html; do
    google-chrome http://localhost:8080/${filename}
done
