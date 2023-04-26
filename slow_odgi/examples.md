# Pollen examples

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
