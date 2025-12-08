import subprocess
import sys
import re

def main():
  with open("performance-output.txt", "w") as f:
    main_runtime = 0.0
    for i, arg in enumerate(sys.argv, start = 0):
      if i == 0:
         arg = "main"
      f.write(f"Testing compression: {arg}\n")
      command = f"git switch -q {arg} ; hyperfine -w3 -N 'fgfa -I tests/DRB1-3123.gfa extract -n 3 -c 3'"
      result = subprocess.run(command, shell=True, capture_output=True, text=True)
      runtime = 0.0
      runtime_search = re.search(r"Time \(mean ± σ\):\s+([0-9.]+)", result.stdout)
      if runtime_search:
        runtime = float(runtime_search.group(1))
      f.write(f"Runtime: {runtime} ms\n\n")
      if i == 0:
        main_runtime = runtime
      if not i == 0:
        diff = runtime / main_runtime
        f.write(f"{arg} ran for {round(diff, 3)} times as long as main")
      
if __name__ == "__main__":
    main()