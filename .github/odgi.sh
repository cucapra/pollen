#!/bin/sh
path=`realpath -s --relative-to=$GITHUB_WORKSPACE ${PWD}`
exec docker run -i --rm -v $GITHUB_WORKSPACE:/work --workdir /work/$path odgi odgi $@
