
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ContextSwitchError {
    NullStackPointer,
    InvalidStackBounds,
    UnalignedStackTop,
    StackRegionTooSmall,
    InvalidEntryPoint,
    InvalidTaskExitLR,
}