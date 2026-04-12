use base64::engine::general_purpose;
use base64::Engine;

use crate::position::{Position, O_BAR, X_BAR};
use crate::{State, Variant, VariantPosition};

pub fn encode(position: VariantPosition) -> String {
    match position {
        VariantPosition::Backgammon(p) => encode_position(&p),
        VariantPosition::Nackgammon(p) => encode_position(&p),
        VariantPosition::Longgammon(p) => encode_position(&p),
        VariantPosition::Hypergammon(p) => encode_position(&p),
        VariantPosition::Hypergammon2(p) => encode_position(&p),
        VariantPosition::Hypergammon4(p) => encode_position(&p),
        VariantPosition::Hypergammon5(p) => encode_position(&p),
    }
}

pub fn decode(variant: Variant, id: &str) -> Option<VariantPosition> {
    match variant {
        Variant::Backgammon => decode_position::<15>(id).map(VariantPosition::Backgammon),
        Variant::Nackgammon => decode_position::<15>(id).map(VariantPosition::Nackgammon),
        Variant::Longgammon => decode_position::<15>(id).map(VariantPosition::Longgammon),
        Variant::Hypergammon => decode_position::<3>(id).map(VariantPosition::Hypergammon),
        Variant::Hypergammon2 => decode_position::<2>(id).map(VariantPosition::Hypergammon2),
        Variant::Hypergammon4 => decode_position::<4>(id).map(VariantPosition::Hypergammon4),
        Variant::Hypergammon5 => decode_position::<5>(id).map(VariantPosition::Hypergammon5),
    }
}

pub fn encode_position<const N: u8>(position: &Position<N>) -> String {
    let key = encode_key(position);
    let b64 = general_purpose::STANDARD.encode(key);
    b64[..14].to_string()
}

pub fn decode_position<const N: u8>(id: &str) -> Option<Position<N>> {
    let padded_id = format!("{}==", id);
    let key = general_purpose::STANDARD.decode(padded_id).ok()?;
    let key: [u8; 10] = key.try_into().ok()?;
    Some(decode_key(key))
}

pub fn encode_key<const N: u8>(position: &Position<N>) -> [u8; 10] {
    let mut key = [0u8; 10];
    let mut bit_index = 0;

    for point in (O_BAR + 1..X_BAR).rev() {
        for _ in 0..-position.pip(point) {
            key[bit_index / 8] |= 1 << (bit_index % 8);
            bit_index += 1;
        }
        bit_index += 1;
    }
    for _ in 0..position.o_bar() {
        key[bit_index / 8] |= 1 << (bit_index % 8);
        bit_index += 1;
    }
    bit_index += 1;

    for point in O_BAR + 1..X_BAR {
        for _ in 0..position.pip(point) {
            key[bit_index / 8] |= 1 << (bit_index % 8);
            bit_index += 1;
        }
        bit_index += 1;
    }
    for _ in 0..position.x_bar() {
        key[bit_index / 8] |= 1 << (bit_index % 8);
        bit_index += 1;
    }

    key
}

pub fn decode_key<const N: u8>(key: [u8; 10]) -> Position<N> {
    let mut bit_index = 0;
    let mut pips = [0i8; 26];

    let mut x_pieces = 0;
    let mut o_pieces = 0;

    for point in (O_BAR + 1..X_BAR).rev() {
        while (key[bit_index / 8] >> (bit_index % 8)) & 1 == 1 {
            pips[point] -= 1;
            o_pieces += 1;
            bit_index += 1;
        }
        bit_index += 1;
    }
    while (key[bit_index / 8] >> (bit_index % 8)) & 1 == 1 {
        pips[O_BAR] -= 1;
        o_pieces += 1;
        bit_index += 1;
    }
    bit_index += 1;

    for pip in pips.iter_mut().take(X_BAR).skip(O_BAR + 1) {
        while (key[bit_index / 8] >> (bit_index % 8)) & 1 == 1 {
            *pip += 1;
            x_pieces += 1;
            bit_index += 1;
        }
        bit_index += 1;
    }
    while (key[bit_index / 8] >> (bit_index % 8)) & 1 == 1 {
        pips[X_BAR] += 1;
        x_pieces += 1;
        bit_index += 1;
    }

    Position {
        turn: true,
        pips,
        x_off: N - x_pieces,
        o_off: N - o_pieces,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};
    use crate::Variant;

    #[test]
    fn gnuid_string_roundtrip() {
        let ids = [
            "4HPwATDgc/ABMA",
            "jGfkASjg8wcBMA",
            "zGbiIQgxH/AAWA",
            "zGbiIYCYD3gALA",
        ];
        for id in ids {
            let pos = decode(Variant::Backgammon, id).expect("must decode gnuid");
            let encoded = encode(pos);
            assert_eq!(encoded, id);
        }
    }
}
