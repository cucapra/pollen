#!/bin/sh
for fn in DRB1-3123 LPA k note5 overlap q.chop t
do
  odgi chop -i../../../test/$fn.og -c3 --out=chopped.og
  odgi view -i./chopped.og -g > temp.out
  python3 ../../mygfa.py < temp.out > $fn.out # normalizing to H/S/P/L
  rm chopped.og temp.out
done