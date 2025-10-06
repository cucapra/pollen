import subprocess
import sys
import re
import tempfile
import json
from bench.bench import hyperfine, HyperfineResult


def main():
  with open("performance-output.txt", "w") as f:
    main_runtime = 0.0
    for i, arg in enumerate(sys.argv, start = 0):
      if i == 0:
         arg = "main"
      f.write(f"Testing conversion to GFA: {arg}\n")
      subprocess.run(["git", "switch", "-q", arg])
      results = hyperfine(["fgfa -i DRB1-3123.flatgfa | less "])
      runtime = results[0].mean
      f.write(f"Runtime: {runtime} ms\n\n")
      if i == 0:
        main_runtime = runtime
      if not i == 0:
        diff = runtime / main_runtime
        f.write(f"{arg} ran for {round(diff, 3)} times as long as main")
      
if __name__ == "__main__":
    main()