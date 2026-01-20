//! # ConnectionHub
//!
//! A mapping from FMU outputs → FMU inputs evaluated at the same solver iterate
//!
//! In CS
//! ConnectionHub:
//! 	•	routes port values at communication points
//!
//! In ME
//! ConnectionHub becomes:
//! 	•	static metadata + index maps
//! 	•	used by solver to assemble U and scatter Y
//!
//! You can:
//! 	•	still model it as a reactor for symmetry
//! 	•	but it does not sit on the critical evaluation path
//!
