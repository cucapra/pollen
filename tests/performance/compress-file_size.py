import subprocess
import sys

def main():
  with open("performance-output.txt", "w") as f:
    main_file_size = 0
    for i, arg in enumerate(sys.argv, start = 0):
      if i == 0:
         arg = "main"
      f.write(f"Testing compression: {arg}\n")
      command = f"git switch -q {arg} ; cargo build --release ; fgfa -I tests/DRB1-3123.gfa -o blarg ; ls -l blarg | awk '{{print $5}}'"
      result = subprocess.run(command, shell=True, capture_output=True, text=True)
      f.write("File size: " + result.stdout.strip() + " bytes\n\n")
      if i == 0:
        main_file_size = int(result.stdout.strip())
      if not i == 0:
       diff = int(result.stdout.strip()) / main_file_size
       f.write(f"{arg} produced a file {diff} times the size vs main")
      
if __name__ == "__main__":
    main()