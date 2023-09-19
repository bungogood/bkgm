use base64::{engine::general_purpose, Engine as _};
use std::{
    cmp::max,
    fmt::{Display, Formatter, Result},
    ops::Add,
    vec,
};

const WHITE: bool = true;
const BLACK: bool = false;

const BAR: usize = 25;
const OFF: usize = 24;

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct Step {
    pub from: usize,
    pub to: usize,
    pub hit: bool,
}

impl Step {
    pub fn from<S: Into<String>>(ms: S) -> Option<Self> {
        let ms = ms.into();
        let mut ms = ms.split('/');
        let move_str = ms.next()?;
        let hit = move_str.ends_with('*');
        let from = move_str.trim_end_matches('*').parse::<usize>().ok()? - 1;
        let to = ms.next()?.parse::<usize>().ok()? - 1;
        Some(Step { from, to, hit })
    }
}

impl Display for Step {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}/{}", self.from + 1, self.to + 1)?;
        if self.hit {
            write!(f, "*")
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Action {
    steps: Vec<Step>,
}

impl Action {
    pub fn new(steps: Vec<Step>) -> Self {
        Action { steps }
    }

    pub fn from<S: Into<String>>(ms: S) -> Option<Self> {
        let ms = ms.into();
        let mut steps = Vec::new();
        for step in ms.split(' ') {
            steps.push(Step::from(step)?);
        }
        Some(Action { steps })
    }

    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Step> {
        self.steps.iter()
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut steps = self.steps.iter();
        let mut prev = steps.next().unwrap().clone();

        for step in steps {
            if step.from == prev.to {
                prev.to = step.to;
            } else {
                write!(f, "{} ", prev)?;
                prev = step.clone();
            }
        }
        write!(f, "{}", prev)
    }
}

#[derive(Clone, Debug)]
pub struct Tree {
    pub step: Step,
    pub children: Vec<Tree>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    board: [i32; 24],
    bar: (i32, i32),
    off: (i32, i32),
    is_white: bool,
}

impl State {
    pub fn new() -> Self {
        State {
            board: [
                -2, 0, 0, 0, 0, 5, // Actioner on roll's home board (points 1-6)
                0, 3, 0, 0, 0, -5, // Actioner on roll's outer board (points 7-12)
                5, 0, 0, 0, -3, 0, // Actioner not on roll's outer board (points 13-18)
                -5, 0, 0, 0, 0, 2, // Actioner not on roll's home board (points 19-24)
            ],
            bar: (0, 0),
            off: (0, 0),
            is_white: WHITE,
        }
    }

    #[inline]
    fn next_point(&self, point: usize) -> usize {
        if self.is_white {
            point - 1
        } else {
            point + 1
        }
    }

    #[inline]
    pub fn dest(start: usize, delta: usize, player: bool) -> isize {
        if player {
            start as isize - delta as isize
        } else {
            start as isize + delta as isize
        }
    }

    #[inline]
    fn is_hit(&self, to: usize) -> bool {
        self.board[to as usize] != 0 && (self.board[to as usize] < 0) == self.is_white
    }

    pub fn valid(&self, to: isize) -> bool {
        if 0 > to || to > 23 {
            return false;
        }

        if self.board[to as usize] == 0 {
            return true;
        }

        (self.board[to as usize] > 0) == self.is_white || self.board[to as usize].abs() == 1
    }

    pub fn pieces_from(&self, start: usize, player: bool) -> Vec<usize> {
        if player {
            (0..=start)
                .rev()
                .filter(|&point| self.board[point] > 0)
                .collect()
        } else {
            (start..24).filter(|&point| self.board[point] < 0).collect()
        }
    }

    pub fn possible_actions(&self, dice: (usize, usize)) -> Vec<Action> {
        let mut trees = Vec::new();
        let mut actions;

        let mut depth;

        let from = if self.is_white { 23 } else { 0 };

        if dice.0 == dice.1 {
            let die = vec![dice.0, dice.0, dice.0, dice.0];
            depth = self.clone().rec_actions(&mut trees, from, &die, false);
            actions = Self::tree_actions(trees, Vec::new(), depth);
        } else {
            let die = vec![dice.0, dice.1];
            depth = self.clone().rec_actions(&mut trees, from, &die, false);
            let die = vec![dice.1, dice.0];
            depth = max(
                depth,
                self.clone().rec_actions(&mut trees, from, &die, true),
            );
            actions = Self::tree_actions(trees, Vec::new(), depth);
            actions = Self::unique_actions(actions);
        }
        actions
    }

    fn rec_actions(
        &mut self,
        pap: &mut Vec<Tree>,
        start: usize,
        dice: &[usize],
        jump: bool,
    ) -> usize {
        if dice.is_empty() {
            return 0;
        }

        let mut lng = 0;

        let current_die = dice[0];

        for from in self.pieces_from(start, self.is_white) {
            let to = Self::dest(from, current_die, self.is_white);
            if self.valid(to) {
                let step = Step {
                    from,
                    to: to as usize,
                    hit: self.is_hit(to as usize),
                };
                let mut child = self.clone();
                let mut children = Vec::new();
                let next = if jump { self.next_point(from) } else { from };
                child.apply_step(step);
                lng = max(
                    lng,
                    child.rec_actions(&mut children, next, &dice[1..], jump),
                );
                // self.undo_step(step);
                pap.push(Tree { step, children });
            }
        }
        lng + 1
    }

    fn tree_actions(trees: Vec<Tree>, history: Vec<Step>, depth: usize) -> Vec<Action> {
        let mut plays = Vec::new();
        for tree in trees {
            let mut steps = history.clone();
            steps.push(tree.step);
            if tree.children.is_empty() && depth == 1 {
                plays.push(Action { steps });
            } else {
                plays.append(&mut Self::tree_actions(tree.children, steps, depth - 1));
            }
        }
        plays
    }

    fn unique_actions(actions: Vec<Action>) -> Vec<Action> {
        let mut unique = Vec::new();

        for action in actions {
            let mut steps = action.steps.iter();
            let mut prev = steps.next().unwrap().clone();
            let mut new_steps = Vec::new();
            for step in steps {
                if step.from == prev.to {
                    prev.to = step.to;
                } else {
                    new_steps.push(prev.clone());
                    prev = step.clone();
                }
            }
            new_steps.push(prev.clone());
            let new_action = Action { steps: new_steps };
            if !unique.contains(&new_action) {
                unique.push(new_action);
            }
        }

        unique
    }

    fn apply_step(&mut self, step: Step) {
        if self.is_white {
            if step.from == BAR {
                self.bar.0 -= 1;
            } else {
                self.board[step.from] -= 1;
            }

            if step.to == OFF {
                self.off.0 += 1;
            } else if step.hit {
                self.board[step.to] = 1;
                self.bar.1 += 1;
            } else {
                self.board[step.to] += 1;
            }
        } else {
            if step.from == BAR {
                self.bar.1 -= 1;
            } else {
                self.board[step.from] += 1;
            }

            if step.to == OFF {
                self.off.1 += 1;
            } else if step.hit {
                self.board[step.to] = -1;
                self.bar.0 += 1;
            } else {
                self.board[step.to] -= 1;
            }
        }
    }

    fn undo_step(&mut self, step: Step) {
        if !self.is_white {
            if step.from == BAR {
                self.bar.0 += 1;
            } else {
                self.board[step.from] += 1;
            }

            if step.to == OFF {
                self.off.0 -= 1;
            } else if step.hit {
                self.board[step.to] = -1;
                self.bar.1 += 1;
            } else {
                self.board[step.to] -= 1;
            }
        } else {
            if step.from == BAR {
                self.bar.1 += 1;
            } else {
                self.board[step.from] -= 1;
            }

            if step.to == OFF {
                self.off.1 -= 1;
            } else if step.hit {
                self.board[step.to] = 1;
                self.bar.0 -= 1;
            } else {
                self.board[step.to] += 1;
            }
        }
    }

    pub fn apply_action(&mut self, action: &Action) {
        for step in &action.steps {
            self.apply_step(*step);
        }
        self.is_white = !self.is_white;
    }

    pub fn undo_action(&mut self, action: &Action) {
        for step in &action.steps {
            self.undo_step(*step);
        }
        self.is_white = !self.is_white;
    }

    fn flip(&self) -> State {
        State {
            board: self
                .board
                .iter()
                .rev()
                .map(|&num| -num)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            bar: (self.bar.1, self.bar.0),
            off: (self.off.1, self.off.0),
            is_white: !self.is_white,
        }
    }

    pub fn decode(key: [u8; 10]) -> State {
        let mut bit_index = 0;
        let mut board = [0i32; 24];

        let mut white_bar = 0;
        let mut black_bar = 0;
        let mut white_pieces = 0;
        let mut black_pieces = 0;

        for point in (0..24).rev() {
            while (key[bit_index / 8] >> (bit_index % 8)) & 1 == 1 {
                board[point] -= 1;
                black_pieces += 1;
                bit_index += 1;
            }
            bit_index += 1; // Appending a 0
        }

        while (key[bit_index / 8] >> (bit_index % 8)) & 1 == 1 {
            black_bar += 1;
            bit_index += 1;
        }

        bit_index += 1; // Appending a 0

        for point in 0..24 {
            while (key[bit_index / 8] >> (bit_index % 8)) & 1 == 1 {
                board[point] += 1;
                white_pieces += 1;
                bit_index += 1;
            }
            bit_index += 1; // Appending a 0
        }

        while (key[bit_index / 8] >> (bit_index % 8)) & 1 == 1 {
            white_bar += 1;
            bit_index += 1;
        }

        State {
            board: board,
            bar: (white_bar, black_bar),
            off: (15 - white_pieces - white_bar, 15 - black_pieces - black_bar),
            is_white: true,
        }
    }

    pub fn from_id(id: String) -> State {
        let key = general_purpose::STANDARD.decode(id.add("==")).unwrap();
        State::decode(key.try_into().unwrap())
    }

    pub fn encode(&self) -> [u8; 10] {
        let mut key = [0u8; 10];
        let mut bit_index = 0;

        let game = match self.is_white {
            true => self.clone(),
            false => self.flip(),
        };

        // Encoding the position for the player not on roll
        for point in (0..24).rev() {
            for _ in 0..-game.board[point] {
                key[bit_index / 8] |= 1 << (bit_index % 8);
                bit_index += 1; // Appending a 1
            }
            bit_index += 1; // Appending a 0
        }
        for _ in 0..game.bar.1 {
            key[bit_index / 8] |= 1 << (bit_index % 8);
            bit_index += 1; // Appending a 1
        }
        bit_index += 1; // Appending a 0

        // Encoding the position for the player on roll
        for point in 0..24 {
            for _ in 0..game.board[point] {
                key[bit_index / 8] |= 1 << (bit_index % 8);
                bit_index += 1; // Appending a 1
            }
            bit_index += 1; // Appending a 0
        }
        for _ in 0..game.bar.0 {
            key[bit_index / 8] |= 1 << (bit_index % 8);
            bit_index += 1; // Appending a 1
        }

        key
    }

    pub fn position_id(&self) -> String {
        let key = self.encode();
        let b64 = String::from(general_purpose::STANDARD.encode(&key));
        b64[..14].to_string()
    }

    pub fn display(&self) {
        println!("Position ID: {}", self.position_id());
        println!("┌13─14─15─16─17─18─┬───┬19─20─21─22─23─24─┬───┐");
        for row in 0..5 {
            print!("│");
            for point in 12..=23 {
                Self::print_point(self.board[point], row);

                if point == 17 {
                    print!("│");
                    Self::print_point(-self.bar.1, row);
                    print!("│");
                }
            }
            print!("│");
            Self::print_point(-self.off.1, row);
            println!("│");
        }
        println!("│                  │BAR│                  │OFF│");
        for row in (0..5).rev() {
            print!("│");
            for point in (0..=11).rev() {
                if point == 5 {
                    print!("│");
                    Self::print_point(self.bar.0, row);
                    print!("│");
                }
                Self::print_point(self.board[point], row)
            }
            print!("│");
            Self::print_point(self.off.0, row);
            println!("│");
        }
        println!("└12─11─10──9──8──7─┴───┴─6──5──4──3──2──1─┴───┘");
    }

    fn print_point(value: i32, row: i32) {
        match (value, row) {
            (val, 4) if val.abs() > 5 => print!(" {} ", val.abs()),
            (val, _) if val > row => print!(" X "),
            (val, _) if val < -row => print!(" O "),
            _ => print!("   "),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn start_id() {
        let game = super::State::new();
        let id = game.position_id();
        assert_eq!(id, "4HPwATDgc/ABMA");
    }

    #[test]
    fn matching_ids() {
        let pids = [
            "4HPwATDgc/ABMA", // starting position
            "jGfkASjg8wcBMA", // random position
            "zGbiIQgxH/AAWA", // X bar
            "zGbiIYCYD3gALA", // O off
        ];
        for pid in pids {
            let game = super::State::from_id(pid.to_string());
            assert_eq!(pid, game.position_id());
        }
    }

    #[test]
    fn pieces_from() {
        let game = super::State::new();
        for start in 0..24 {
            game.pieces_from(start, super::WHITE)
                .iter()
                .for_each(|&piece| assert!(piece <= start));
            game.pieces_from(start, super::BLACK)
                .iter()
                .for_each(|&piece| assert!(piece >= start));
        }
    }
}
