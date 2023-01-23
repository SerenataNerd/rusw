#[derive(Debug, Clone)] // TODO signal bug to vscode about this gliph (?)
pub struct AlignCell {
    from: (isize, isize),
    score: isize,
}

pub struct Aligner<'a> {
    matrix: Vec<Vec<AlignCell>>,
    row_seq: Vec<u8>,
    col_seq: Vec<u8>,
    scorer: &'a dyn AlignScorer
}

pub trait AlignScorer {
    /// An AlignScorer is a trait needed to perform S-W
    ///
    /// # Arguments
    /// # a - a char representing... suca² suca³ suca⁷
    /// * `self` - the object with trait CanScoreSequence.
    fn get_score(&self, row_seq: u8, col_seq: u8) -> isize;

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
    pub fn new<'a> (row_seq: &str, col_seq: &str, scorer: & 'a dyn AlignScorer) -> Aligner<'a> {
        let matrix = vec![vec![AlignCell { from: (-1, -1), score: 0 }; col_seq.len() + 1]; row_seq.len() + 1];
        Aligner { scorer: scorer, row_seq: row_seq.as_bytes().to_owned(), col_seq: col_seq.as_bytes().to_owned(), matrix: matrix }
    }

    pub fn generate_scores(&mut self) -> AlignCell {
        let mut max_cell = AlignCell {from: (-1, -1), score: -1};
        for r in 1..self.matrix.len() { 
            println!("{:?}", self.row_seq[r-1] as char); // as Christmas
            for c in 1..self.matrix[r].len() {
                let diag_ancestor = self.matrix[r-1][c-1].score;
                let gap_up: AlignCell = self.gap_up(r, c);
                let gap_left: AlignCell = self.gap_left(r, c);
                let mut this_cell: AlignCell;
                if diag_ancestor >= gap_up.score && diag_ancestor >= gap_left.score {
                    this_cell = AlignCell { from: ((r-1) as isize, (c-1) as isize), 
                        score: diag_ancestor + self.scorer.get_score(self.row_seq[r-1], self.col_seq[c-1]) };
                        // We do not comment our choices regarding rows/cols and r/c because it will
                        // be _the most_ confounding thing here.
                } else if gap_up.score >= gap_left.score {
                    this_cell = gap_up;
                } else {
                    this_cell = gap_left;
                }
                if this_cell.score < 0 {
                    this_cell = AlignCell{score: 0, ..this_cell}
                }
                self.matrix[r][c] = this_cell.clone();
                if self.matrix[r][c].score >= max_cell.score {
                    max_cell = AlignCell{score: this_cell.score, from: (r as isize, c as isize)}; // This is not a from but a here. But...who knows?
                }
            }
        }
        max_cell
    }

    fn gap_up(&self, r: usize, c: usize) -> AlignCell {
        // We will implement this before Christmas!
        let mut max_cell_up = AlignCell {from: (-1, -1), score: isize::MIN};
        //for r_up in (r-1) ..= 1 { // We do not like this but still.
        for r_up in 1 .. r { // This is different from how the algorithm is described going up, but we hate ..=.
            let score = self.matrix[r_up][c].score + self.scorer.get_gap_score(r-r_up);
            if score >= max_cell_up.score {
                max_cell_up = AlignCell {from: (r_up as isize, c as isize), score: score};
            }
        } 
        max_cell_up
    }
    
    fn gap_left(&self, r: usize, c: usize) -> AlignCell {
        // We are implement this after Christmas (...)
        let mut max_cell_left = AlignCell {from: (-1, -1), score: -1};
        for c_left in 1 .. c { // This is different from how the algorithm is described going up, but we hate ..=.
            let score = self.matrix[r][c_left].score + self.scorer.get_gap_score(c-c_left); // ma csrivo comunque più veloce di te anche se scrivo scrivo scrivo scrivo
            if score >= max_cell_left.score {
                max_cell_left = AlignCell {from: (r as isize, c_left as isize), score: score};
            }
        }
        max_cell_left
    }

    pub fn traceback(&self, max: &AlignCell, here: (isize, isize),  row_aligned: &mut String, col_aligned: &mut String) {
        if here.0 == max.from.0 + 1 && here.1 == max.from.1 + 1 { // diagonal
            row_aligned.push(self.row_seq[(here.0-1) as usize] as char);
            col_aligned.push(self.col_seq[(here.1-1) as usize] as char);
       } else if here.0 == max.from.0 { // gap on the column
            let delta = (here.1 - max.from.1) as usize;
            for i in 0 .. delta {
                row_aligned.push(self.row_seq[max.from.1 as usize - i] as char);
                col_aligned.push('-');
            }
            //row_aligned.push_str(&self.row_seq[ max.from.1 as usize .. here.1 as usize].map( |x| {x as char}));
            //(0 .. delta).map( || { col_aligned.push('-') });
       } else {
            let delta = (here.0 - max.from.0) as usize;
            for i in 0 .. delta {
                col_aligned.push(self.col_seq[max.from.0 as usize - i] as char);
                row_aligned.push('-');
            }
       } 
       if self.matrix[max.from.0 as usize][max.from.1 as usize].score != 0 {
            self.traceback(&self.matrix[max.from.0 as usize][max.from.1 as usize], (max.from.0, max.from.1), row_aligned, col_aligned);
       }
    }
}


fn main() {
    let scorer = BaseScorer::new();
    let mut align = Aligner::new("GATTACATAAAAATGGGGGC", "GATACATAAAAAAAATGGGGGC", &scorer);
    let max = align.generate_scores();
    println!("{:?}", max);
    let mut aligned_row = "".to_owned();
    let mut aligned_col = "".to_owned();
    align.traceback(&max, max.from, &mut aligned_row, &mut aligned_col); // the from of the returned max is a here.
    println!("{:?}\n{:?}", aligned_row.chars().rev().collect::<String>(), aligned_col.chars().rev().collect::<String>()); // turbo fish is like  <==> <--00--> 
}