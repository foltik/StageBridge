use stagebridge::midi::device::launchpad_x::{types::*, *};
use stagebridge::util::pipeline::Pipeline;

pub fn clamp8() -> Pipeline<Pos, Pos> {
    Pipeline::<Pos>::new().filter(|p| {
        let Coord(x, y) = p.into();
        x >= 0 && x <= 7 &&
        y >= 0 && y <= 7
    })
}

pub fn press() -> Pipeline<Input, Pos> {
    Pipeline::<Input>::new().filter_map(|i| {
        if let Input::Press(idx, _) = i {
            Some(idx.into())
        } else {
            None
        }
    })
}
