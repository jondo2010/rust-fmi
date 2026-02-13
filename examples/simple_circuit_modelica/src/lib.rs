// Generated implementation is emitted to OUT_DIR by build.rs.
// This keeps the source tree stable while still allowing dynamic generation.
include!(concat!(env!("OUT_DIR"), "/generated_fmu.rs"));

fmi_export::export_fmu!(SimpleCircuit);
