
pub struct UciMove {
    uci: String,
    from: (usize, usize),
    to: (usize, usize)
}

impl UciMove {
    pub fn parse_move(state: String) -> UciMove {
        let chars: Vec<_> = state.chars().collect();
        let (x1, y1, x2, y2) = (
            UciMove::file(chars[0]), UciMove::rank(chars[1]),
            UciMove::file(chars[2]), UciMove::rank(chars[3]));
        UciMove {
            uci: state,
            from: (x1, y1),
            to: (x2, y2),
        }
    }

    fn file(c: char) -> usize {
        match c {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => panic!()
        }
    }

    fn rank(c: char) -> usize {
        match c {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            _ => panic!()
        }
    }
}
