import sys
import os
import json
import subprocess

subprocess.run(["cargo", "build", "--release"], check = True)

def benchmark(test_file):
  subprocess.run(["fgfa", "-I", test_file, "-o", "filesize_benchmark.txt"], check = True) 
  size_bytes = os.path.getsize("filesize_benchmark.txt")
  subprocess.run(["rm", "-rf", "filesize_benchmark.txt"], check = True) 
  return size_bytes

size_bytes_1 = float(benchmark("tests/chr6.C4.gfa")) / 1000.0
size_bytes_2 = float(benchmark("tests/DRB1-3123.gfa")) / 1000.0
size_bytes_3 = float(benchmark("tests/LPA.gfa")) / 1000.0
size_bytes_avg = (size_bytes_1 + size_bytes_2 + size_bytes_3) / 3.0

bencher_json = {
  "FlatGFA File Size": {
    "chr6.C4 (File Size)": {"value": round(size_bytes_1, 2)}, 
    "DRB1-3123 (File Size)": {"value": round(size_bytes_2, 2)}, 
    "LPA (File Size)": {"value": round(size_bytes_3, 2)}, 
    "Average (File Size)": {"value": round(size_bytes_avg, 2)}
  }
}

print(json.dumps(bencher_json))




