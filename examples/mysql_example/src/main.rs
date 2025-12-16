use std::time::{SystemTime, UNIX_EPOCH};

use sqlxplus::{Crud, DbPool, QueryBuilder};
use test_models::User;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://root:1qaz!QAZ@localhost/test".to_string());

    println!("Connecting to MySQL database...");
    let pool = DbPool::connect(&database_url).await?;

    println!("Connected successfully!\n");

    // 生成唯一的时间戳用于避免重复数据
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // ========== 1. INSERT (插入) ==========
    println!("=== 1. INSERT (插入新记录) ===");
    let new_user = User {
        id: None, // 数据库自动生成
        username: Some(format!("test_user_{}", timestamp)),
        email: Some(format!("test_{}@example.com", timestamp)),
        is_del: Some(0i16),
        ..Default::default()
    };
    let inserted_id = new_user.insert(&pool).await?;
    println!("插入成功，新记录 ID: {}\n", inserted_id);

    // ========== 2. FIND_BY_ID (根据 ID 查找) ==========
    println!("=== 2. FIND_BY_ID (根据 ID 查找) ===");
    let found_user = User::find_by_id(&pool, inserted_id).await?;
    match &found_user {
        Some(user) => {
            println!(
                "找到用户: ID={:?}, username={:?}, email={:?}, is_del={:?}",
                user.id, user.username, user.email, user.is_del
            );
        }
        None => {
            println!("未找到 ID 为 {} 的用户", inserted_id);
        }
    }
    println!();

    // ========== 3. UPDATE (更新) ==========
    println!("=== 3. UPDATE & UPDATE_WITH_NONE (更新记录) ===");
    if let Some(mut user) = found_user {
        let id = user.id;

        // 3.1 Patch 语义：update，不应覆盖为 None 的字段
        println!("--- 3.1 使用 update（Patch）更新 email 和 system_type ---");
        user.email = Some(format!("updated_{}@example.com", timestamp));
        user.system_type = Some(2i16);
        user.update(&pool).await?;

        let patched_user = User::find_by_id(&pool, id.unwrap_or_default()).await?;
        if let Some(u) = patched_user {
            println!(
                "update 后: email={:?}, system_type={:?}",
                u.email, u.system_type
            );
        }

        // 3.2 Reset 语义：update_with_none，将 Option::None 重置为默认值
        println!("--- 3.2 使用 update_with_none（Reset）将 system_type 置为默认 ---");
        let mut user = User::find_by_id(&pool, id.unwrap_or_default())
            .await?
            .expect("user should exist");
        user.system_type = None;
        user.update_with_none(&pool).await?;

        let reset_user = User::find_by_id(&pool, id.unwrap_or_default()).await?;
        if let Some(u) = reset_user {
            println!(
                "update_with_none 后: email={:?}, system_type={:?}（应恢复为数据库默认值 1）",
                u.email, u.system_type
            );
        }
        println!();
    }

    // ========== 4. PAGINATE (分页查询) ==========
    println!("=== 4. PAGINATE (分页查询) ===");
    let builder = QueryBuilder::new("SELECT * FROM user WHERE 1=1")
        .and_eq("is_del", 0)
        .order_by("id", false);
    let page_result = User::paginate(&pool, builder, 1, 10).await?;
    println!("分页查询结果:");
    println!("  总记录数: {}", page_result.total);
    println!("  当前页记录数: {}", page_result.items.len());
    println!("  当前页: 1, 每页: 10");
    for (idx, user) in page_result.items.iter().enumerate() {
        println!(
            "  [{}] ID: {:?}, username: {:?}, email: {:?}, is_del: {:?}",
            idx + 1,
            user.id,
            user.username,
            user.email,
            user.is_del
        );
    }
    println!();

    // ========== 5. SOFT_DELETE (逻辑删除) ==========
    println!("=== 5. SOFT_DELETE (逻辑删除) ===");
    if let Some(user) = User::find_by_id(&pool, inserted_id).await? {
        println!("逻辑删除前: is_del = {:?}", user.is_del);
        User::soft_delete_by_id(&pool, inserted_id).await?;
        println!("逻辑删除结果: 成功");

        // 验证逻辑删除后，find_by_id 应该返回 None（因为会自动过滤已删除记录）
        let deleted_user = User::find_by_id(&pool, inserted_id).await?;
        match deleted_user {
            Some(_) => println!("警告：逻辑删除后仍能查询到记录！"),
            None => println!("正确：逻辑删除后无法通过 find_by_id 查询到记录"),
        }
    }
    println!();

    // ========== 6. 验证分页查询自动过滤逻辑删除的记录 ==========
    println!("=== 6. 验证分页查询自动过滤逻辑删除的记录 ===");
    let builder = QueryBuilder::new("SELECT * FROM user WHERE 1=1").order_by("id", false);
    let page_result = User::paginate(&pool, builder, 1, 10).await?;
    println!("分页查询结果（应该自动过滤已逻辑删除的记录）:");
    println!("  总记录数: {}", page_result.total);
    let found_deleted = page_result
        .items
        .iter()
        .any(|u| u.id == Some(inserted_id) && u.is_del == Some(0i16));
    if found_deleted {
        println!("  警告：分页查询返回了已逻辑删除的记录！");
    } else {
        println!("  正确：分页查询自动过滤了已逻辑删除的记录");
    }
    println!();

    // ========== 7. HARD_DELETE (物理删除) ==========
    println!("=== 7. HARD_DELETE (物理删除) ===");
    // 先插入一条新记录用于物理删除测试
    let test_user = User {
        id: None,
        username: Some(format!("delete_test_{}", timestamp)),
        email: Some(format!("delete_{}@example.com", timestamp)),
        is_del: Some(0i16),
        ..Default::default()
    };
    let test_id = test_user.insert(&pool).await?;
    println!("插入测试记录，ID: {}", test_id);

    User::hard_delete_by_id(&pool, test_id).await?;
    println!("物理删除结果: 成功");

    // 验证物理删除后，记录确实不存在了
    let deleted_user = User::find_by_id(&pool, test_id).await?;
    match deleted_user {
        Some(_) => println!("警告：物理删除后仍能查询到记录！"),
        None => println!("正确：物理删除后记录已不存在"),
    }
    println!();

    // ========== 8. FIND_ALL (查询所有记录) ==========
    println!("=== 8. FIND_ALL (查询所有记录 - 最多 1000 条) ===");
    println!("测试 find_all 方法，应该只返回未删除的记录");

    // 查询所有记录（不传 builder）
    let all_users = User::find_all(&pool, None).await?;
    println!("find_all(None) 返回 {} 条记录", all_users.len());
    for (idx, user) in all_users.iter().take(10).enumerate() {
        println!(
            "  [{}] ID: {:?}, username: {:?}, is_del: {:?}",
            idx + 1,
            user.id,
            user.username,
            user.is_del
        );
        if user.is_del == Some(1i16) {
            println!("    警告：不应该查询到已逻辑删除的记录！");
        }
    }
    if all_users.len() > 10 {
        println!("  ... (还有 {} 条记录未显示)", all_users.len() - 10);
    }
    println!();

    // 查询所有记录（使用 builder 添加额外条件）
    let builder = QueryBuilder::new("SELECT * FROM user WHERE 1=1").order_by("id", false);
    let all_users_with_builder = User::find_all(&pool, Some(builder)).await?;
    println!(
        "find_all(Some(builder)) 返回 {} 条记录",
        all_users_with_builder.len()
    );
    for (idx, user) in all_users_with_builder.iter().take(10).enumerate() {
        println!(
            "  [{}] ID: {:?}, username: {:?}, is_del: {:?}",
            idx + 1,
            user.id,
            user.username,
            user.is_del
        );
    }
    if all_users_with_builder.len() > 10 {
        println!(
            "  ... (还有 {} 条记录未显示)",
            all_users_with_builder.len() - 10
        );
    }
    println!();

    // ========== 9. QueryBuilder 功能测试 ==========
    println!("=== 9. QueryBuilder 功能测试 ===");

    // 测试 AND 条件
    let builder = QueryBuilder::new("SELECT * FROM user WHERE 1=1")
        .and_eq("is_del", 0)
        .and_gt("id", 0)
        .order_by("id", true);
    let users = User::find_all(&pool, Some(builder)).await?;
    println!(
        "AND 条件查询: is_del = 0 AND id > 0，返回 {} 条记录",
        users.len()
    );

    // 测试 OR 条件
    let builder = QueryBuilder::new("SELECT * FROM user WHERE 1=1")
        .and_eq("is_del", 0)
        .or_eq("id", inserted_id)
        .or_eq("id", test_id);
    let users = User::find_all(&pool, Some(builder)).await?;
    println!(
        "OR 条件查询: is_del = 0 OR id = {} OR id = {}，返回 {} 条记录",
        inserted_id,
        test_id,
        users.len()
    );

    // 测试 LIKE 查询
    let builder = QueryBuilder::new("SELECT * FROM user WHERE 1=1")
        .and_eq("is_del", 0)
        .and_like("username", "test_user");
    let users = User::find_all(&pool, Some(builder)).await?;
    println!(
        "LIKE 查询: username LIKE '%test_user%'，返回 {} 条记录",
        users.len()
    );

    // 测试 IN 查询
    let builder = QueryBuilder::new("SELECT * FROM user WHERE 1=1")
        .and_eq("is_del", 0)
        .and_in("id", vec![inserted_id, test_id]);
    let users = User::find_all(&pool, Some(builder)).await?;
    println!(
        "IN 查询: id IN ({}, {})，返回 {} 条记录",
        inserted_id,
        test_id,
        users.len()
    );

    // 测试 BETWEEN 查询
    let builder = QueryBuilder::new("SELECT * FROM user WHERE 1=1")
        .and_eq("is_del", 0)
        .and_between("id", 0, inserted_id + 100);
    let users = User::find_all(&pool, Some(builder)).await?;
    println!(
        "BETWEEN 查询: id BETWEEN 0 AND {}，返回 {} 条记录",
        inserted_id + 100,
        users.len()
    );

    // 测试条件分组
    let builder = QueryBuilder::new("SELECT * FROM user WHERE 1=1")
        .and_eq("is_del", 0)
        .and_group(|b| b.and_eq("id", inserted_id).or_eq("id", test_id));
    let users = User::find_all(&pool, Some(builder)).await?;
    println!(
        "条件分组查询: is_del = 0 AND (id = {} OR id = {})，返回 {} 条记录",
        inserted_id,
        test_id,
        users.len()
    );

    println!();

    println!("所有测试完成！");
    Ok(())
}

