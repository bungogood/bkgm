use crate::dice::Dice;
use crate::rules::{legal_positions_with, ClassicRules};
use crate::{Position, State, VariantPosition};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
struct Step {
    from: usize,
    to: usize,
}

pub fn legal(
    position: VariantPosition,
    dice: Dice,
) -> Result<Vec<(String, VariantPosition)>, String> {
    match position {
        VariantPosition::Backgammon(p) => legal_for(p, dice, VariantPosition::Backgammon),
        VariantPosition::Nackgammon(p) => legal_for(p, dice, VariantPosition::Nackgammon),
        VariantPosition::Longgammon(p) => legal_for(p, dice, VariantPosition::Longgammon),
        VariantPosition::Hypergammon(p) => legal_for(p, dice, VariantPosition::Hypergammon),
        VariantPosition::Hypergammon2(p) => legal_for(p, dice, VariantPosition::Hypergammon2),
        VariantPosition::Hypergammon4(p) => legal_for(p, dice, VariantPosition::Hypergammon4),
        VariantPosition::Hypergammon5(p) => legal_for(p, dice, VariantPosition::Hypergammon5),
    }
}

pub fn encode(
    position: VariantPosition,
    next_position: VariantPosition,
    dice: Dice,
) -> Result<String, String> {
    match (position, next_position) {
        (VariantPosition::Backgammon(start), VariantPosition::Backgammon(next)) => {
            encode_for(start, next, dice)
        }
        (VariantPosition::Nackgammon(start), VariantPosition::Nackgammon(next)) => {
            encode_for(start, next, dice)
        }
        (VariantPosition::Longgammon(start), VariantPosition::Longgammon(next)) => {
            encode_for(start, next, dice)
        }
        (VariantPosition::Hypergammon(start), VariantPosition::Hypergammon(next)) => {
            encode_for(start, next, dice)
        }
        (VariantPosition::Hypergammon2(start), VariantPosition::Hypergammon2(next)) => {
            encode_for(start, next, dice)
        }
        (VariantPosition::Hypergammon4(start), VariantPosition::Hypergammon4(next)) => {
            encode_for(start, next, dice)
        }
        (VariantPosition::Hypergammon5(start), VariantPosition::Hypergammon5(next)) => {
            encode_for(start, next, dice)
        }
        _ => Err("variant mismatch while encoding move".to_string()),
    }
}

fn encode_for<const N: u8>(
    start: Position<N>,
    next: Position<N>,
    dice: Dice,
) -> Result<String, String> {
    let target_after_steps = next.flip();
    let mut steps = Vec::new();

    for order in die_orders(dice) {
        steps.clear();
        if find_steps_for_target(start, &order, 0, target_after_steps, &mut steps) {
            return Ok(format_steps(&steps));
        }
    }

    Err("failed to derive move text from legal child".to_string())
}

fn find_steps_for_target<const N: u8>(
    current: Position<N>,
    order: &[usize],
    die_idx: usize,
    target: Position<N>,
    steps: &mut Vec<Step>,
) -> bool {
    if die_idx == order.len() {
        return current == target;
    }

    let die = order[die_idx];
    let mut moved_any = false;
    for from in 1..=25 {
        if let Some(next) = current.try_move_single_checker(from, die) {
            moved_any = true;
            let to = from.saturating_sub(die);
            steps.push(Step { from, to });
            if find_steps_for_target(next, order, die_idx + 1, target, steps) {
                return true;
            }
            steps.pop();
        }
    }

    if !moved_any {
        return find_steps_for_target(current, order, die_idx + 1, target, steps);
    }

    false
}

pub fn normalize(text: &str) -> Option<String> {
    if text.trim().eq_ignore_ascii_case("pass") {
        return Some("pass".to_string());
    }
    let mut steps: Vec<String> = Vec::new();
    for token in text.split_whitespace() {
        let cleaned = token.replace(['*', ','], "");
        let parts: Vec<&str> = cleaned.split('/').collect();
        let points = parse_path_points(&parts)?;
        for pair in points.windows(2) {
            steps.push(format!(
                "{}/{}",
                point_to_text(pair[0]),
                point_to_text(pair[1])
            ));
        }
    }
    if steps.is_empty() {
        None
    } else {
        Some(steps.join(" "))
    }
}

pub fn apply(position: VariantPosition, dice: Dice, text: &str) -> Option<VariantPosition> {
    match position {
        VariantPosition::Backgammon(p) => apply_for(p, dice, text).map(VariantPosition::Backgammon),
        VariantPosition::Nackgammon(p) => apply_for(p, dice, text).map(VariantPosition::Nackgammon),
        VariantPosition::Longgammon(p) => apply_for(p, dice, text).map(VariantPosition::Longgammon),
        VariantPosition::Hypergammon(p) => {
            apply_for(p, dice, text).map(VariantPosition::Hypergammon)
        }
        VariantPosition::Hypergammon2(p) => {
            apply_for(p, dice, text).map(VariantPosition::Hypergammon2)
        }
        VariantPosition::Hypergammon4(p) => {
            apply_for(p, dice, text).map(VariantPosition::Hypergammon4)
        }
        VariantPosition::Hypergammon5(p) => {
            apply_for(p, dice, text).map(VariantPosition::Hypergammon5)
        }
    }
}

fn legal_for<const N: u8>(
    start: Position<N>,
    dice: Dice,
    wrap: fn(Position<N>) -> VariantPosition,
) -> Result<Vec<(String, VariantPosition)>, String> {
    let legal = legal_positions_with::<ClassicRules, N>(start, &dice);
    let legal_set: HashSet<Position<N>> = legal.iter().copied().collect();
    let mut move_by_child: HashMap<Position<N>, String> = HashMap::new();

    for order in die_orders(dice) {
        let mut steps = Vec::new();
        collect_paths(start, &order, 0, &mut steps, &legal_set, &mut move_by_child);
        let mut steps_flipped = Vec::new();
        collect_paths(
            start.flip(),
            &order,
            0,
            &mut steps_flipped,
            &legal_set,
            &mut move_by_child,
        );
    }

    let mut out = Vec::with_capacity(legal.len());
    for child in legal {
        if let Some(mv) = move_by_child.get(&child) {
            out.push((mv.clone(), wrap(child)));
        }
    }
    Ok(out)
}

fn apply_for<const N: u8>(position: Position<N>, dice: Dice, text: &str) -> Option<Position<N>> {
    let normalized = normalize(text)?;
    let steps = parse_steps(&normalized)?;
    let mut remaining = match dice {
        Dice::Double(d) => vec![d, d, d, d],
        Dice::Mixed(m) => vec![m.big(), m.small()],
    };
    let current = apply_steps(position, &steps, &mut remaining)?;
    if remaining.iter().any(|d| any_move_for_die(current, *d)) {
        return None;
    }
    Some(current.flip())
}

fn apply_steps<const N: u8>(
    current: Position<N>,
    steps: &[Step],
    remaining: &mut Vec<usize>,
) -> Option<Position<N>> {
    if steps.is_empty() {
        return Some(current);
    }
    let step = steps[0];
    for idx in 0..remaining.len() {
        let die = remaining[idx];
        if !step_matches_die(step, die) {
            continue;
        }
        if let Some(next) = current.try_move_single_checker(step.from, die) {
            let removed = remaining.remove(idx);
            if let Some(done) = apply_steps(next, &steps[1..], remaining) {
                return Some(done);
            }
            remaining.insert(idx, removed);
        }
    }
    None
}

fn step_matches_die(step: Step, die: usize) -> bool {
    if step.to == 0 {
        step.from <= die
    } else {
        step.from > die && step.from - die == step.to
    }
}

fn parse_steps(text: &str) -> Option<Vec<Step>> {
    if text.trim().eq_ignore_ascii_case("pass") {
        return Some(Vec::new());
    }
    let mut steps = Vec::new();
    for token in text.split_whitespace() {
        let parts: Vec<&str> = token.split('/').collect();
        if parts.len() != 2 {
            return None;
        }
        steps.push(Step {
            from: parse_from_point(parts[0])?,
            to: parse_to_point(parts[1])?,
        });
    }
    Some(steps)
}

fn any_move_for_die<const N: u8>(position: Position<N>, die: usize) -> bool {
    (1..=25).any(|from| position.try_move_single_checker(from, die).is_some())
}

fn collect_paths<const N: u8>(
    current: Position<N>,
    order: &[usize],
    die_idx: usize,
    steps: &mut Vec<Step>,
    legal_set: &HashSet<Position<N>>,
    move_by_child: &mut HashMap<Position<N>, String>,
) {
    if die_idx == order.len() {
        let child = current.flip();
        if legal_set.contains(&child) {
            let mv = format_steps(steps);
            move_by_child.entry(child).or_insert(mv);
        }
        if legal_set.contains(&current) {
            let mv = format_steps(steps);
            move_by_child.entry(current).or_insert(mv);
        }
        return;
    }

    let die = order[die_idx];
    let mut found_any = false;
    for from in 1..=25 {
        if let Some(next) = current.try_move_single_checker(from, die) {
            found_any = true;
            let to = from.saturating_sub(die);
            steps.push(Step { from, to });
            collect_paths(next, order, die_idx + 1, steps, legal_set, move_by_child);
            steps.pop();
        }
    }
    if !found_any {
        collect_paths(current, order, die_idx + 1, steps, legal_set, move_by_child);
    }
}

fn format_steps(steps: &[Step]) -> String {
    if steps.is_empty() {
        return "pass".to_string();
    }
    steps
        .iter()
        .map(|s| format!("{}/{}", point_to_text(s.from), point_to_text(s.to)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_from_point(raw: &str) -> Option<usize> {
    let lower = raw.trim().to_ascii_lowercase();
    if lower == "bar" {
        return Some(25);
    }
    let value = lower.parse::<usize>().ok()?;
    if (1..=24).contains(&value) {
        Some(value)
    } else {
        None
    }
}

fn parse_to_point(raw: &str) -> Option<usize> {
    let lower = raw.trim().to_ascii_lowercase();
    if lower == "off" {
        return Some(0);
    }
    let value = lower.parse::<usize>().ok()?;
    if (1..=24).contains(&value) {
        Some(value)
    } else {
        None
    }
}

fn parse_mid_point(raw: &str) -> Option<usize> {
    let lower = raw.trim().to_ascii_lowercase();
    let value = lower.parse::<usize>().ok()?;
    if (1..=24).contains(&value) {
        Some(value)
    } else {
        None
    }
}

fn parse_path_points(parts: &[&str]) -> Option<Vec<usize>> {
    if parts.len() < 2 {
        return None;
    }
    let mut points = Vec::with_capacity(parts.len());
    points.push(parse_from_point(parts[0])?);
    for idx in 1..parts.len() {
        if idx == parts.len() - 1 {
            points.push(parse_to_point(parts[idx])?);
        } else {
            points.push(parse_mid_point(parts[idx])?);
        }
    }
    Some(points)
}

fn point_to_text(point: usize) -> String {
    match point {
        25 => "bar".to_string(),
        0 => "off".to_string(),
        p => p.to_string(),
    }
}

fn die_orders(dice: Dice) -> Vec<Vec<usize>> {
    match dice {
        Dice::Double(d) => vec![vec![d, d, d, d]],
        Dice::Mixed(m) => vec![vec![m.big(), m.small()], vec![m.small(), m.big()]],
    }
}

#[cfg(test)]
mod tests {
    use super::{apply, encode, legal, normalize};
    use crate::codecs::gnuid;
    use crate::dice::Dice;
    use crate::{Game, Variant};

    #[test]
    fn move_texts_exist_for_start_position() {
        let game = Game::new(Variant::Backgammon);
        let dice = Dice::new(6, 1);
        let moves = legal(game.position(), dice).unwrap();
        assert!(!moves.is_empty());
        assert!(moves.iter().all(|(mv, _)| !mv.is_empty()));
    }

    #[test]
    fn normalize_supports_bar_off_and_chains() {
        let normalized = normalize("bar/22 6/off 13/8/5").unwrap();
        assert_eq!(normalized, "bar/22 6/off 13/8 8/5");
    }

    #[test]
    fn normalize_rejects_numeric_bar_off_aliases() {
        assert!(normalize("25/22").is_none());
        assert!(normalize("6/0").is_none());
    }

    #[test]
    fn legal_moves_apply_roundtrip_for_multiple_rolls() {
        let game = Game::new(Variant::Backgammon);
        let position = game.position();
        let rolls = [
            Dice::new(6, 1),
            Dice::new(5, 3),
            Dice::new(4, 4),
            Dice::new(2, 1),
        ];

        for dice in rolls {
            let moves = legal(position, dice).unwrap();
            assert!(!moves.is_empty());
            for (mv, child) in moves {
                let applied = apply(position, dice, &mv).expect("move should apply");
                assert_eq!(gnuid::encode(applied), gnuid::encode(child));
            }
        }
    }

    #[test]
    fn encode_move_roundtrips_to_same_child() {
        let game = Game::new(Variant::Backgammon);
        let position = game.position();
        let dice = Dice::new(6, 1);

        for child in game.legal_positions(&dice) {
            let mv = encode(position, child, dice).expect("child should be encodable");
            let applied = apply(position, dice, &mv).expect("encoded move should apply");
            assert_eq!(gnuid::encode(applied), gnuid::encode(child));
        }
    }
}
