import sys
import time
import os
import json
import subprocess


subprocess.run(["cargo", "build", "--release"], check = True)

def benchmark(test_file, num_iter):
  total_time = 0.0
  for i in range(num_iter):
    start_time = time.time()
    with open(os.devnull, "w") as devnull:
      subprocess.run(["fgfa", "-I", test_file, "extract", "-n", "3", "-c", "3"], stdout=devnull,
          stderr=devnull,
          check=True) 
    end_time = time.time()
    total_time += (end_time - start_time) * 1000
  return total_time / num_iter

avg_time = 0.0

benchmark("tests/DRB1-3123.gfa", 1) # warmup rounds
benchmark("tests/chr6.C4.gfa", 1)
benchmark("tests/LPA.gfa", 1)
time_1 = benchmark("tests/chr6.C4.gfa", 10)
time_2 = benchmark("tests/DRB1-3123.gfa", 10)
time_3 = benchmark("tests/LPA.gfa", 10)
avg_time = (time_1 + time_2 + time_3) / 3


bencher_json = {
  "FlatGFA Extract Latency": {
    "latency": {
      "chr6.C4": {"value": round(time_1, 2)}, 
      "DRB1-3123": {"value": round(time_2, 2)}, 
      "LPA": {"value": round(time_3, 2)}, 
      "Average": {"value": round(avg_time, 2)}
    }
  }
}

print(json.dumps(bencher_json))




