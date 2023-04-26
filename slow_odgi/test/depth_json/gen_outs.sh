#!/bin/sh
for fn in t k note5 overlap
# q.chop
# Large enough that they need bespoke CLI-passed maxes
# LPA DRB1-3123 chr6.C4
do
  odgi build -g $fn.gfa -o $fn.og
  exine depth -d $fn.og -o $fn.out -n 20
  echo "" >> $fn.out # just to add a newline at EoF
  echo "null" >> $fn.out # Matching the mysterious null at the end of the JSON
done
