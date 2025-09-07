use palette::{convert::FromColorUnclamped, Okhsv, Srgb};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, StatefulWidget, Widget},
    DefaultTerminal, Frame,
};
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::{thread, time::{self, Duration}};
use String;

#[derive(Debug, Clone)] // TODO signal bug to vscode about this gliph (?)
pub struct AlignCell {
    from: (usize, usize),
    score: i32,
}

pub struct Aligner {
    matrix: Vec<Vec<AlignCell>>,
    row_seq: Vec<u8>,
    col_seq: Vec<u8>,
    row_reached: usize,
    col_reached: usize,
    scorer: Box<dyn AlignScorer>,
    status: (usize, usize),
}

pub trait AlignScorer {
    /// An AlignScorer is a trait needed to perform S-W
    ///
    /// # Arguments
    /// # a - a char representing... suca² suca³ suca⁷
    /// * `self` - the object with trait CanScoreSequence.
    fn get_score(&self, row_seq: u8, col_seq: u8) -> i32;

    fn get_gap_score(&self, len: usize) -> i32;
}

pub struct BaseScorer {
    mmatch: i32,
    mismatch: i32,
    opengap: i32, // if opengap == gap will fall back to linear
    gap: i32
}

impl BaseScorer {
    fn new() -> BaseScorer {
        BaseScorer { mmatch: 3, mismatch: -2, opengap: -4, gap: -3 }
    }
}

impl AlignScorer for BaseScorer {
    fn get_score(&self, a: u8, b: u8) -> i32 {
        if a == b {
            self.mmatch
        } else {
            self.mismatch
        }
    }

    fn get_gap_score(&self, len: usize) -> i32 {
        self.opengap + self.gap * (len as i32) - 1
    }
}

impl Silly {
    pub fn new() -> Silly {
        Silly{}
    }

    pub fn draw(self, frame: &mut Frame, state: &mut Aligner) {
        //frame.render_widget(String::from_utf8(self.row_seq).unwrap(), frame.area());
        // frame.area() is the size of the terminal, should check and die if we do not have space FIXME
        frame.render_stateful_widget(self, Rect::new(0, 0, (state.col_seq.len()+4) as u16, (state.row_seq.len()+4) as u16), state); // was frame.area()
        // +3 cause we want space for the border and the strings
    }
}

impl StatefulWidget for Silly {
    type State = Aligner;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let title = Line::from(" ruSW ".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        block.render(area, buf);
        let (r, c) = state.status;
        let score = state.matrix[r][c].score;
        let mut red = 0;
        for j in 2..state.matrix.len()-1 {
            buf.set_string(j as u16, 1 as u16, (state.col_seq[j-2] as char).to_string(), Style::default().fg(Color::Rgb(0, 0, 255)));
        } 
        for i in 2..(state.matrix[0].len()-1) {
            buf.set_string(1 as u16, i as u16, (state.row_seq[i-2] as char).to_string(), Style::default().fg(Color::Rgb(0, 0, 255)));
        }
        for i in 2..state.matrix[0].len()+2 {
            for j in 2..state.matrix.len()+2 {
                if j == r+2 && i == c+2 {
                    red = ((255 - score as u8)) as u8;
                }
                buf.set_string(i as u16, j as u16, state.matrix[j-2][i-2].score.to_string(), Style::default().fg(Color::Rgb(red, 0, 0)));
                red = 0;
            }
        }
    }
}

// va rifatto tutto perchè serve lo spingitore di cavalieri esterno perchè non è lo stato che decide quando avanza il mondo.
impl Aligner {
    pub fn new<'a> (row_seq: &str, col_seq: &str, scorer:  Box<dyn AlignScorer>) -> Aligner {
        let matrix = vec![vec![AlignCell { from: (0, 0), score: 0 }; col_seq.len() + 1]; row_seq.len() + 1];
        Aligner { scorer: scorer, row_seq: row_seq.as_bytes().to_owned(), col_seq: col_seq.as_bytes().to_owned(), row_reached: 1, col_reached: 1, matrix: matrix, status: (0, 0) }
    }

    pub fn generate_next_score(&mut self) {
        if let Ok(true) = event::poll(Duration::ZERO) {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                if code == KeyCode::Char('q') {
                    return
                }
            }
        }
        let r: usize = self.row_reached;
        let c: usize = self.col_reached;
        self.status = (r, c);
        let diag_ancestor = self.matrix[r-1][c-1].score;
        let gap_up: AlignCell = self.gap_up(r, c);
        let gap_left: AlignCell = self.gap_left(r, c);
        let mut this_cell: AlignCell;
        if diag_ancestor >= gap_up.score && diag_ancestor >= gap_left.score {
            this_cell = AlignCell { from: ((r-1), (c-1)),
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
        if r < self.matrix.len()-1 {
            self.row_reached += 1;
        } else if c < self.matrix.len()-1 {
            self.col_reached += 1;
            self.row_reached = 1
        }
        // increment
        //if self.matrix[r][c].score >= max_cell.score { // max_cell needs to be stored in the aligner
        //   max_cell = AlignCell{score: this_cell.score, from: (r, c)}; // This is not a from but a here. But...who knows?
        //}
    }

    pub fn generate_scores(&mut self) -> Result<AlignCell> {
        let mut max_cell = AlignCell {from: (0, 0), score: -1};
        for r in 1..self.matrix.len() {
            //println!("{:?}", self.row_seq[r-1] as char); // as Christmas
            for c in 1..self.matrix[r].len() {
                if let Ok(true) = event::poll(Duration::ZERO) {
                    if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                        if code == KeyCode::Char('q') {
                            return Ok(max_cell);
                        }
                    }
                }
                self.status = (r, c);
                let diag_ancestor = self.matrix[r-1][c-1].score;
                let gap_up: AlignCell = self.gap_up(r, c);
                let gap_left: AlignCell = self.gap_left(r, c);
                let mut this_cell: AlignCell;
                if diag_ancestor >= gap_up.score && diag_ancestor >= gap_left.score {
                    this_cell = AlignCell { from: ((r-1), (c-1)),
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
                    max_cell = AlignCell{score: this_cell.score, from: (r, c)}; // This is not a from but a here. But...who knows?
                }
                //let _ = terminal.draw(|frame| self.draw(frame)); // Brutalm   ente rimosso ? Perchè uesta roba non ritorna un Result<>`
                self.col_reached += 1;
            }
            self.row_reached += 1;
        }
        Ok(max_cell)
    }

    fn gap_up(&self, r: usize, c: usize) -> AlignCell {
        // We will implement this before Christmas!
        let mut max_cell_up = AlignCell {from: (0, 0), score: i32::MIN};
        //for r_up in (r-1) ..= 1 { // We do not like this but still.
        for r_up in 1 .. r { // This is different from how the algorithm is described going up, but we hate ..=.
            let score = self.matrix[r_up][c].score + self.scorer.get_gap_score(r-r_up);
            if score >= max_cell_up.score {
                max_cell_up = AlignCell {from: (r_up, c), score: score};
            }
        }
        max_cell_up
    }

    fn gap_left(&self, r: usize, c: usize) -> AlignCell {
        // We are implement this after Christmas (...)
        let mut max_cell_left = AlignCell {from: (0, 0), score: -1};
        for c_left in 1 .. c { // This is different from how the algorithm is described going up, but we hate ..=.
            let score = self.matrix[r][c_left].score + self.scorer.get_gap_score(c-c_left); // ma csrivo comunque più veloce di te anche se scrivo scrivo scrivo scrivo
            if score >= max_cell_left.score {
                max_cell_left = AlignCell {from: (r, c_left), score: score};
            }
        }
        max_cell_left
    }

    pub fn traceback(&self, max: &AlignCell, here: (usize, usize),  row_aligned: &mut String, col_aligned: &mut String) {
        if here.0 == max.from.0 + 1 && here.1 == max.from.1 + 1 { // diagonal
            row_aligned.push(self.row_seq[here.0-1] as char);
            col_aligned.push(self.col_seq[here.1-1] as char);
       } else if here.0 == max.from.0 { // gap on the column
            let delta = here.1 - max.from.1;
            for i in 0 .. delta {
                row_aligned.push(self.row_seq[max.from.1 - i] as char);
                col_aligned.push('-');
            }
            //row_aligned.push_str(&self.row_seq[ max.from.1 .. here.1 as usize].map( |x| {x as char}));
            //(0 .. delta).map( || { col_aligned.push('-') });
       } else {
            let delta = here.0 - max.from.0;
            for i in 0 .. delta {
                col_aligned.push(self.col_seq[max.from.0 - i] as char);
                row_aligned.push('-');
            }
       }
       if self.matrix[max.from.0][max.from.1].score != 0 {
            self.traceback(&self.matrix[max.from.0][max.from.1], (max.from.0, max.from.1), row_aligned, col_aligned);
       }
    }
}


fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let result = run(terminal);

    //let max = align.generate_scores().unwrap(); //
    //println!("{:?}", max);
    //let mut aligned_row = "".to_owned();
    //let mut aligned_col = "".to_owned();
    //align.traceback(&max, max.from, &mut aligned_row, &mut aligned_col); // the from of the returned max is a here.
    //println!("{:?}\n{:?}", aligned_row.chars().rev().collect::<String>(), aligned_col.chars().rev().collect::<String>()); // turbo fish is like  <==> <--00-->
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let sil: Silly = Silly::new();
    let scorer = BaseScorer::new();
    let s1_row = "GATTACATAAAAATGGGGGCAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    let s2_col = "GATACATAAAAAAAATGGGGGC";
    let mut align = Aligner::new(s1_row, s2_col, Box::new(scorer));
    for _i in 1 .. s1_row.len()*s2_col.len() {
        align.generate_next_score();
        let _ = terminal.draw(|frame| sil.draw(frame, &mut align)); // Brutalmente rimosso ? Perchè uesta roba non ritorna un Result<>`
        thread::sleep(time::Duration::from_millis(100));
        
        if let Ok(true) = event::poll(Duration::ZERO) {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                if code == KeyCode::Char('q') {
                    return Ok(());
                }
            }
        }
    }
    return Ok(())
}

#[derive(Debug, Clone, Copy)] // TODO signal bug to vscode about this gliph (?)
pub struct Silly {

}