use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub struct Move {
    steps: Vec<Step>,
}

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

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.from + 1, self.to + 1)?;
        if self.hit {
            write!(f, "*")
        } else {
            Ok(())
        }
    }
}

impl Move {
    pub fn new(steps: Vec<Step>) -> Self {
        Move { steps }
    }

    // TODO: fix multiple hit (e.g. 1/2*/3*/4*)
    pub fn from<S: Into<String>>(ms: S) -> Option<Self> {
        let ms = ms.into();
        let mut steps = Vec::new();
        for step in ms.split(' ') {
            steps.push(Step::from(step)?);
        }
        Some(Move { steps })
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

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut steps = self.steps.iter();
        let mut prev = steps.next().unwrap().clone();

        for step in steps {
            if step.from == prev.to && !prev.hit {
                prev.to = step.to;
            } else {
                write!(f, "{} ", prev)?;
                prev = step.clone();
            }
        }
        write!(f, "{}", prev)
    }
}

#[cfg(test)]
mod move_tests {
    use super::Move;

    #[test]
    fn parse_move() {
        let m = Move::from("1/2 3/4 5/6").unwrap();
        assert_eq!(m.to_string(), "1/2 3/4 5/6");
    }

    #[test]
    #[ignore = "TODO: fix multiple hit"]
    fn multiple_hit() {
        let m = Move::from("1/2*/3*/4*").unwrap();
        assert_eq!(m.to_string(), "1/2*/3*/4*");
    }
}
