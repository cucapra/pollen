import sys
import os
import json
import subprocess

subprocess.run(["cargo", "build", "--release"])
subprocess.run(["fgfa", "-I", "tests/DRB1-3123.gfa","-o", "filesize_benchmark.txt"]) 


size_bytes = os.path.getsize("filesize_benchmark.txt")

bencher_json = {
  "benchmark_name": {
    "latency": {
      "value": size_bytes ,
      "lower_value": size_bytes ,
      "upper_value": size_bytes 
    }
  }
}

print(json.dumps(bencher_json))




