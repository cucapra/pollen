#!/bin/sh
path=`realpath -s --relative-to=$GITHUB_WORKSPACE ${PWD}`
docker run --rm -v $GITHUB_WORKSPACE:/work --workdir /work/$path odgi odgi $@
