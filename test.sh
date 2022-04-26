#!/bin/bash

Help()
{
	echo "Syntax: ./test.sh [-f filename] [-i]"
	echo "options:"
	echo "f	    Insert argument for file name (gfa)"
	echo "i     Creates folder for intermediate files called test_files, instead of deleting them"
	echo "h     Displays Help"
}

while getopts 'f:ih' OPTION; do
	case "$OPTION" in
		f)
			odgi build -g $OPTARG -o temp.og
			odgi depth -i temp.og -d > temp_depth.txt

			python3 process.py temp_depth.txt

			python3 depth1.py $OPTARG
			diff odgi_output.txt python_output.txt > diff_output.txt

			if [ -s diff_output.txt ]; then
        			echo -e "\x1b[31mTest failed! Differences in diff_output.txt\x1b[0m"
			else
        			echo -e "\x1b[32mTest passed! No differences.\x1b[0m"
			fi

			;;
		i)
			mkdir test_files
			mv temp* test_files 
			;;
		h)
			Help
			exit 1
			;;
	esac
done


rm -rf temp*
