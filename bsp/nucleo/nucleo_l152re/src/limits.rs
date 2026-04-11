/// NOTE: The available task count should be comptime calculated but storing a fixed num for simplicity (fix in future)
/// upper bound on TaskId.0+1 for static slot tables (makes implementation simpler)
pub const MAX_TASKS: usize = 32;