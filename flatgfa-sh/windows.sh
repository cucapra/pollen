#!/bin/sh
odgi depth -i ../tests/note5.gfa -r 5 | bedtools makewindows -b /dev/stdin -w 4 > note5.w4.bed
head -n1 note5.w4.bed
rm -f note5.w4.bed
