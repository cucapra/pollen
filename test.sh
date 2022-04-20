#!/bin/bash

odgi build -g $1 -o temp.og
odgi depth -i temp.og -d > temp_depth.txt

python3 process.py temp_depth.txt

python3 depth.py $1
diff odgi_output.txt python_output.txt > diff_output.txt

if [ -s diff_output.txt ]; then
	echo "Test failed! Differences in diff_output.txt"
else
	echo "Test passed! No differences."
fi

rm -rf temp* odgi.og odgi.txt
