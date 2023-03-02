#!/bin/sh
for fn in DRB1-3123 LPA k note5 overlap q.chop t
do
  odgi crush -i../../../test/$fn.og --out=crushed.og
  odgi view -i./crushed.og -g > $fntemp.out
  python3 ../../mygfa.py < $fntemp.out > $fn.out # normalizing to H/S/P/L
  rm crushed.og $fntemp.out
done