import sys
import time
import os
import json
import subprocess


subprocess.run(["cargo", "build", "--release"])

start_time = 0.0
end_time = 0.0
avg_time = 0.0
 
for i in range(10):

  start_time = time.time()

  subprocess.run(["fgfa", "-I", "tests/DRB1-3123.gfa", "extract", "-n", "3", "-c", "3"]) 

  end_time = time.time()

  avg_time += (end_time - start_time) * 1000 # ms


avg_time /= 10


bencher_json = {
  "FlatGFA Extract Latency": {
    "latency": {
      "value": avg_time
    }
  }
}

print(json.dumps(bencher_json))




