# Pollen language walkthrough

## Core types in Pollen

    type Graph = {
      segments: Set<Segment>;
      paths: Set<Path>;
      links: Set<Links>; 
    }

    type Segment = {
      sequence: Strand; //ACTGAC, etc. 
      links: Set<Link>; //links encode their direction + orientation
      steps: Set<Step>; //steps that go through segment
    }

    type Handle = {
      segment: Segment; 
      orientation: bool;
    }

    type Path = {
      steps: List<Step>;
    }

    type Step = {
      path: Path; 
      idx: int; //where in the path is this sequence?
      handle: Handle; //segment + orientation
    }

    type Base = A | C | T | G | N
    type Strand = list Base

    type Link = {
      start: Handle; 
      end: Handle;
    }

## Chop

## Crush

    graph out_g;
    parset out_segs[Segment, out_g];

    for segment in Segments {
      seq = Strand();
      in_n = false;
      for c in segment.sequence {
        if c == 'N' {
          if !in_n {
            in_n = true;
            seq.push(c);
          }
        } else {
            in_n = false;
            seq.push(c);
        }
      }
      emit { segment with sequence: seq } to out_segs;
    }

## Degree

    // Output is a (Segment, int) array.
    outset out_degs[Segment, int];

    for segment in in_g.segments {
      emit (segment, segment.edges.size()) to out_degs;
    }

## Depth

    // Output is a (Segment, int) array.
    outset out_depths[Segment, int];

    for segment in in_g.segments {
      emit (segment, segment.steps.size()) to out_depths;
    }

## Flatten

## Flip

    graph out_g;
    parset out_steps[Step, out_g];
    parset out_paths[Path, out_g];

    local is_rev(Path path) {
      fw = 0;
      bw = 0;
      for step.handle as sh in path.steps {
        len = sh.segment.sequence.length();
        if sh.orientation {
          fw += len;
        }
        else {
          bw += len;
        }
      }
      return bw > fw;
    }

    local flip_path(Path path) {
      if is_rev(path) {
        for step in rev(path.steps) {
          emit { 
            step with handle: 
              { step.handle with
                orientation = !step.handle.orientation
              }
          } to out_steps;
        }
      }
      emit { path with steps: path_steps } to out_paths;
    }

    for path in in_g.paths {
      flip_path(path);
    }

## Inject

## Matrix

## Overlap

    // Output is a (Path, Path) array.
    outset out_overlaps[Path, Path];

    for path in in_g.paths {
      for step in path.steps {
          for s in step.handle.segment.steps {
            if s.path != path {
              emit (path, s.path) to out_overlaps;
            }
        }
      }
    }

## Paths

Since Pollen doesn't explicitly have path identifiers, we print the start and end steps of each path (akin to `odgi paths --idx = graph.og --list-paths --list-paths-start-end`). See [here][paths] for more options for the `paths` command.

    // Output is a (Step, Step) array.
    outset out_paths[Step, Step];

    for path in Paths {
      emit (path.steps[0], 
            path.steps[path.steps.size() - 1]) 
        to out_paths;
    }

## Validate

[paths]: https://odgi.readthedocs.io/en/latest/rst/commands/odgi_paths.html
