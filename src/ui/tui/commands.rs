pub enum Command {
    Quit,
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Open,
    Back,
    SearchStart,
    SearchInput(char),
    SearchApply,
    SearchCancel,
    Add,
    Edit,
    Delete,
    Confirm,
    Cancel,
    Tick, // Refer to no input until timeout
    None, // Refer to the undefined key
}
