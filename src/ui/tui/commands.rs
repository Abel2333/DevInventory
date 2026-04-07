#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    // App Operation
    Quit,
    Tick, // Refer to no input until timeout
    None, // Refer to the undefined key

    // Move
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,

    // Detail <-> list
    OpenDetail,
    BackToList,

    // Search
    StartSearch,
    SearchInput(char),
    SearchApply,
    SearchCancel,

    // Edit item
    StartAdd,
    StartEdit,
    StartDelete,

    // Edit Form
    FormInput(char),
    FormBackspace,
    FormNextField,
    FormPrevField,

    // Verify
    Confirm,
    Cancel,
}
