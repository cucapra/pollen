#!/bin/sh
for fn in DRB1-3123 LPA k note5 overlap q.chop t
do
  python3 slow_odgi/perturb.py < test/$fn.gfa > test/temp.$fn.gfa
done
