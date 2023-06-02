# Pollen language walkthrough

## Core types in Pollen

        type Graph = {
          segments: Set(Segment);
          paths: Set(Path);
          links: Set(Links); 
        }

        type Segment = {
          sequence: Strand; //ACTGAC, etc. 
          links: Set(Link); //links encode their direction + orientation
          steps: Set(Step); //steps that go through segment
        }

        type Handle = {
          segment: Segment; 
          orientation: bool;
        }

        type Path = {
          steps: List(Step);
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

    for segment in Segments {
      seq = Strand();
      in_n = false;
      for c in segment.sequence {
        if (c == 'N') {
          if (!in_n) {
            in_n = true;
            seq.push(c);
          }
        } else {
            in_n = false;
            seq.push(c);
        }
      }
      emit Segment {
        sequence: seq
        ..segment
      };
    }

## Degree

    for segment in Segments {
      emit (segment, segment.edges.size());
    }

## Depth

    for segment in Segments {
      emit (segment, segment.steps.size());
    }

## Flatten

## Flip

    local is_rev(Path path) {
      fw = 0;
      bw = 0;
      for step.handle as sh in path.steps {
        len = sh.segment.sequence.length();
        if (sh.orientation) {
          fw += len;
        }
        else {
          bw += len;
        }
      }
      return bw > fw;
    }

    local flip_path(Path path) {
      if (is_rev(path)) {
        for step in rev(path.steps) {
          emit Step {
            handle: {
              orientation: !step.handle.orientation
              ..step.handle
            }
            ..step
          };
        }
      }
      emit Path {
        steps: path_steps
      };
    }

    for path in Paths {
      flip_path(path);
    }

## Inject

## Matrix

## Overlap

    for path in Paths {
      for step in path.steps {
          for s in step.handle.segment.steps {
            if s.path != path {
              emit (path, s.path);
            }
        }
      }
    }

## Paths

Since Pollen doesn't explicitly name paths, we print the start and end steps of each path.

    for path in Paths {
      emit (path.steps[0], path.steps[path.steps.size()])
    }

## Validate
