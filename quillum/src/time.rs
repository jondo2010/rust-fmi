use boomerang::prelude::*;

use crate::Phase;

#[reactor]
pub fn TimeManager(
    dt: f64,

    #[output] phase: Phase,
    #[output] t_comm: f64, // "accepted"/communication-point time
    #[output] cycle: i32,

    #[state] t: f64,
    #[state] k: i32,
) -> impl Reactor2 {
    let wakeup = builder.add_logical_action::<()>("wakeup", None)?;
    let a_route = builder.add_logical_action::<()>("a_route", None)?;
    let a_apply = builder.add_logical_action::<()>("a_apply", None)?;
    let a_solve = builder.add_logical_action::<()>("a_solve", None)?;
    let a_publish = builder.add_logical_action::<()>("a_publish", None)?;

    reaction! {
        (startup) -> wakeup {
            ctx.schedule_action(&mut wakeup, (), None);
        }
    }

    reaction! {
        (wakeup) -> t_comm, cycle, a_route {
            *t_comm = Some(state.t);
            *cycle = Some(state.k);
            ctx.schedule_action(&mut a_route, (), None);
        }
    }

    reaction! {
      (a_route) -> phase, a_apply {
        *phase = Some(Phase::Route);
        ctx.schedule_action(&mut a_apply, (), None);
      }
    }

    reaction! {
      (a_apply) -> phase, a_solve {
        *phase = Some(Phase::Apply);
        ctx.schedule_action(&mut a_solve, (), None);
      }
    }

    reaction! {
      (a_solve) -> phase, a_publish {
        *phase = Some(Phase::Solve);
        ctx.schedule_action(&mut a_publish, (), None);
      }
    }

    reaction! {
        (a_publish) -> phase, wakeup {
            *phase = Some(Phase::Publish);

            state.t += dt;
            state.k += 1;
            ctx.schedule_action(&mut wakeup, (), Some(Duration::seconds_f64(dt)));
        }
    }
}
