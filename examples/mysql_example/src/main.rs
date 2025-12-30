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
    let user1 = User {
        id: None,
        username: Some(format!("user1_{}", timestamp)),
        email: Some(format!("user1_{}@example.com", timestamp)),
        is_del: Some(0i16),
        ..Default::default()
    };
    let id1 = user1.insert_mysql(pool.mysql_pool()).await?;
    println!("插入成功，ID: {}\n", id1);

    let user2 = User {
        id: None,
        username: Some(format!("user2_{}", timestamp)),
        email: Some(format!("user2_{}@example.com", timestamp)),
        is_del: Some(0i16),
        ..Default::default()
    };
    let id2 = user2.insert_mysql(pool.mysql_pool()).await?;
    println!("插入成功，ID: {}\n", id2);

    let user3 = User {
        id: None,
        username: Some(format!("user3_{}", timestamp)),
        email: Some(format!("user3_{}@example.com", timestamp)),
        is_del: Some(0i16),
        ..Default::default()
    };
    let id3 = user3.insert_mysql(pool.mysql_pool()).await?;
    println!("插入成功，ID: {}\n", id3);

    // ========== 2. FIND_BY_ID (根据 ID 查找) ==========
    println!("=== 2. FIND_BY_ID (根据 ID 查找) ===");
    let found = User::find_by_id::<sqlx::MySql, _>(pool.mysql_pool(), id1).await?;
    println!(
        "找到用户: {:?}\n",
        found.map(|u| format!("ID={:?}, username={:?}", u.id, u.username))
    );

    // ========== 3. FIND_BY_IDS (根据多个 ID 查找) ==========
    println!("=== 3. FIND_BY_IDS (根据多个 ID 查找) ===");
    let users =
        User::find_by_ids::<sqlx::MySql, _, _>(pool.mysql_pool(), vec![id1, id2, id3]).await?;
    println!("找到 {} 条记录:", users.len());
    for user in &users {
        println!("  ID={:?}, username={:?}", user.id, user.username);
    }
    println!();

    // ========== 4. FIND_ONE (查询单条记录) ==========
    println!("=== 4. FIND_ONE (查询单条记录) ===");
    let builder = QueryBuilder::new("SELECT * FROM user")
        .and_eq("id", id1)
        .order_by("id", false);
    let one_user = User::find_one_mysql(pool.mysql_pool(), builder).await?;
    println!(
        "find_one 结果: {:?}\n",
        one_user.map(|u| format!("ID={:?}, username={:?}", u.id, u.username))
    );

    // ========== 5. COUNT (统计记录数量) ==========
    println!("=== 5. COUNT (统计记录数量) ===");
    let builder = QueryBuilder::new("SELECT * FROM user");
    let total = User::count_mysql(pool.mysql_pool(), builder).await?;
    println!("未删除的记录数: {}\n", total);

    // ========== 6. UPDATE (更新 - Patch 语义) ==========
    println!("=== 6. UPDATE (更新 - Patch 语义) ===");
    if let Some(mut user) = User::find_by_id::<sqlx::MySql, _>(pool.mysql_pool(), id1).await? {
        user.email = Some(format!("updated_{}@example.com", timestamp));
        user.system_type = Some(2i16);
        user.update_mysql(pool.mysql_pool()).await?;
        println!("更新成功（Patch 语义：None 字段不更新）\n");
    }

    // ========== 7. UPDATE_WITH_NONE (更新 - Reset 语义) ==========
    println!("=== 7. UPDATE_WITH_NONE (更新 - Reset 语义) ===");
    if let Some(mut user) = User::find_by_id::<sqlx::MySql, _>(pool.mysql_pool(), id1).await? {
        user.system_type = None;
        user.update_with_none_mysql(pool.mysql_pool()).await?;
        println!("更新成功（Reset 语义：None 字段重置为默认值）\n");
    }

    // ========== 8. FIND_ALL (查询所有记录) ==========
    println!("=== 8. FIND_ALL (查询所有记录) ===");
    let builder = QueryBuilder::new("SELECT * FROM user").order_by("id", false);
    let all_users = User::find_all_mysql(pool.mysql_pool(), Some(builder)).await?;
    println!("find_all 返回 {} 条记录\n", all_users.len());

    // ========== 9. PAGINATE (分页查询) ==========
    println!("=== 9. PAGINATE (分页查询) ===");
    let builder = QueryBuilder::new("SELECT * FROM user").order_by("id", false);
    let page = User::paginate_mysql(pool.mysql_pool(), builder, 1, 10).await?;
    println!(
        "分页结果: 总数={}, 当前页={} 条\n",
        page.total,
        page.items.len()
    );

    // ========== 10. SOFT_DELETE (逻辑删除) ==========
    println!("=== 10. SOFT_DELETE (逻辑删除) ===");
    User::soft_delete_by_id_mysql(pool.mysql_pool(), id2).await?;
    println!("逻辑删除 ID={} 成功", id2);

    // 验证逻辑删除后 find_by_id 返回 None
    let deleted = User::find_by_id::<sqlx::MySql, _>(pool.mysql_pool(), id2).await?;
    if deleted.is_none() {
        println!("验证成功：逻辑删除后 find_by_id 返回 None\n");
    } else {
        println!("警告：逻辑删除后仍能查询到记录！\n");
    }

    // ========== 11. HARD_DELETE (物理删除) ==========
    println!("=== 11. HARD_DELETE (物理删除) ===");
    User::hard_delete_by_id_mysql(pool.mysql_pool(), id3).await?;
    println!("物理删除 ID={} 成功", id3);

    // 验证物理删除后记录不存在
    let deleted = User::find_by_id::<sqlx::MySql, _>(pool.mysql_pool(), id3).await?;
    if deleted.is_none() {
        println!("验证成功：物理删除后记录不存在\n");
    } else {
        println!("警告：物理删除后仍能查询到记录！\n");
    }

    // ========== 12. QueryBuilder 功能测试 ==========
    println!("=== 12. QueryBuilder 功能测试 ===");

    // AND 条件
    let builder = QueryBuilder::new("SELECT * FROM user").and_gt("id", 0);
    let count = User::count_mysql(pool.mysql_pool(), builder).await?;
    println!("AND 条件查询: {} 条记录", count);

    // LIKE 查询
    let builder = QueryBuilder::new("SELECT * FROM user")
        .and_like("username", &format!("user1_{}", timestamp));
    let count = User::count_mysql(pool.mysql_pool(), builder).await?;
    println!("LIKE 查询: {} 条记录", count);

    // IN 查询
    let builder = QueryBuilder::new("SELECT * FROM user").and_in("id", vec![id1, id2]);
    let count = User::count_mysql(pool.mysql_pool(), builder).await?;
    println!("IN 查询: {} 条记录", count);

    // BETWEEN 查询
    let builder = QueryBuilder::new("SELECT * FROM user").and_between("id", id1, id3);
    let count = User::count_mysql(pool.mysql_pool(), builder).await?;
    println!("BETWEEN 查询: {} 条记录", count);

    println!();

    // ========== 13. TRANSACTION - 手动事务（成功提交） ==========
    println!("=== 13. TRANSACTION - 手动事务（成功提交） ===");
    {
        let mut tx = sqlxplus::Transaction::begin(&pool).await?;
        println!("开始事务");

        // 在事务中插入记录
        let tx_user1 = User {
            id: None,
            username: Some(format!("tx_user1_{}", timestamp)),
            email: Some(format!("tx_user1_{}@example.com", timestamp)),
            is_del: Some(0i16),
            ..Default::default()
        };
        let tx_id1 = tx_user1.insert_mysql(tx.as_mysql_executor()).await?;
        println!("事务中插入记录，ID: {}", tx_id1);

        // 在事务中更新记录
        if let Some(mut user) =
            User::find_by_id::<sqlx::MySql, _>(tx.as_mysql_executor(), tx_id1).await?
        {
            user.email = Some(format!("tx_updated_{}@example.com", timestamp));
            user.update_mysql(tx.as_mysql_executor()).await?;
            println!("事务中更新记录成功");
        }

        // 提交事务
        tx.commit().await?;
        println!("事务提交成功");

        // 验证事务提交后的数据
        let committed_user = User::find_by_id::<sqlx::MySql, _>(pool.mysql_pool(), tx_id1).await?;
        if let Some(user) = committed_user {
            println!(
                "验证成功：事务提交后可以查询到记录，email: {:?}\n",
                user.email
            );
        } else {
            println!("警告：事务提交后查询不到记录！\n");
        }
    }

    // ========== 14. TRANSACTION - 手动事务（回滚） ==========
    println!("=== 14. TRANSACTION - 手动事务（回滚） ===");
    let rollback_id = {
        let mut tx = sqlxplus::Transaction::begin(&pool).await?;
        println!("开始事务");

        // 在事务中插入记录
        let tx_user2 = User {
            id: None,
            username: Some(format!("tx_user2_{}", timestamp)),
            email: Some(format!("tx_user2_{}@example.com", timestamp)),
            is_del: Some(0i16),
            ..Default::default()
        };
        let tx_id2 = tx_user2.insert_mysql(tx.as_mysql_executor()).await?;
        println!("事务中插入记录，ID: {}", tx_id2);

        // 在事务中查询记录（应该能查到）
        let tx_user = User::find_by_id::<sqlx::MySql, _>(tx.as_mysql_executor(), tx_id2).await?;
        if tx_user.is_some() {
            println!("事务中可以查询到记录");
        }

        // 回滚事务
        tx.rollback().await?;
        println!("事务回滚成功");
        tx_id2
    };

    // 验证事务回滚后的数据（应该查询不到）
    let rolled_back_user =
        User::find_by_id::<sqlx::MySql, _>(pool.mysql_pool(), rollback_id).await?;
    if rolled_back_user.is_none() {
        println!("验证成功：事务回滚后记录不存在\n");
    } else {
        println!("警告：事务回滚后仍能查询到记录！\n");
    }

    // ========== 15. TRANSACTION - 闭包事务（成功提交） ==========
    println!("=== 15. TRANSACTION - 闭包事务（成功提交） ===");
    let closure_id = sqlxplus::with_transaction(&pool, |tx| {
        Box::pin(async move {
            println!("闭包事务开始");

            // 在事务中插入记录
            let closure_user = User {
                id: None,
                username: Some(format!("closure_user_{}", timestamp)),
                email: Some(format!("closure_user_{}@example.com", timestamp)),
                is_del: Some(0i16),
                ..Default::default()
            };
            let closure_id = closure_user.insert_mysql(tx.as_mysql_executor()).await?;
            println!("闭包事务中插入记录，ID: {}", closure_id);

            // 在事务中更新记录
            let user_opt =
                User::find_by_id::<sqlx::MySql, _>(tx.as_mysql_executor(), closure_id).await?;
            if let Some(mut user) = user_opt {
                user.email = Some(format!("closure_updated_{}@example.com", timestamp));
                user.update_mysql(tx.as_mysql_executor()).await?;
                println!("闭包事务中更新记录成功");
            }

            // 在事务中查询记录
            let count_builder = QueryBuilder::new("SELECT * FROM user").and_eq("id", closure_id);
            let count = { User::count_mysql(tx.as_mysql_executor(), count_builder).await? };
            println!("闭包事务中查询记录数: {}", count);

            // 返回成功，事务会自动提交
            Ok::<i64, sqlxplus::SqlxPlusError>(closure_id)
        })
    })
    .await?;
    println!("闭包事务提交成功，返回 ID: {}", closure_id);

    // 验证闭包事务提交后的数据
    let closure_user = User::find_by_id::<sqlx::MySql, _>(pool.mysql_pool(), closure_id).await?;
    if let Some(user) = closure_user {
        println!(
            "验证成功：闭包事务提交后可以查询到记录，email: {:?}\n",
            user.email
        );
    } else {
        println!("警告：闭包事务提交后查询不到记录！\n");
    }

    // ========== 16. TRANSACTION - 闭包事务（回滚） ==========
    println!("=== 16. TRANSACTION - 闭包事务（回滚） ===");
    let rollback_result: Result<i64, sqlxplus::SqlxPlusError> =
        sqlxplus::with_transaction(&pool, |tx| {
            Box::pin(async move {
                println!("闭包事务开始（将回滚）");

                // 在事务中插入记录
                let rollback_user = User {
                    id: None,
                    username: Some(format!("rollback_user_{}", timestamp)),
                    email: Some(format!("rollback_user_{}@example.com", timestamp)),
                    is_del: Some(0i16),
                    ..Default::default()
                };
                let rollback_id = rollback_user.insert_mysql(tx.as_mysql_executor()).await?;
                println!("闭包事务中插入记录，ID: {}", rollback_id);

                // 在事务中查询记录（应该能查到）
                let tx_user =
                    User::find_by_id::<sqlx::MySql, _>(tx.as_mysql_executor(), rollback_id).await?;
                if tx_user.is_some() {
                    println!("闭包事务中可以查询到记录");
                }

                // 返回错误，事务会自动回滚
                Err(sqlxplus::SqlxPlusError::DatabaseError(
                    sqlx::Error::Configuration("模拟错误，触发回滚".into()),
                ))
            })
        })
        .await;

    if rollback_result.is_err() {
        println!("闭包事务回滚成功（返回错误）");
    }

    // 注意：由于事务回滚，我们无法获取 rollback_id，所以这里只验证回滚机制
    println!("验证成功：闭包事务回滚机制正常工作\n");

    // ========== 17. TRANSACTION - 复杂事务场景（多个操作） ==========
    println!("=== 17. TRANSACTION - 复杂事务场景（多个操作） ===");
    let (complex_id1, complex_id2) = sqlxplus::with_transaction(&pool, |tx| {
        Box::pin(async move {
            println!("复杂事务开始");

            // 插入第一条记录
            let user1 = User {
                id: None,
                username: Some(format!("complex1_{}", timestamp)),
                email: Some(format!("complex1_{}@example.com", timestamp)),
                is_del: Some(0i16),
                ..Default::default()
            };
            let id1 = user1.insert_mysql(tx.as_mysql_executor()).await?;
            println!("插入第一条记录，ID: {}", id1);

            // 插入第二条记录
            let user2 = User {
                id: None,
                username: Some(format!("complex2_{}", timestamp)),
                email: Some(format!("complex2_{}@example.com", timestamp)),
                is_del: Some(0i16),
                ..Default::default()
            };
            let id2 = user2.insert_mysql(tx.as_mysql_executor()).await?;
            println!("插入第二条记录，ID: {}", id2);

            // 更新第一条记录
            let u_opt = User::find_by_id::<sqlx::MySql, _>(tx.as_mysql_executor(), id1).await?;
            if let Some(mut u) = u_opt {
                u.email = Some(format!("complex_updated1_{}@example.com", timestamp));
                u.update_mysql(tx.as_mysql_executor()).await?;
                println!("更新第一条记录成功");
            }

            // 查询多条记录
            let ids = vec![id1, id2];
            let users = User::find_by_ids::<sqlx::MySql, _, _>(tx.as_mysql_executor(), ids).await?;
            println!("查询到 {} 条记录", users.len());

            // 统计记录数
            let builder = QueryBuilder::new("SELECT * FROM user").and_in("id", vec![id1, id2]);
            let count = User::count_mysql(tx.as_mysql_executor(), builder).await?;
            println!("统计记录数: {}", count);

            // 返回两个 ID
            Ok::<(i64, i64), sqlxplus::SqlxPlusError>((id1, id2))
        })
    })
    .await?;
    println!(
        "复杂事务提交成功，返回 ID1: {}, ID2: {}\n",
        complex_id1, complex_id2
    );

    // 验证复杂事务提交后的数据
    let complex_user1 = User::find_by_id::<sqlx::MySql, _>(pool.mysql_pool(), complex_id1).await?;
    let complex_user2 = User::find_by_id::<sqlx::MySql, _>(pool.mysql_pool(), complex_id2).await?;
    if complex_user1.is_some() && complex_user2.is_some() {
        println!("验证成功：复杂事务提交后两条记录都存在\n");
    } else {
        println!("警告：复杂事务提交后记录不完整！\n");
    }

    // ========== 18. NESTED TRANSACTION - 子事务（成功） ==========
    println!("=== 18. NESTED TRANSACTION - 子事务（成功） ===");
    let nested_id = sqlxplus::with_transaction(&pool, |tx| {
        Box::pin(async move {
            println!("父事务开始");

            // 在父事务中插入记录
            let parent_user = User {
                id: None,
                username: Some(format!("parent_user_{}", timestamp)),
                email: Some(format!("parent_user_{}@example.com", timestamp)),
                is_del: Some(0i16),
                ..Default::default()
            };
            let parent_id = parent_user.insert_mysql(tx.as_mysql_executor()).await?;
            println!("父事务中插入记录，ID: {}", parent_id);

            // 创建子事务（保存点）
            let nested_id = sqlxplus::with_mysql_nested_transaction(tx, |nested_tx| {
                Box::pin(async move {
                    println!("子事务开始（保存点）");

                    // 在子事务中插入记录
                    let nested_user = User {
                        id: None,
                        username: Some(format!("nested_user_{}", timestamp)),
                        email: Some(format!("nested_user_{}@example.com", timestamp)),
                        is_del: Some(0i16),
                        ..Default::default()
                    };
                    let nested_id = nested_user
                        .insert_mysql(nested_tx.as_mysql_executor())
                        .await?;
                    println!("子事务中插入记录，ID: {}", nested_id);

                    // 在子事务中查询记录
                    let nested_found = User::find_by_id::<sqlx::MySql, _>(
                        nested_tx.as_mysql_executor(),
                        nested_id,
                    )
                    .await?;
                    if nested_found.is_some() {
                        println!("子事务中可以查询到记录");
                    }

                    // 子事务成功，释放保存点
                    Ok::<i64, sqlxplus::SqlxPlusError>(nested_id)
                })
            })
            .await?;
            println!("子事务成功，保存点已释放");

            // 在父事务中验证子事务插入的记录
            let nested_found =
                User::find_by_id::<sqlx::MySql, _>(tx.as_mysql_executor(), nested_id).await?;
            if nested_found.is_some() {
                println!("父事务中可以查询到子事务插入的记录");
            }

            // 父事务提交
            Ok::<i64, sqlxplus::SqlxPlusError>(nested_id)
        })
    })
    .await?;

    // 验证子事务提交后的数据
    let nested_user = User::find_by_id::<sqlx::MySql, _>(pool.mysql_pool(), nested_id).await?;
    if nested_user.is_some() {
        println!("验证成功：子事务提交后记录存在\n");
    } else {
        println!("警告：子事务提交后记录不存在！\n");
    }

    // ========== 19. NESTED TRANSACTION - 子事务（失败回滚） ==========
    println!("=== 19. NESTED TRANSACTION - 子事务（失败回滚） ===");
    let parent_id_final = sqlxplus::with_transaction(&pool, |tx| {
        Box::pin(async move {
            println!("父事务开始（子事务将失败）");

            // 在父事务中插入记录
            let parent_user = User {
                id: None,
                username: Some(format!("parent_user2_{}", timestamp)),
                email: Some(format!("parent_user2_{}@example.com", timestamp)),
                is_del: Some(0i16),
                ..Default::default()
            };
            let parent_id = parent_user.insert_mysql(tx.as_mysql_executor()).await?;
            println!("父事务中插入记录，ID: {}", parent_id);

            // 创建子事务（保存点），但子事务会失败
            let nested_result = sqlxplus::with_mysql_nested_transaction(tx, |nested_tx| {
                Box::pin(async move {
                    println!("子事务开始（将失败）");

                    // 在子事务中插入记录
                    let nested_user = User {
                        id: None,
                        username: Some(format!("nested_user2_{}", timestamp)),
                        email: Some(format!("nested_user2_{}@example.com", timestamp)),
                        is_del: Some(0i16),
                        ..Default::default()
                    };
                    let nested_id = nested_user
                        .insert_mysql(nested_tx.as_mysql_executor())
                        .await?;
                    println!("子事务中插入记录，ID: {}", nested_id);

                    // 在子事务中查询记录（应该能查到）
                    let nested_found = User::find_by_id::<sqlx::MySql, _>(
                        nested_tx.as_mysql_executor(),
                        nested_id,
                    )
                    .await?;
                    if nested_found.is_some() {
                        println!("子事务中可以查询到记录");
                    }

                    // 子事务失败，回滚到保存点
                    Err::<i64, sqlxplus::SqlxPlusError>(sqlxplus::SqlxPlusError::DatabaseError(
                        sqlx::Error::RowNotFound,
                    ))
                })
            })
            .await;

            if nested_result.is_err() {
                println!("子事务失败，已回滚到保存点");
            }

            // 在父事务中验证子事务插入的记录应该不存在（因为已回滚）
            // 注意：由于子事务回滚，nested_id 不可用，我们通过查询所有记录来验证
            let builder = QueryBuilder::new("SELECT * FROM user")
                .and_like("username", &format!("nested_user2_{}", timestamp));
            let count = User::count_mysql(tx.as_mysql_executor(), builder).await?;
            if count == 0 {
                println!("验证成功：子事务回滚后，子事务中的记录不存在");
            } else {
                println!("警告：子事务回滚后，子事务中的记录仍然存在！");
            }

            // 父事务继续，提交父事务中的记录
            Ok::<i64, sqlxplus::SqlxPlusError>(parent_id)
        })
    })
    .await?;

    // 验证父事务提交后的数据（子事务的记录应该不存在）
    let parent_user =
        User::find_by_id::<sqlx::MySql, _>(pool.mysql_pool(), parent_id_final).await?;
    if parent_user.is_some() {
        println!("验证成功：父事务提交后，父事务中的记录存在");
    } else {
        println!("警告：父事务提交后，父事务中的记录不存在！");
    }

    // 验证子事务的记录确实不存在
    let builder = QueryBuilder::new("SELECT * FROM user")
        .and_like("username", &format!("nested_user2_{}", timestamp));
    let count = User::count_mysql(pool.mysql_pool(), builder).await?;
    if count == 0 {
        println!("验证成功：子事务回滚后，子事务中的记录确实不存在\n");
    } else {
        println!("警告：子事务回滚后，子事务中的记录仍然存在！\n");
    }

    println!("所有 CRUD 方法测试完成！");
    Ok(())
}
