#[derive(Debug, Clone)]
pub struct AlignCell {
    from: (isize, isize),
    score: isize,
}

pub struct Aligner<'a> {
    matrix: Vec<Vec<AlignCell>>,
    scorer: &'a dyn AlignScorer
}

pub trait AlignScorer {
    /// TODO
    ///
    /// # Arguments
    ///
    /// * `self` - the object with trait CanScoreSequence.
    fn get_score(&self, a: char, b: char) -> isize;

    fn get_gap_score(&self, len: usize) -> isize;
}

pub struct BaseScorer {
    mmatch: isize,
    mismatch: isize,
    opengap: isize, // if opengap == gap will fall back to linear
    gap: isize
}

impl BaseScorer {
    fn new() -> BaseScorer {
        BaseScorer { mmatch: 3, mismatch: -2, opengap: -4, gap: -3 }
    }
}

impl AlignScorer for BaseScorer {
    fn get_score(&self, a: char, b: char) -> isize {
        if a == b {
            return self.mmatch
        } else {
            return self.mismatch
        }
    }
    
    fn get_gap_score(&self, len: usize) -> isize {
        return self.opengap + self.gap * ((len as isize) - 1)
    }
}

impl Aligner<'_> {
    pub fn new(a: String, b: String, scorer: &dyn AlignScorer) -> Aligner {
        let matrix = vec![vec![AlignCell { from: (-1, -1), score: 0 }; a.len() + 1]; b.len() + 1];

        // fill with 0 first row and column

        Aligner { scorer: scorer, matrix: matrix }
    }

    pub fn generate_scores(&mut self) -> AlignCell {
        AlignCell { from: (-1, -1), score: 0 }
    }

    pub fn traceback(&self, max: &AlignCell) -> String {
        "TODO".to_owned()
    }
}


fn main() {
    let scorer = BaseScorer::new();
    let mut align = Aligner::new("GATTACA".to_owned(), "TACG".to_owned(), &scorer);
    let max = align.generate_scores();
    let alignment = align.traceback(&max);
    println!("{:?}", alignment);
}
