#!/bin/bash

for filename in *.html; do
    open http://localhost:8080/${filename}
done
