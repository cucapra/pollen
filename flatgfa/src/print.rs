use crate::flatgfa;
use std::fmt;

impl fmt::Display for flatgfa::Orientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            flatgfa::Orientation::Forward => write!(f, "+"),
            flatgfa::Orientation::Backward => write!(f, "-"),
        }
    }
}

impl fmt::Display for flatgfa::AlignOpcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            flatgfa::AlignOpcode::Match => write!(f, "M"),
            flatgfa::AlignOpcode::Gap => write!(f, "N"),
            flatgfa::AlignOpcode::Insertion => write!(f, "D"),
            flatgfa::AlignOpcode::Deletion => write!(f, "I"),
        }
    }
}

impl<'a> fmt::Display for flatgfa::Alignment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for op in self.ops {
            write!(f, "{}{}", op.len(), op.op())?;
        }
        Ok(())
    }
}

/// A wrapper for displaying components from FlatGFA.
struct Display<'a, T>(&'a flatgfa::FlatGFA<'a>, T);

impl<'a> fmt::Display for Display<'a, flatgfa::Handle> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let seg = self.0.get_handle_seg(self.1);
        let name = seg.name;
        write!(f, "{}{}", name, self.1.orient())
    }
}

impl<'a> fmt::Display for Display<'a, &flatgfa::Path> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "P\t{}\t", self.0.get_path_name(&self.1))?;
        let steps = &self.0.steps[self.1.steps];
        write!(f, "{}", Display(self.0, steps[0]))?;
        for step in steps[1..].iter() {
            write!(f, ",{}", Display(self.0, *step))?;
        }
        write!(f, "\t")?;
        let overlaps = &self.0.overlaps[self.1.overlaps];
        if overlaps.is_empty() {
            write!(f, "*")?;
        } else {
            write!(f, "{}", self.0.get_alignment(overlaps[0]))?;
            for overlap in overlaps[1..].iter() {
                write!(f, ",{}", self.0.get_alignment(*overlap))?;
            }
        }
        writeln!(f)
    }
}

impl<'a> fmt::Display for Display<'a, &flatgfa::Link> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let from = self.1.from;
        let from_name = self.0.get_handle_seg(from).name;
        let to = self.1.to;
        let to_name = self.0.get_handle_seg(to).name;
        writeln!(
            f,
            "L\t{}\t{}\t{}\t{}\t{}",
            from_name,
            from.orient(),
            to_name,
            to.orient(),
            self.0.get_alignment(self.1.overlap)
        )
    }
}

impl<'a> fmt::Display for Display<'a, &flatgfa::Segment> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.1.name;
        write!(f, "S\t{}\t{}", name, self.0.get_seq(self.1))?;
        if !self.1.optional.is_empty() {
            write!(f, "\t{}", self.0.get_optional_data(self.1))?;
        }
        writeln!(f)
    }
}

/// Print a graph in the order preserved from an original GFA file.
fn write_preserved(gfa: &flatgfa::FlatGFA, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut seg_iter = gfa.segs.all().iter();
    let mut path_iter = gfa.paths.all().iter();
    let mut link_iter = gfa.links.all().iter();
    for kind in gfa.get_line_order() {
        match kind {
            flatgfa::LineKind::Header => {
                let version = gfa.header;
                assert!(!version.is_empty());
                writeln!(f, "H\t{}", bstr::BStr::new(version.all()))?;
            }
            flatgfa::LineKind::Segment => {
                let seg = seg_iter.next().expect("too few segments");
                write!(f, "{}", Display(gfa, seg))?;
            }
            flatgfa::LineKind::Path => {
                let path = path_iter.next().expect("too few paths");
                write!(f, "{}", Display(gfa, path))?;
            }
            flatgfa::LineKind::Link => {
                let link = link_iter.next().expect("too few links");
                write!(f, "{}", Display(gfa, link))?;
            }
        }
    }
    Ok(())
}

/// Print a graph in a normalized order, ignoring the original GFA line order.
pub fn write_normalized(gfa: &flatgfa::FlatGFA, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if !gfa.header.is_empty() {
        writeln!(f, "H\t{}", bstr::BStr::new(gfa.header.all()))?;
    }
    for seg in gfa.segs.all().iter() {
        write!(f, "{}", Display(gfa, seg))?;
    }
    for path in gfa.paths.all().iter() {
        write!(f, "{}", Display(gfa, path))?;
    }
    for link in gfa.links.all().iter() {
        write!(f, "{}", Display(gfa, link))?;
    }
    Ok(())
}

/// Print our flat representation as in GFA text format.
impl<'a> fmt::Display for &'a flatgfa::FlatGFA<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line_order.is_empty() {
            write_normalized(self, f)
        } else {
            write_preserved(self, f)
        }
    }
}
