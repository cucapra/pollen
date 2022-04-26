import sys

with open(sys.argv[1], 'r') as f: 
    contents = f.readlines()
    data = []
    for line in contents:
        data.append(line.split())
    
    for i in range(1, len(data)):
        print(str(data[i][0].strip()) + " " + str(data[i][1].strip()), end='\n')
