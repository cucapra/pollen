# Core types in Pollen

Below is a user reference for key types and data structures in the Pollen language.

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

    enum Base = {
      A,
      C,
      T,
      G,
      N,
    }
    type Strand = List<Base>;

    type Link = {
      start: Handle; 
      end: Handle;
    }
    