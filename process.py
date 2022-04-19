with open('odgi_depth.txt', 'r') as f: 
    contents = f.readlines()
    data = []
    for line in contents:
        data.append(line.split())

with open('odgi_depth_output.txt', 'w') as f2:
    for i in range(1, len(data)):
        f2.write(f'{data[i][0].strip()} {data[i][1].strip()}\n')
