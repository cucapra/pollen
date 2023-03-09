#!/bin/sh
for fn in DRB1-3123 LPA k note5 overlap q.chop t
do
  odgi flip -i../../../test/$fn.og --out=flipped.og
  odgi view -i./flipped.og -g > temp.out
  python3 ../../mygfa.py < temp.out > $fn.out # normalizing to H/S/P/L
  rm flipped.og temp.out
done