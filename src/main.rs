#[derive(Debug, Clone)]
pub struct AlignCell {
    from: (isize, isize),
    score: isize,
}

pub struct Aligner<'a> {
    matrix: Vec<Vec<AlignCell>>,
    a: Vec<u8>,
    b: Vec<u8>,
    scorer: &'a dyn AlignScorer
}

pub trait AlignScorer {
    /// An AlignScorer is a trait needed to perform S-W
    ///
    /// # Arguments
    /// # a - a char representing... suca² suca³ suca⁷
    /// * `self` - the object with trait CanScoreSequence.
    fn get_score(&self, a: u8, b: u8) -> isize;

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
    fn get_score(&self, a: u8, b: u8) -> isize {
        if a == b {
            self.mmatch
        } else {
            self.mismatch
        }
    }

    fn get_gap_score(&self, len: usize) -> isize {
        self.opengap + self.gap * ((len as isize) - 1)
    }
}

impl Aligner<'_> {
    pub fn new<'a> (a: &str, b: &str, scorer: & 'a dyn AlignScorer) -> Aligner<'a> {
        let matrix = vec![vec![AlignCell { from: (-1, -1), score: 0 }; a.len() + 1]; b.len() + 1];
        Aligner { scorer: scorer, a: a.as_bytes().to_owned(), b: b.as_bytes().to_owned(), matrix: matrix }
    }

    pub fn generate_scores(&mut self) -> AlignCell {
        for i in 1..self.matrix.len() {
            println!("{:?}", i);
            for j in 1..self.matrix[i].len() {
                let diag_ancestor = self.matrix[i-1][j-1].score;
                let gap_up: AlignCell = self.gap_up(i, j);
                let gap_left: AlignCell = self.gap_left(i, j);
                if diag_ancestor >= gap_up.score && diag_ancestor >= gap_left.score {
                    self.matrix[i][j] = AlignCell { from: ((i-1) as isize, (j-1) as isize), 
                        score: diag_ancestor + self.scorer.get_score(self.a[j-1], self.b[i-1]) };
                } else if gap_up.score >= gap_left.score {
                    self.matrix[i][j] = gap_up;
                } else {
                    self.matrix[i][j] = gap_left;
                }
            }
        }
        AlignCell { from: (-1, -1), score: 0 }
    }

    fn gap_up(&self, i: usize, j: usize) -> AlignCell {
        // We will implement this before Christmas!
        AlignCell { from: (-1, -1), score: 0 }
    }
    
    fn gap_left(&self, i: usize, j: usize) -> AlignCell {
        AlignCell { from: (-1, -1), score: 0 }
    }

    pub fn traceback(&self, max: &AlignCell) -> String {
        "TODO".to_owned()
    }
}


fn main() {
    let scorer = BaseScorer::new();
    let mut align = Aligner::new("GATTACA", "TACG", &scorer);
    let max = align.generate_scores();
    let alignment = align.traceback(&max);
    println!("{:?}", alignment);
}
