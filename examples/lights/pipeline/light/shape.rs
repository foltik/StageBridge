use stagebridge::util::pipeline::Pipeline;
use stagebridge::midi::device::launchpad_x::{types::*};

pub fn row() -> Pipeline<Pos, Pos> {
    Pipeline::<Pos>::new()
        .flat_map(move |i| {
            let Coord(_, y) = i.into();
            (0..8).map(move |x| Coord(x, y).into())
        })
}

pub fn col() -> Pipeline<Pos, Pos> {
    Pipeline::<Pos>::new()
        .flat_map(move |i| {
            let Coord(x, _) = i.into();
            (0..8).map(move |y| Coord(x, y).into())
        })
}

pub fn rect(dx: i8, dy: i8) -> Pipeline<Pos, Pos> {
    Pipeline::<Pos>::new()
        .flat_map(move |i| {
            let Coord(x0, y0) = i.into();

            let xs = x0..(x0 + dx);
            let ys = y0..(y0 + dy);

            xs.flat_map(move |x| 
                ys.clone().map(move |y| 
                    Coord(x, y).into()))
        })
}

pub fn full() -> Pipeline<Pos, Pos> {
    Pipeline::<Pos>::new()
        .flat_map(|_| (0..64).map(|i| Index(i).into()))
}