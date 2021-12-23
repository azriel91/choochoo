/// Execution profile identifier.
///
/// This is the top level namespace that should logically distinguish different
/// invocations / executions of the tasks.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Profile(String);
