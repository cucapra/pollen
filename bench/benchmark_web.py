import sys
import os
import json
import subprocess
from pathlib import Path
import time
import tomllib
import gzip
import shutil

# Parse the GFA URLs from graphs.toml
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

# Download a GFA file from the internet
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

# Run a single test      
def test(command, test_file_name, num_iter):
  if command == "extract":
    with open(os.devnull, "w") as devnull:
      start_time = time.time()
      for _ in range(num_iter):
        subprocess.run(["fgfa", "-I", test_file_name, "extract", "-n", "3", "-c", "3"], stdout=devnull,
            stderr=devnull,
            check=True) 
      end_time = time.time()
      return ((end_time - start_time) * 1000) / num_iter
        
  elif command == "chop":
    with open(os.devnull, "w") as devnull:
      start_time = time.time()
      for _ in range(num_iter):
        subprocess.run(["fgfa", "-I", test_file_name, "chop", "-c", "3", "-l"], stdout=devnull,
            stderr=devnull,
            check=True) 
      end_time = time.time()
      return ((end_time - start_time) * 1000) / num_iter

  elif command == "depth":
    with open(os.devnull, "w") as devnull:
      start_time = time.time()
      for _ in range(num_iter):
        subprocess.run(["fgfa", "-I", test_file_name, "depth"], stdout=devnull,
            stderr=devnull,
            check=True) 
      end_time = time.time()
      return ((end_time - start_time) * 1000) / num_iter
  return 0.0
  
# Run the latency benchmark across all test files
def benchmark(test_config):
  del_cond = ""
  norm_cond = ""
  num_iter = 0
  iter_count = -1
  
  # Read command-line arguments
  if len(sys.argv) >= 3:
    iter_count = int(sys.argv[2]) # Can be any integer
  
  if len(sys.argv) >= 4:
    del_cond = sys.argv[3] # Can be "del", "_", or not provided

  if len(sys.argv) >= 5:
    norm_cond = sys.argv[4] # Can be "norm", or not provided

  # Choose test file set
  test_files = []
  if "smoke" in test_config:
    test_files = smoke_files
    num_iter = 2
  elif "mini" in test_config:
    test_files = mini_files
    num_iter = 10
  elif "med" in test_config:
    test_files = med_files
    num_iter = 5
  elif "big" in test_config:
    test_files = big_files
    num_iter = 2
  else:
    raise ValueError("Incorrect test config provided")
  
  # Set number of test iterations
  if not iter_count == -1:
   num_iter = iter_count
  
  i = 0
  total_time = 0.0
  extract_time = 0.0
  chop_time = 0.0
  depth_time = 0.0
  size_bytes_avg = 0

  # Run a test for each file in the set
  for file in test_files:
    test_file_name = f"tests/{test_config}_{i}.gfa"
    download_file(test_file_name, file)
    subprocess.run(["fgfa", "-I", test_file_name, "-o", results],
                  check = True) 
    size_bytes_avg += os.path.getsize(results)
    extract_time += test("extract", test_file_name, num_iter)
    chop_time += test("chop", test_file_name, num_iter)
    depth_time += test("depth", test_file_name, num_iter)
    subprocess.run(["rm", "-rf", results], check = True) 

    # Delete test files if flag set
    if del_cond == "del":
      subprocess.run(["rm", "-rf", test_file_name], check = True) 
    i += 1
  if (norm_cond == "norm"):

    # Write new normalization values
    with open("bench/normalization.toml", "w") as f:
      f.write("[normalization_factors]\n")
      f.write(f"extract = {extract_time}\n")
      f.write(f"chop = {chop_time}\n")
      f.write(f"depth = {depth_time}\n")
    return (1.0, size_bytes_avg)
  else:

    # Read normalization values
    with open("bench/normalization.toml", "rb") as f:
      data = tomllib.load(f)
    extract_norm = data["normalization_factors"]["extract"]
    chop_norm = data["normalization_factors"]["chop"]
    depth_norm = data["normalization_factors"]["depth"]

    # Normalize values
    extract_time /= extract_norm
    chop_time /= chop_norm
    depth_time /= depth_norm

    # Return the harmonic mean
    size_bytes_avg /= len(test_files)
    return (3 / ((1 / extract_time) + (1 / chop_time) + (1 / depth_time)), size_bytes_avg / 1000.0)

# Read the desired test file set from command-line input
test_config = ""
if len(sys.argv) >= 2:
  test_config = sys.argv[1] # Can be either "smoke", "mini", "med", or "big"
else:
  raise ValueError("No arguments provided")

bench_results = benchmark(test_config)
          

# Output the benchmark results, either in a Bencher JSON format, or a standard 
# command-line format
if "bencher" in test_config:
  bencher_json = {
    "FlatGFA Benchmark Results": {
      "Average Execution Latency": {"value": round(bench_results[0], 2)}, 
      "Average File Size": {"value": round(bench_results[1], 2)},
    }
  }
  json.dump(bencher_json, sys.stdout)
else:

  # Only print latency info if flag set
  if "latency" in test_config:
    print(f"Average Execution Latency: {round(bench_results[0], 2)} ms")

  # Only print filesize info if flag set
  elif "filesize" in test_config:
    print(f"Average File Size: {round(bench_results[1], 2)} KB")
  else:
    print(f"Average Execution Latency: {round(bench_results[0], 2)} ms")
    print(f"Average File Size: {round(bench_results[1], 2)} KB")
  

# Command format: python bench/latency_benchmark_web.py [size](_bencher/_latency/_filesize) [run_count] (del/_) (norm)
# () = optional, [] = replace with value  