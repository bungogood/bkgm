use crate::dice::Dice;
use crate::rules::{legal_positions_with, ClassicRules};
use crate::{Position, State, VariantPosition};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum MoveTextError {
    #[error("variant mismatch while encoding move")]
    VariantMismatch,
    #[error("failed to derive move text from legal child")]
    EncodeTargetNotReachable,
}

pub type MoveTextResult<T> = Result<T, MoveTextError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveStep {
    pub from: usize,
    pub to: usize,
}

pub fn legal_steps(
    position: VariantPosition,
    dice: Dice,
) -> MoveTextResult<Vec<(Vec<MoveStep>, VariantPosition)>> {
    match position {
        VariantPosition::Backgammon(p) => legal_for_steps(p, dice, VariantPosition::Backgammon),
        VariantPosition::Nackgammon(p) => legal_for_steps(p, dice, VariantPosition::Nackgammon),
        VariantPosition::Longgammon(p) => legal_for_steps(p, dice, VariantPosition::Longgammon),
        VariantPosition::Hypergammon(p) => legal_for_steps(p, dice, VariantPosition::Hypergammon),
        VariantPosition::Hypergammon2(p) => legal_for_steps(p, dice, VariantPosition::Hypergammon2),
        VariantPosition::Hypergammon4(p) => legal_for_steps(p, dice, VariantPosition::Hypergammon4),
        VariantPosition::Hypergammon5(p) => legal_for_steps(p, dice, VariantPosition::Hypergammon5),
    }
}

pub fn legal(
    position: VariantPosition,
    dice: Dice,
) -> MoveTextResult<Vec<(String, VariantPosition)>> {
    let with_steps = legal_steps(position, dice)?;
    Ok(with_steps
        .into_iter()
        .map(|(steps, pos)| (format_move_steps(&steps), pos))
        .collect())
}

pub fn encode(
    position: VariantPosition,
    next_position: VariantPosition,
    dice: Dice,
) -> MoveTextResult<String> {
    let steps = encode_steps(position, next_position, dice)?;
    Ok(format_move_steps(&steps))
}

pub fn encode_steps(
    position: VariantPosition,
    next_position: VariantPosition,
    dice: Dice,
) -> MoveTextResult<Vec<MoveStep>> {
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
        _ => Err(MoveTextError::VariantMismatch),
    }
}

fn encode_for<const N: u8>(
    start: Position<N>,
    next: Position<N>,
    dice: Dice,
) -> MoveTextResult<Vec<MoveStep>> {
    let target_after_steps = next.flip();
    let mut steps = Vec::with_capacity(4);
    let mut out: Option<Vec<MoveStep>> = None;
    for_each_die_order(dice, |order| {
        steps.clear();
        if find_steps_for_target(start, order, 0, target_after_steps, &mut steps) {
            out = Some(steps.clone());
            true
        } else {
            false
        }
    });
    if let Some(found) = out {
        return Ok(found);
    }

    Err(MoveTextError::EncodeTargetNotReachable)
}

fn find_steps_for_target<const N: u8>(
    current: Position<N>,
    order: &[usize],
    die_idx: usize,
    target: Position<N>,
    steps: &mut Vec<MoveStep>,
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
            steps.push(MoveStep { from, to });
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

fn legal_for_steps<const N: u8>(
    start: Position<N>,
    dice: Dice,
    wrap: fn(Position<N>) -> VariantPosition,
) -> MoveTextResult<Vec<(Vec<MoveStep>, VariantPosition)>> {
    let legal = legal_positions_with::<ClassicRules, N>(start, &dice);
    let legal_set: HashSet<Position<N>> = legal.iter().copied().collect();
    let mut move_by_child: HashMap<Position<N>, Vec<MoveStep>> = HashMap::new();

    for_each_die_order(dice, |order| {
        let mut steps = Vec::with_capacity(4);
        collect_paths(start, order, 0, &mut steps, &legal_set, &mut move_by_child);
        let mut steps_flipped = Vec::with_capacity(4);
        collect_paths(
            start.flip(),
            order,
            0,
            &mut steps_flipped,
            &legal_set,
            &mut move_by_child,
        );
        false
    });

    let mut out = Vec::with_capacity(legal.len());
    for child in legal {
        if let Some(steps) = move_by_child.get(&child) {
            out.push((steps.clone(), wrap(child)));
        }
    }
    Ok(out)
}

fn apply_for<const N: u8>(position: Position<N>, dice: Dice, text: &str) -> Option<Position<N>> {
    let normalized = normalize(text)?;
    let steps = parse_move_steps(&normalized)?;
    let (dice_buf, dice_len) = match dice {
        Dice::Double(d) => ([d, d, d, d], 4usize),
        Dice::Mixed(m) => ([m.big(), m.small(), 0, 0], 2usize),
    };
    let (current, used_mask) = apply_steps(position, &steps, &dice_buf, dice_len, 0)?;
    for (idx, die) in dice_buf.iter().enumerate().take(dice_len) {
        if (used_mask & (1 << idx)) == 0 && any_move_for_die(current, *die) {
            return None;
        }
    }
    Some(current.flip())
}

fn apply_steps<const N: u8>(
    current: Position<N>,
    steps: &[MoveStep],
    dice: &[usize; 4],
    dice_len: usize,
    used_mask: u8,
) -> Option<(Position<N>, u8)> {
    if steps.is_empty() {
        return Some((current, used_mask));
    }
    let step = steps[0];
    for idx in 0..dice_len {
        if (used_mask & (1 << idx)) != 0 {
            continue;
        }
        let die = dice[idx];
        if !step_matches_die(step, die) {
            continue;
        }
        if let Some(next) = current.try_move_single_checker(step.from, die) {
            if let Some(done) =
                apply_steps(next, &steps[1..], dice, dice_len, used_mask | (1 << idx))
            {
                return Some(done);
            }
        }
    }
    None
}

fn step_matches_die(step: MoveStep, die: usize) -> bool {
    if step.to == 0 {
        step.from <= die
    } else {
        step.from > die && step.from - die == step.to
    }
}

pub fn parse_move_steps(text: &str) -> Option<Vec<MoveStep>> {
    if text.trim().eq_ignore_ascii_case("pass") {
        return Some(Vec::new());
    }
    let mut steps = Vec::new();
    for token in text.split_whitespace() {
        let parts: Vec<&str> = token.split('/').collect();
        if parts.len() != 2 {
            return None;
        }
        steps.push(MoveStep {
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
    steps: &mut Vec<MoveStep>,
    legal_set: &HashSet<Position<N>>,
    move_by_child: &mut HashMap<Position<N>, Vec<MoveStep>>,
) {
    if die_idx == order.len() {
        let child = current.flip();
        if legal_set.contains(&child) {
            move_by_child.entry(child).or_insert_with(|| steps.clone());
        }
        if legal_set.contains(&current) {
            move_by_child
                .entry(current)
                .or_insert_with(|| steps.clone());
        }
        return;
    }

    let die = order[die_idx];
    let mut found_any = false;
    for from in 1..=25 {
        if let Some(next) = current.try_move_single_checker(from, die) {
            found_any = true;
            let to = from.saturating_sub(die);
            steps.push(MoveStep { from, to });
            collect_paths(next, order, die_idx + 1, steps, legal_set, move_by_child);
            steps.pop();
        }
    }
    if !found_any {
        collect_paths(current, order, die_idx + 1, steps, legal_set, move_by_child);
    }
}

pub fn format_move_steps(steps: &[MoveStep]) -> String {
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

fn for_each_die_order<F>(dice: Dice, mut f: F)
where
    F: FnMut(&[usize]) -> bool,
{
    match dice {
        Dice::Double(d) => {
            let order = [d, d, d, d];
            let _ = f(&order);
        }
        Dice::Mixed(m) => {
            let big_small = [m.big(), m.small()];
            if f(&big_small) {
                return;
            }
            let small_big = [m.small(), m.big()];
            let _ = f(&small_big);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{apply, encode, format_move_steps, legal, legal_steps, normalize};
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

    #[test]
    fn legal_steps_match_legal_text() {
        let game = Game::new(Variant::Backgammon);
        let dice = Dice::new(6, 1);
        let text_moves = legal(game.position(), dice).unwrap();
        let step_moves = legal_steps(game.position(), dice).unwrap();
        let rendered: Vec<(String, _)> = step_moves
            .into_iter()
            .map(|(steps, pos)| (format_move_steps(&steps), pos))
            .collect();
        assert_eq!(text_moves.len(), rendered.len());
        for (mv, child) in text_moves {
            assert!(rendered.iter().any(|(m, c)| m == &mv && c == &child));
        }
    }
}
