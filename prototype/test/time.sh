files=(*.og)
nfiles=${#files[@]}
for ((i = 0; i < nfiles; i += 1)); do
    # Do something with ${files[$i]}
    file=${files[$i]}
    echo $file
    python3 ../parse_data.py $file -a -o temp.data
    echo "Baseline:"
    python3 ../calyx_depth_simple.py -a $file > temp.futil
    fud exec temp.futil --to dat --through verilog -s verilog.data temp.data -pr
    echo "Short circuit:"
    python3 ../calyx_depth_simple_short_circuit_loops.py -a $file > temp.futil
    fud exec temp.futil --to dat --through verilog -s verilog.data temp.data -pr
    echo "Short circuit 2:"
    python3 ../parse_data_scl2.py $file -a -o temp.data
    python3 ../calyx_depth_simple_short_circuit_loops2.py -a $file > temp.futil
    fud exec temp.futil --to dat --through verilog -s verilog.data temp.data -pr
done
rm temp.*
