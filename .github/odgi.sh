#!/bin/sh
docker run --rm -v `pwd`:/work --workdir /work odgi odgi $@
