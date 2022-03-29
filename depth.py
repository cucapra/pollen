import gfapy as gf

gfa = gf.Gfa.from_file("DRB1-3123.gfa")

lines = gfa.lines
edges = gfa.edges
dovetails = gfa.dovetails
containments = gfa.containments
segment_names = gfa.segment_names
path_names = gfa.path_names
edge_names = gfa.edge_names

#print("lines:{l}\n edges:{e}".format(l=lines, e=edges))
#print("segment names:{sg}\n edge names:{en}".format(sg=segment_names, en=edge_names))

#print("edges:{e}".format(e=edges))

pfs = set([tuple(line.positional_fieldnames) for line in lines])
#print("segments, edges, paths", pfs)
print(path_names)


