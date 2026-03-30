use std::ops::Range;

use crate::flatgfa::{FlatGFA, Path, Segment};
use crate::pool::{Id, Span};

/// A reference to a specific Step within a Path
pub struct StepRef {
    /// The ID of the Path
    pub path: Id<Path>,

    /// The offset of the Step within that Path's Span of Steps
    pub offset: u32,
}

/// An in memory index that provides all Steps over a given Segment
pub struct StepsBySegIndex {
    /// All Steps that are accessible by the index.
    /// StepRefs that are cross over the same segment are contiguous
    pub steps: Vec<StepRef>,

    /// Spans into the steps vector. All Steps in the Span cross over one Segment.
    /// A Span's index maps to a Segment's index in a corresponding FlatGFA
    pub segment_steps: Vec<Span<StepRef>>,
}

// TODO: make this return an Id<Segment> instead of a usize
/// Extract the segment index from a StepRef in a given FlatGFA
fn segment_of_step(fgfa: &FlatGFA, step: &StepRef) -> Id<Segment> {
    let path = &fgfa.paths[step.path];
    let step_span = path.steps;
    let step_slice = &fgfa.steps[step_span];
    step_slice[step.offset as usize].segment()
}

impl StepsBySegIndex {
    pub fn new(fgfa: &FlatGFA) -> Self {
        // will be our `steps` vector that contains all steprefs
        let mut all_steps = Vec::new();

        // build the vector of all the steprefs, by iterating over each path, so we can construct using a path id and offset
        // package the Id<Segment> with the StepRef's into tuples so sorting by it is easier
        for (path_id, path) in fgfa.paths.items() {
            for (offset, _) in fgfa.get_path_steps(path).enumerate() {
                // the constructed StepRef based on the path and step offset
                let step = StepRef {
                    path: path_id,
                    offset: offset as u32,
                };

                // the segment that this step passes over in the form of a Id<Segment>
                let seg = segment_of_step(fgfa, &step);

                // push the tuple of the Id<Segment> and StepRef into the steps vector
                all_steps.push((seg, step));
            }
        }

        // sort the steprefs by the index of the segment in the segment pool
        // by extracting the actual numeric index from the Id<Segment>
        all_steps.sort_by_key(|a| a.0.index());

        // once sorted, steps that cross the same segment will be contiguous in all_steps
        // this allows us to generate spans over steps that cross over the same segment,
        // storing them in segment_steps

        // the working segment's index
        let mut seg_ind: &Id<Segment> = &all_steps[0].0;

        // the working start of the span of StepRefs
        let mut span_start: usize = 0;

        // the vector of spans of step refs that will act as our index
        let mut segment_steps: Vec<Span<StepRef>> = Vec::new();

        // fill the segment_steps with empty spans
        // TODO: we definitely don't need to do another iteration to fill this with empty spans
        // It's likely more efficient to push empty spans as needed
        for _ in 0..fgfa.segs.len() {
            segment_steps.push(Span::new_empty());
        }

        // enumerate over each step ref in all_steps to build our spans
        for (i, (curr_seg_ind, _)) in all_steps.iter().enumerate() {
            // if we have moved onto a new segment add to the segment_steps vec
            if curr_seg_ind.index() > seg_ind.index() {
                // create and set the new span of step refs for this segment
                // it will be left inclusive right exclusive
                let new_span: Span<StepRef> = Span::new(Id::new(span_start), Id::new(i));

                // assign the span to the index in segment_steps that maps to the index of the segment in the FlatGFA segment pool
                segment_steps[seg_ind.index()] = new_span;

                // update the working seg_ind and range_start variables to reflect the new segment/range
                seg_ind = curr_seg_ind;
                span_start = i;
            }
        }

        // because we've only set a new span whenever we move onto a new segment
        // we will need to set the final span after iterating
        let new_span: Span<_> = Span::new(Id::new(span_start), Id::new(all_steps.len()));
        segment_steps[seg_ind.index()] = new_span;

        // unpackage all_steps into a vector with just spans
        let steps: Vec<StepRef> = all_steps.into_iter().map(|x| x.1).collect();

        Self {
            steps,
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
        self.segment_steps[segment.index()].len()
    }
}
