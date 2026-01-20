mod fmu;
mod hub;
mod solver;
mod time;
mod world;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Route,
    Apply,
    Solve,
    Publish,
}
