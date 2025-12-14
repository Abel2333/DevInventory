# DevInventory 架构重构计划

## 目标
重构项目以降低耦合度，并为未来添加TUI做准备。

## 当前架构问题

### 严重问题
1. **业务逻辑嵌入CLI模块** - `cli.rs`混合了命令解析、用户交互、输出格式化和编排逻辑
2. **缺少服务/领域层** - CLI直接调用db和crypto，没有抽象
3. **展示逻辑与业务逻辑耦合** - `mask()`、`SecretRow`、println!调用遍布代码
4. **Repository模式不完整** - `reencrypt_all()`直接依赖`SecretCrypto`
5. **配置管理分散** - `resolve_db_path()`在db.rs中，CLI参数直接传递

### 当前结构（720行代码）
```
main.rs (19行)     - 入口
cli.rs (212行)     - CLI + 所有业务逻辑 + UI
db.rs (270行)      - Repository + 迁移
crypto.rs (88行)   - 加密原语（隔离良好）
keymgr.rs (131行)  - 主密钥生命周期
```

## 目标架构

```
src/
  main.rs           - 入口点（最小化）
  config.rs         - 配置管理（新增）
  domain.rs         - 领域模型（新增）
  service.rs        - 服务层/业务逻辑（新增）
  crypto.rs         - 加密原语（不变）
  db.rs             - Repository（轻微修改）
  keymgr.rs         - 密钥管理（轻微修改）
  ui/
    mod.rs          - UI模块导出（新增）
    cli.rs          - CLI界面（Clap + 格式化）（新增）
    common.rs       - 共享UI工具（mask等）（新增）
    (tui.rs)        - 未来的TUI（占位）
```

### 架构层次

```
┌─────────────────────────────────────────┐
│         UI层 (ui/)                      │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │ cli.rs  │  │ tui.rs  │  │ common  │ │
│  └─────────┘  └─────────┘  └─────────┘ │
└──────────────┬──────────────────────────┘
               │ 调用
┌──────────────▼──────────────────────────┐
│       服务层 (service.rs)               │
│     SecretService - 业务逻辑核心        │
└──────────────┬──────────────────────────┘
               │ 使用
┌──────────────▼──────────────────────────┐
│     基础设施层                          │
│  ┌─────────┐ ┌─────────┐ ┌──────────┐  │
│  │  db.rs  │ │keymgr.rs│ │crypto.rs │  │
│  └─────────┘ └─────────┘ └──────────┘  │
└─────────────────────────────────────────┘
          ▲
          │ 使用
┌─────────┴───────────┐
│   domain.rs         │
│   (领域模型)        │
└─────────────────────┘
```

### 数据流示例（添加密钥命令）

```
用户输入 "devinventory add github-token"
  │
  ▼
ui/cli.rs - 解析参数，提示用户输入
  │ 调用: service.add_secret("github-token", value, ...)
  ▼
service.rs - SecretService::add_secret()
  │ - 获取主密钥
  │ - 加密数据
  │ - 保存到数据库
  │ 返回: Result<Secret>
  ▼
ui/cli.rs - 格式化输出 "✓ Secret added"
```

## 重构步骤

### 阶段1：创建基础模块（无破坏性）

**1.1 创建 `src/domain.rs`**
定义领域模型，与DB和UI表示分离：
```rust
pub struct Secret {
    pub id: Uuid,
    pub name: String,
    pub kind: Option<String>,
    pub note: Option<String>,
    pub plaintext: Vec<u8>,  // 仅在内存中
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct SecretMetadata {
    pub id: Uuid,
    pub name: String,
    pub kind: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**1.2 创建 `src/config.rs`**
集中配置管理：
```rust
pub struct Config {
    pub db_path: PathBuf,
    pub master_key_source: MasterKeySource,
}

impl Config {
    pub fn from_env() -> Result<Self> { ... }
    pub fn resolve_db_path() -> Result<PathBuf> { ... }
}
```
- 从 `db.rs` 移动 `resolve_db_path()`
- 将CLI参数转换为配置对象

**1.3 创建 `src/service.rs`**
核心业务逻辑层：
```rust
pub struct SecretService {
    repo: Repository,
    key_provider: MasterKeyProvider,
}

impl SecretService {
    pub fn new(repo: Repository, key_provider: MasterKeyProvider) -> Self

    pub async fn add_secret(
        &self,
        name: String,
        value: Vec<u8>,
        kind: Option<String>,
        note: Option<String>,
    ) -> Result<Secret>

    pub async fn get_secret(&self, name: &str) -> Result<Secret>

    pub async fn list_secrets(&self) -> Result<Vec<SecretMetadata>>

    pub async fn search_secrets(&self, query: &str) -> Result<Vec<SecretMetadata>>

    pub async fn delete_secret(&self, name: &str) -> Result<()>

    pub async fn rotate_master_key(&self) -> Result<()>
}
```
- 封装所有 crypto + keymgr + db 的协调逻辑
- 返回领域模型，而不是数据库记录

### 阶段2：创建UI模块

**2.1 创建 `src/ui/mod.rs`**
```rust
pub mod cli;
pub mod common;

pub use cli::run_cli;
```

**2.2 创建 `src/ui/common.rs`**
从 `cli.rs` 移动展示相关工具：
```rust
pub fn mask(s: &str) -> String { ... }

pub struct SecretDisplayRow {
    // 为tabled准备的结构
}
```

**2.3 创建 `src/ui/cli.rs`**
纯CLI关注点：
- Clap命令定义
- 用户交互（rpassword提示）
- 格式化输出（Table打印）
- 调用 `SecretService` 方法
- **不包含**任何加密、数据库或密钥管理逻辑

### 阶段3：迁移业务逻辑

**3.1 将命令处理逻辑从 `cli.rs` 移到 `service.rs`**

当前 `cli.rs` 中的每个命令处理器：
```rust
// 旧方式（cli.rs）
Commands::Add { name, value, kind, note } => {
    let key = key_provider.obtain(false)?;
    let crypto = SecretCrypto::new(key);
    let plaintext = /* 提示或从value获取 */;
    let ciphertext = crypto.encrypt(...)?;
    repo.upsert_secret(...)?;
    println!("✓ Secret added");
}
```

重构为：
```rust
// 新方式（ui/cli.rs）
Commands::Add { name, value, kind, note } => {
    let plaintext = /* 提示或从value获取 */;
    let secret = service.add_secret(name, plaintext, kind, note).await?;
    println!("✓ Secret '{}' added", secret.name);
}

// 业务逻辑（service.rs）
pub async fn add_secret(...) -> Result<Secret> {
    let key = self.key_provider.obtain(false)?;
    let crypto = SecretCrypto::new(key);
    let ciphertext = crypto.encrypt(...)?;
    let record = self.repo.upsert_secret(...).await?;
    // 转换为领域模型
    Ok(Secret { ... })
}
```

**3.2 更新 `db.rs` 中的 `reencrypt_all` 解耦**

当前问题：
```rust
// db.rs:189
pub async fn reencrypt_all(&self, old: &SecretCrypto, new: &SecretCrypto) -> Result<()>
```

重构方案 - 使用函数指针：
```rust
pub async fn reencrypt_all<F>(
    &self,
    decrypt_fn: F,
    encrypt_fn: F,
) -> Result<()>
where
    F: Fn(&[u8]) -> Result<Vec<u8>>
```

或者更简单 - 在service层处理：
```rust
// service.rs
pub async fn rotate_master_key(&self) -> Result<()> {
    let old_key = /* 现有密钥 */;
    let new_key = /* 新密钥 */;
    let old_crypto = SecretCrypto::new(old_key);
    let new_crypto = SecretCrypto::new(new_key);

    // 获取所有记录
    let records = self.repo.list_secrets(None).await?;

    // 逐个重新加密
    for record in records {
        let plaintext = old_crypto.decrypt(&record.ciphertext, ...)?;
        let new_ciphertext = new_crypto.encrypt(&plaintext, ...)?;
        self.repo.update_ciphertext(record.id, new_ciphertext).await?;
    }

    Ok(())
}
```

### 阶段4：更新入口点

**4.1 简化 `main.rs`**
```rust
mod config;
mod crypto;
mod db;
mod domain;
mod keymgr;
mod service;
mod ui;

use anyhow::Result;
use config::Config;
use db::Repository;
use keymgr::MasterKeyProvider;
use service::SecretService;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // 解析配置
    let config = Config::from_env()?;

    // 初始化基础设施
    let repo = Repository::connect(&config.db_path).await?;
    repo.migrate().await?;

    let key_provider = MasterKeyProvider::new(config.master_key_source);

    // 创建服务
    let service = SecretService::new(repo, key_provider);

    // 运行UI
    ui::cli::run_cli(service).await?;

    Ok(())
}
```

### 阶段5：清理和测试

**5.1 删除旧的 `cli.rs`**
- 所有逻辑已迁移到 `ui/cli.rs` 和 `service.rs`

**5.2 添加服务层测试**
`service.rs` 现在可测试了：
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_and_get_secret() {
        let repo = Repository::connect(":memory:").await.unwrap();
        // ...
    }
}
```

**5.3 更新 `Cargo.toml` 如果需要**
检查是否有新依赖或功能标志需要添加。

## 关键决策和权衡

### 决策1：SecretRecord vs Secret
**决策**: 保留 `SecretRecord` 在 `db.rs`，创建新的 `Secret` 在 `domain.rs`
**理由**:
- `SecretRecord` 包含 `ciphertext`，是数据库表示
- `Secret` 包含 `plaintext`，是领域表示
- Repository 负责转换：`SecretRecord` ↔ `Secret`

### 决策2：服务层位置
**决策**: 单个 `service.rs` 文件，而不是 `services/` 目录
**理由**:
- 当前只有一个服务（SecretService）
- 保持简单，项目只有720行
- 如果将来需要，可以轻松拆分为 `services/secret.rs`

### 决策3：错误处理
**决策**: 继续使用 `anyhow::Result`，不创建自定义错误类型
**理由**:
- 对CLI应用足够好
- 避免过度工程
- 如果将来需要细粒度错误处理，可以逐步迁移

### 决策4：异步边界
**决策**: 服务层保持异步（因为Repository是异步的）
**理由**:
- Repository 使用 SQLite 的异步驱动
- 服务层自然应该是异步的
- UI层（CLI/TUI）可以选择如何处理异步

### 决策5：配置注入 vs 全局状态
**决策**: 通过构造函数注入依赖，避免全局状态
**理由**:
- 更易测试
- 更清晰的依赖关系
- 符合Rust最佳实践

## 为TUI做准备

重构后，添加TUI将变得简单：

```rust
// src/ui/tui.rs（未来）
pub async fn run_tui(service: SecretService) -> Result<()> {
    // 使用ratatui或cursive
    // 调用相同的 service 方法
    // 不需要重复任何业务逻辑
}

// src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // ... 初始化 service ...

    // 根据CLI参数选择UI
    match mode {
        UiMode::Cli => ui::cli::run_cli(service).await?,
        UiMode::Tui => ui::tui::run_tui(service).await?,
    }

    Ok(())
}
```

## 关键文件修改列表

### 新增文件
- `src/config.rs` - 配置管理
- `src/domain.rs` - 领域模型
- `src/service.rs` - 业务逻辑服务层
- `src/ui/mod.rs` - UI模块导出
- `src/ui/cli.rs` - CLI界面
- `src/ui/common.rs` - 共享UI工具

### 修改文件
- `src/main.rs` - 简化为配置+初始化+UI启动
- `src/db.rs` - 移除 `resolve_db_path()`，可选：重构 `reencrypt_all()`

### 删除文件
- `src/cli.rs` - 拆分到 `ui/cli.rs` 和 `service.rs`

## 验证清单

重构完成后，验证：
- [ ] 所有原有命令功能正常（init, add, get, list, search, rm, rotate）
- [ ] 现有测试通过（crypto和db的测试）
- [ ] service层有新的单元测试
- [ ] 代码行数大致相同（~750行）
- [ ] 没有 `cli.rs` 直接调用 `db.rs` 或 `crypto.rs`（通过service层）
- [ ] `ui/cli.rs` 只包含UI关注点，无加密/数据库逻辑
- [ ] 可以轻松模拟一个简单的TUI入口（即使不实现）

## 预期收益

1. **低耦合**: UI、业务逻辑、基础设施清晰分离
2. **可测试性**: service层可以独立测试
3. **可扩展性**: 添加TUI只需新增 `ui/tui.rs`，复用service层
4. **可维护性**: 每个模块职责单一，易于理解和修改
5. **类型安全**: 领域模型与数据库模型分离，减少混淆
