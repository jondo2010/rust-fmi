use boomerang::prelude::*;

use crate::Phase;

#[reactor]
pub fn World() -> impl Reactor2 {
    const dt: f64 = 0.01;

    let tm = builder.add_child_reactor(
        crate::time::TimeManager(dt),
        "tm",
        Default::default(),
        false,
    )?;
    let solver = builder.add_child_reactor(
        crate::solver::SolverSystem2(),
        "solver",
        Default::default(),
        false,
    )?;
    let src = builder.add_child_reactor(ConstantReal(dt), "src", (), false)?;

    // TimeManager -> solver
    builder.connect_port(tm.phase, solver.phase, None, false)?;
    builder.connect_port(tm.t_comm, solver.t_comm, None, false)?;

    // source -> solver
    builder.connect_port(src.y, solver.u1_src, None, false)?;

    // If you want to observe:
    // solver.y1_comm, solver.y2_comm
}

#[reactor]
pub fn ConstantReal(value: f64, #[output] y: f64) -> impl Reactor2 {
    reaction! {
        (startup) -> y {
            *y = Some(value);
        }
    }
}

#[reactor]
fn World2(dt: Duration, #[output] phase_tick: Phase) -> impl Reactor2 {
    let tick = builder.add_timer("tick", TimerSpec::default().with_period(dt))?;
    let emit_phase = builder.add_logical_action::<Phase>("emit_phase", None)?;

    reaction! {
        (tick) -> emit_phase {
            ctx.schedule_action(&mut emit_phase, Phase::Route, None);
            ctx.schedule_action(&mut emit_phase, Phase::Apply, None);
            ctx.schedule_action(&mut emit_phase, Phase::Solve, None);
            ctx.schedule_action(&mut emit_phase, Phase::Publish, None);
        }
    }

    reaction! {
        (emit_phase) -> phase_tick {
            let value = ctx.get_action_value(&mut emit_phase);
            *phase_tick = value.cloned();
        }
    }
}

#[test]
fn test_world() {
    tracing_subscriber::fmt::init();
    let config = runtime::Config::default()
        .with_fast_forward(true)
        .with_timeout(Duration::milliseconds(50));

    let _ = boomerang_util::runner::build_and_test_reactor2(World(), "tester", (), config).unwrap();
}
