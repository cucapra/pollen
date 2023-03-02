#!/bin/sh
for fn in DRB1-3123 LPA k note5 overlap q.chop t
do
  odgi degree -d --input=../../../test/$fn.og > $fn.out
done