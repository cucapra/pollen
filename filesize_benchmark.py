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

size_bytes_1 = benchmark("tests/chr6.C4.gfa")
size_bytes_2 = benchmark("tests/DRB1-3123.gfa")
size_bytes_3 = benchmark("tests/LPA.gfa")
size_bytes_avg = (size_bytes_1 + size_bytes_2 + size_bytes_3) / 3

bencher_json = {
  "FlatGFA File Size": {
    "chr6.C4 (File Size)": {"value": size_bytes_1}, 
    "DRB1-3123 (File Size)": {"value": size_bytes_2}, 
    "LPA (File Size)": {"value": size_bytes_3}, 
    "Average (File Size)": {"value": size_bytes_avg}
  }
}

print(json.dumps(bencher_json))




