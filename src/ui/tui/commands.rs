// - Quit：退出（如果有未保存，弹确认）
// - MoveUp / MoveDown / PageUp / PageDown
// - Open：从列表进详情
// - Back：返回上一级（详情/编辑 → 列表）
// - SearchStart / SearchInput(char) / SearchApply / SearchCancel
// - Add / Edit / Delete / Confirm / Cancel

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
