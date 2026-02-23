import sys
import os
import json
import subprocess
import tomllib

def benchmark(test_file):
  subprocess.run(["fgfa", "-I", test_file, "-o", "filesize_benchmark.txt"], 
                 check = True) 
  size_bytes = os.path.getsize("filesize_benchmark.txt")
  subprocess.run(["rm", "-rf", "filesize_benchmark.txt"], check = True) 
  return size_bytes

gfa_files = ["tests/chr6.C4.gfa", "tests/DRB1-3123.gfa", "tests/LPA.gfa"]
sizes = {name: float(benchmark(name)) / 1000.0 for name in gfa_files}
size_bytes_avg = (sizes["tests/chr6.C4.gfa"] + sizes["tests/DRB1-3123.gfa"] +
                   sizes["tests/DRB1-3123.gfa"]) / 3.0

bencher_json = {
  "FlatGFA File Size": {
    "chr6.C4 (File Size)": {"value": round(sizes["tests/chr6.C4.gfa"], 2)}, 
    "DRB1-3123 (File Size)": {"value": round(sizes["tests/DRB1-3123.gfa"], 2)}, 
    "LPA (File Size)": {"value": round(sizes["tests/DRB1-3123.gfa"], 2)}, 
    "Average (File Size)": {"value": round(size_bytes_avg, 2)}
  }
}

json.dump(bencher_json, sys.stdout)




