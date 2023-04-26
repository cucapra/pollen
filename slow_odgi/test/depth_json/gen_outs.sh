#!/bin/sh
for fn in t k note5 overlap
# Large enough that they need bespoke CLI-passed maxes
# q.chop LPA DRB1-3123 chr6.C4
do
  odgi build -g $fn.gfa -o $fn.og
  exine depth -d $fn.og -o $fn.out
  echo "" >> $fn.out # just to add a newline at EoF
done
