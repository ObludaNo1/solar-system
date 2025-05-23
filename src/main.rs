mod app;
mod run;

use run::*;

fn main() {
    pollster::block_on(run());
}
