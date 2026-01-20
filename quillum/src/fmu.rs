use boomerang::prelude::*;

/*
reactor MEFMU1x1u1y {
  // Control
  input eval_req:int;

  // Evaluation point
  input t_eval:double;
  input x_eval:double;
  input u_eval:double;

  // Response
  output eval_ack:int;
  output xdot_out:double;
  output y_eval:double;

  // Internal FMU handle + cached metadata would live here
  state initialized:bool = false;

  reaction(startup) {
    // TODO: fmi3InstantiateModelExchange, enter/exit init mode, etc.
    initialized = true;
  }

  reaction(eval_req) {
    if (!initialized) { return; }

    // TODO: real FMI calls here:
    // fmi3SetTime(handle, t_eval)
    // fmi3SetContinuousStates(handle, &x_eval)
    // fmi3SetReal(handle, inputRefs, &u_eval)
    // fmi3GetContinuousStateDerivatives(handle, &xdot_out)
    // fmi3GetReal(handle, outputRefs, &y_eval)

    // Placeholder model for bring-up:
    // dx/dt = -x + u ; y = x
    xdot_out = -x_eval + u_eval;
    y_eval = x_eval;

    eval_ack = eval_req;
  }
}
*/

/// Start minimal: one state x, one input u, one output y, one derivative xdot.
#[reactor]
pub fn FmuME1x1u1y(
    #[input] eval_req: i32,

    // Evaluation point
    #[input] t_eval: f64,
    #[input] x_eval: f64,
    #[input] u_eval: f64,

    // Response
    #[output] eval_ack: i32,
    #[output] xdot_out: f64,
    #[output] y_eval: f64,

    // Internal FMU handle + cached metadata would live here
    #[state] initialized: bool,
) -> impl Reactor2 {
    reaction! {
        (startup) {
            // TODO: fmi3InstantiateModelExchange, enter/exit init mode, etc.
            state.initialized = true;
        }
    }

    reaction! {
        (eval_req) t_eval, x_eval, u_eval -> eval_ack, xdot_out, y_eval {
            if !state.initialized {
                return;
            }

            let (t_eval, x_eval, u_eval, eval_req) = match (
                t_eval.as_ref(),
                x_eval.as_ref(),
                u_eval.as_ref(),
                eval_req.as_ref(),
            ) {
                (Some(t_eval), Some(x_eval), Some(u_eval), Some(eval_req)) => {
                    (*t_eval, *x_eval, *u_eval, *eval_req)
                }
                _ => return,
            };

            // TODO: real FMI calls here:
            // fmi3SetTime(handle, t_eval)
            // fmi3SetContinuousStates(handle, &x_eval)
            // fmi3SetReal(handle, inputRefs, &u_eval)
            // fmi3GetContinuousStateDerivatives(handle, &xdot_out)
            // fmi3GetReal(handle, outputRefs, &y_eval)

            // Placeholder model for bring-up:
            // dx/dt = -x + u ; y = x
            let _ = t_eval;
            *xdot_out = Some(-x_eval + u_eval);
            *y_eval = Some(x_eval);
            *eval_ack = Some(eval_req);
        }
    }
}
