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
      "value": 88.0,
      "lower_value": 87.42,
      "upper_value": 88.88
    }
  }
}

print(json.dumps(bencher_json))




