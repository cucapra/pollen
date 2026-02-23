import sys
import os
import json
import subprocess
from pathlib import Path
import tomllib
import gzip
import shutil

with open("bench/graphs.toml", "rb") as f:
    toml_graphs = tomllib.load(f)

hprc_dict = dict(toml_graphs["hprc"])
 
test_dict = dict(toml_graphs["test"]) 

gont_dict = dict(toml_graphs["1000gont"])

smoke_files = [test_dict["k"]]
 
mini_files = [test_dict["lpa"], test_dict["chr6c4"], hprc_dict["chrM"]]

med_files = [hprc_dict["chr20"], hprc_dict["chrX"], gont_dict["chr16"]]
 
big_files = [hprc_dict["chrY"], hprc_dict["chr1"], hprc_dict["chr10"]]

results = "filesize_benchmark.txt"

def download_file(target_name, web_file):
  gzipped = False
  temp_name = ""
  if "gfa.gz" in web_file:
    gzipped = True
  if gzipped:
    temp_name = f"{target_name}.gz"

  if not Path(target_name).exists():
    if gzipped:
      subprocess.run(["curl", "-o", temp_name, web_file],
              check = True) 
      with gzip.open(temp_name, "rb") as f_in:
        with open(target_name, "wb") as f_out:
          shutil.copyfileobj(f_in, f_out)
      subprocess.run(["rm", "-rf", temp_name], check = True) 
    else:
      subprocess.run(["curl", "-o", target_name, web_file],
              check = True) 
  
def benchmark(test_config):
  test_cond = ""
  if len(sys.argv) >= 3:
    test_cond = sys.argv[2] # Can be "del", or not provided

  test_files = []
  if "smoke" in test_config:
    test_files = smoke_files
  elif "mini" in test_config:
    test_files = mini_files
  elif "med" in test_config:
     test_files = med_files
  elif "big" in test_config:
    test_files = big_files
  else:
    raise ValueError("Incorrect test config provided")

  size_bytes_avg = 0
  i = 0
  for file in test_files:
    test_file_name = f"tests/{test_config}_{i}.gfa"
    download_file(test_file_name, file)
    subprocess.run(["fgfa", "-I", test_file_name, "-o", results],
                  check = True) 
    size_bytes = os.path.getsize(results)
    subprocess.run(["rm", "-rf", results], check = True) 
    if test_cond == "del":
      subprocess.run(["rm", "-rf", test_file_name], check = True) 
    size_bytes_avg += size_bytes
    i += 1
  size_bytes_avg /= len(test_files)
  return size_bytes_avg / 1000.0 

test_config = ""
if len(sys.argv) >= 2:
  test_config = sys.argv[1] # Can be either "smoke", "mini", "med", or "big"
else:
  raise ValueError("No arguments provided")



if "bencher" in test_config:
  bencher_json = {
    "FlatGFA File Size Avg": {
      "File": {"value": round(benchmark(test_config), 2)}, 
    }
  }
  json.dump(bencher_json, sys.stdout)
else:
  print(f"File Size Average: {round(benchmark(test_config), 2)} KB")

# Command format: python latency_benchmark_web.py [size](_bencher) (del) 
# () = optional, [] = replace with value
