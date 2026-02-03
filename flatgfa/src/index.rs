use std::ops::Range;

use crate::flatgfa::{FlatGFA, Path, Segment};
use crate::pool::{Id, Span};

/// A reference to a specific Step within a Path
pub struct StepRef {
    /// The ID of the Path
    path: Id<Path>,

    /// The offset of the Step within that Path's Span of Steps
    offset: u32,
}

/// An in memory index that provides all Steps over a given Segment
pub struct StepsBySegIndex {
    /// All Steps that are accessible by the index
    /// StepRefs that are cross over the same segment are contiguous
    steps: Vec<StepRef>,

    /// Spans into the steps vector. All Steps in the Span cross over one Segment
    /// A Span's index maps to a Segment's index in a corresponding FlatGFA
    segment_steps: Vec<Span<StepRef>>,
}

// TODO: make this return an Id<Segment> instead of a usize
/// Extract the segment index from a StepRef in a given FlatGFA
fn segment_of_step(fgfa: &FlatGFA, step: &StepRef) -> usize {
    let path = &fgfa.paths[step.path];
    let step_span = path.steps;
    let step_slice = &fgfa.steps[step_span];
    step_slice[step.offset as usize].segment().index()
}

impl StepsBySegIndex {
    pub fn new(fgfa: &FlatGFA) -> Self {
        // will be our `steps` vector that contains all steprefs
        let mut all_steps = Vec::new();

        // build the vector of all the steprefs, by iterating over each path, so we can construct using a path id and offset
        for (path_id, path) in fgfa.paths.items() {
            for (offset, _) in fgfa.get_path_steps(path).enumerate() {
                all_steps.push(StepRef {
                    path: path_id,
                    offset: offset as u32,
                });
            }
        }

        // sort the steprefs by the index of the segment in the segment pool
        all_steps.sort_by_key(|a| segment_of_step(fgfa, a));

        // once sorted, steps that cross the same segment will be contiguous in all_steps
        // this allows us to generate spans over steps that cross over the same segment,
        // storing them in segment_steps

        // the working segment's index
        let mut seg_ind: usize = segment_of_step(fgfa, &all_steps[0]);

        // the working start of the span of StepRefs
        let mut span_start: usize = 0;

        // the vector of spans of step refs that will act as our index
        let mut segment_steps: Vec<Span<StepRef>> = Vec::new();

        // enumerate over each step ref in all_steps to build our spans
        for (i, step_ref) in all_steps.iter().enumerate() {
            let curr_seg_ind: usize = segment_of_step(fgfa, step_ref);

            // if we have moved onto a new segment add to the segment_steps vec
            if curr_seg_ind > seg_ind {
                // create and push the new span of step refs for this segment
                // it will be left inclusive right exclusive
                let new_seg: Span<StepRef> = Span::new(Id::new(span_start), Id::new(i));
                segment_steps.push(new_seg);

                // update the working seg_ind and range_start variables to reflect the new segment/range
                seg_ind = curr_seg_ind;
                span_start = i;
            }
        }

        // because we've only pushed a new span whenever we move onto a new segment
        // we will need to push the final span after iterating
        let new_seg = Span::new(Id::new(span_start), Id::new(all_steps.len()));
        segment_steps.push(new_seg);

        Self {
            steps: all_steps,
            segment_steps,
        }
    }

    /// Returns a slice of StepRefs that cross over the given segment
    pub fn get_steps_slice(&self, segment: Id<Segment>) -> &[StepRef] {
        let range: Range<usize> = Range::from(&self.segment_steps[segment.index()]);

        &(self.steps[range])
    }

    /// Returns the number of steps that cross over this segment
    pub fn get_num_steps(&self, segment: Id<Segment>) -> usize {
        let vec_length = self.segment_steps.len();

        println!("segment_steps length = {}", vec_length);
        self.segment_steps[segment.index()].len()
    }
}
