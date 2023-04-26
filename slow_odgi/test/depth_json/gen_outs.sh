#!/bin/sh
for fn in k note5 overlap t
    # will grow this list
    # DRB1-3123 LPA k note5 overlap q.chop t
do
  odgi build -g $fn.gfa -o $fn.og
  exine depth -d $fn.og -o $fn.out
  echo "" >> $fn.out # just to add a newline at EoF
done
