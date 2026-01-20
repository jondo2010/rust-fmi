use boomerang::prelude::*;

use crate::Phase;

#[reactor]
pub fn SolverSystem2(
    // Driven by TimeManager
    #[input] phase: Phase,
    #[input] t_comm: f64,
    #[input] dt: f64,

    // External input source (could be a ConnectionHub output bundle)
    #[input] u1_src: f64,

    // Published outputs at comm points
    #[output] y1_comm: f64,
    #[output] y2_comm: f64,

    // Solver-owned continuous states
    #[state] x1: f64,
    #[state] x2: f64,

    // Latched “current iterate” outputs (for coupling)
    #[state] y1_iter: f64,
    #[state] y2_iter: f64,

    // Latched derivatives for integration
    #[state] xdot1: f64,
    #[state] xdot2: f64,

    // Eval request token + sequencing
    #[state] req: i32,
) -> impl Reactor2 {
    // Children: two ME FMUs
    let fmu1 =
        builder.add_child_reactor(crate::fmu::FmuME1x1u1y(), "fmu1", Default::default(), false)?;
    let fmu2 =
        builder.add_child_reactor(crate::fmu::FmuME1x1u1y(), "fmu2", Default::default(), false)?;

    let eval1 = builder.add_logical_action::<()>("eval1", None)?;
    let eval2 = builder.add_logical_action::<()>("eval2", None)?;
    let integrate = builder.add_logical_action::<()>("integrate", None)?;

    // Route/apply is solver-owned in ME: compute u’s for this evaluation
    // We drive evaluation only during SOLVE phase.
    reaction! {
        phase (phase) -> eval1, y1_comm, y2_comm {
            if (phase.unwrap() == Phase::Solve) {
                // Start a solve/eval chain in later microsteps
                ctx.schedule_action(&mut eval1, (), None);
            }

            if (phase.unwrap() == Phase::Publish) {
                // Publish accepted comm-point outputs (from latest iterates)
                *y1_comm = Some(state.y1_iter);
                *y2_comm = Some(state.y2_iter);
            }
        }
    }

    // Evaluate FMU1 at (t_comm, x1, u1_src)
    reaction! {
        eval_fmu1 (eval1) t_comm, u1_src -> fmu1.t_eval, fmu1.x_eval, fmu1.u_eval, fmu1.eval_req {
            state.req += 1;

            // Present eval point to FMU1 (ports are ephemeral, so do this in same reaction that triggers eval)
            *fmu1_t_eval = t_comm.clone();
            *fmu1_x_eval = Some(state.x1);
            *fmu1_u_eval = u1_src.clone();
            *fmu1_eval_req = Some(state.req);

            // Next microstep: wait for ack by triggering eval2 only after FMU1 responds.
            // (We can’t “block”; instead we react to fmu1.eval_ack.)
        }
    }

    reaction! {
        (fmu1.eval_ack) fmu1.xdot_out, fmu1.y_eval -> eval2 {
            // Latch FMU1 results (these ports exist at this tag because this reaction is downstream of that write)
            if (*fmu1_eval_ack != Some(state.req)) {
                return;
            }

            state.xdot1 = fmu1_xdot_out.unwrap();
            state.y1_iter = fmu1_y_eval.unwrap();

            // Now evaluate FMU2, whose input u2 is wired to y1
            ctx.schedule_action(&mut eval2, (), None);
        }
    }

    // Evaluate FMU2 at (t_comm, x2, u2=y1_iter)
    reaction! {
        (eval2) t_comm -> fmu2.t_eval, fmu2.x_eval, fmu2.u_eval, fmu2.eval_req {
            state.req += 1;

            *fmu2_t_eval = t_comm.clone();
            *fmu2_x_eval = Some(state.x2);
            *fmu2_u_eval = Some(state.y1_iter);    // <-- “connection” implemented here
            *fmu2_eval_req = Some(state.req);
        }
    }

    reaction! {
        (fmu2.eval_ack) fmu2.xdot_out, fmu2.y_eval -> integrate {
            if (*fmu2_eval_ack != Some(state.req)) {
                return;
            }

            state.xdot2 = fmu2_xdot_out.unwrap();
            state.y2_iter = fmu2_y_eval.unwrap();

            // With both derivatives latched, integrate (explicit Euler)
            ctx.schedule_action(&mut integrate, (), None);
        }
    }

    reaction! {
        (integrate) dt{
            state.x1 += dt.unwrap() * state.xdot1;
            state.x2 += dt.unwrap() * state.xdot2;
        }
    }
}
