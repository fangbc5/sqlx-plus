mod database;
mod generator;
mod sql_generator;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use generator::TableInfo;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "sqlxplus-cli")]
#[command(about = "Code generator for sqlxplus")]
#[command(version)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Generate Rust model code from database
    Generate {
        /// Database URL (e.g., mysql://user:pass@localhost/db)
        #[arg(short, long)]
        database_url: String,

        /// Output directory for generated models
        #[arg(short, long, default_value = "models")]
        output: PathBuf,

        /// Specific table names to generate (if not specified, will show interactive selection)
        #[arg(short, long)]
        tables: Vec<String>,

        /// Generate all tables without prompting
        #[arg(short, long)]
        all: bool,

        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,

        /// Dry run (don't write files)
        #[arg(long)]
        dry_run: bool,

        /// Generate serde derives
        #[arg(long, default_value_t = true)]
        serde: bool,

        /// Generate CRUD derives
        #[arg(long, default_value_t = true)]
        derive_crud: bool,
    },
    /// Generate CREATE TABLE SQL from Rust model files
    Sql {
        /// Model file path(s) (Rust file with #[model(...)] struct). Can be specified multiple times.
        #[arg(short, long)]
        model: Vec<PathBuf>,

        /// Directory containing model files (will scan all .rs files)
        #[arg(short = 'D', long)]
        dir: Option<PathBuf>,

        /// Database type (mysql, postgres, sqlite)
        #[arg(short, long, default_value = "mysql")]
        database: String,

        /// Output SQL file path (if not specified, print to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Generate {
            database_url,
            output,
            tables,
            all,
            overwrite,
            dry_run,
            serde,
            derive_crud,
        } => {
            handle_generate(
                database_url,
                output,
                tables,
                all,
                overwrite,
                dry_run,
                serde,
                derive_crud,
            )
            .await
        }
        Commands::Sql {
            model,
            dir,
            database,
            output,
        } => handle_sql(model, dir, database, output),
    }
}

async fn handle_generate(
    database_url: String,
    output: PathBuf,
    tables: Vec<String>,
    all: bool,
    overwrite: bool,
    dry_run: bool,
    serde: bool,
    derive_crud: bool,
) -> Result<()> {
    println!("🚀 sqlx-plus CLI Code Generator");
    println!("📦 Database URL: {}", database_url);
    println!("📁 Output directory: {:?}", output);

    // 连接到数据库
    println!("\n🔌 Connecting to database...");
    let pool = database::DbPool::connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    let driver = pool.driver();
    println!("✅ Connected to {:?} database", driver);

    // 获取所有表
    println!("\n📋 Fetching table list...");
    let all_tables = pool.get_tables().await?;

    if all_tables.is_empty() {
        println!("⚠️  No tables found in the database");
        return Ok(());
    }

    println!("✅ Found {} table(s)", all_tables.len());

    // 确定要生成的表
    let selected_tables = if !tables.is_empty() {
        // 使用命令行指定的表
        let mut selected = Vec::new();
        for table_name in &tables {
            if all_tables.contains(table_name) {
                selected.push(table_name.clone());
            } else {
                eprintln!("⚠️  Warning: Table '{}' not found, skipping", table_name);
            }
        }
        if selected.is_empty() {
            anyhow::bail!("No valid tables specified");
        }
        selected
    } else if all {
        // 生成所有表
        all_tables.clone()
    } else {
        // 交互式选择
        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select tables to generate (use Space to select, Enter to confirm)")
            .items(&all_tables)
            .interact()
            .context("Failed to get user input")?;

        if selections.is_empty() {
            println!("❌ No tables selected");
            return Ok(());
        }

        selections
            .into_iter()
            .map(|i| all_tables[i].clone())
            .collect()
    };

    println!(
        "\n📝 Selected {} table(s) to generate:",
        selected_tables.len()
    );
    for table in &selected_tables {
        println!("   - {}", table);
    }

    // 创建输出目录
    if !dry_run {
        fs::create_dir_all(&output).context("Failed to create output directory")?;
    }

    // 生成代码
    let generator = generator::CodeGenerator::new(serde, derive_crud);

    let mut generated_tables: Vec<TableInfo> = Vec::new();

    for table_name in &selected_tables {
        println!("\n🔍 Analyzing table: {}", table_name);

        let table_info = pool
            .get_table_info(table_name)
            .await
            .with_context(|| format!("Failed to get table info for '{}'", table_name))?;

        println!("   Columns: {}", table_info.columns.len());
        if let Some(pk) = table_info.columns.iter().find(|c| c.is_pk) {
            println!("   Primary key: {}", pk.name);
        }
        if let Some(soft_delete) = table_info.detect_soft_delete_field() {
            println!("   Soft delete field: {}", soft_delete);
        }

        // 生成模型代码
        let code = generator.generate_model(&table_info)?;

        if dry_run {
            println!("\n📄 Generated code for {}:\n", table_name);
            println!("{}", code);
            generated_tables.push(table_info);
            continue;
        }

        // 写入模型文件
        let file_name = format!("{}.rs", to_snake_case(table_name));
        let file_path = output.join(&file_name);

        if file_path.exists() && !overwrite {
            eprintln!(
                "⚠️  File {:?} already exists, skipping (use --overwrite to overwrite)",
                file_path
            );
            continue;
        }

        fs::write(&file_path, &code)
            .with_context(|| format!("Failed to write file {:?}", file_path))?;

        println!("✅ Generated: {:?}", file_path);

        generated_tables.push(table_info);
    }

    // 生成 mod.rs 汇总模块
    if !generated_tables.is_empty() {
        let mod_code = generator.generate_mod_rs(&generated_tables)?;

        if dry_run {
            println!("\n📄 Generated mod.rs preview:\n{}", mod_code);
        } else {
            let mod_path = output.join("mod.rs");
            if mod_path.exists() && !overwrite {
                eprintln!(
                    "⚠️  File {:?} already exists, skipping mod.rs (use --overwrite to overwrite)",
                    mod_path
                );
            } else {
                fs::write(&mod_path, &mod_code)
                    .with_context(|| format!("Failed to write file {:?}", mod_path))?;
                println!("✅ Generated: {:?}", mod_path);
            }
        }
    }

    println!("\n✨ Code generation completed!");
    if dry_run {
        println!("   (Dry run mode - no files were written)");
    }

    Ok(())
}

fn handle_sql(
    models: Vec<PathBuf>,
    dir: Option<PathBuf>,
    database: String,
    output: Option<PathBuf>,
) -> Result<()> {
    println!("🚀 sqlx-plus CLI SQL Generator");
    println!("🗄️  Database: {}", database);

    // 验证数据库类型
    let db_lower = database.to_lowercase();
    if !matches!(db_lower.as_str(), "mysql" | "postgres" | "sqlite") {
        anyhow::bail!("Unsupported database type: {}. Supported: mysql, postgres, sqlite", database);
    }

    // 收集所有要处理的文件
    let mut model_files = Vec::new();

    // 添加命令行指定的文件
    for model in models {
        if model.is_file() {
            model_files.push(model);
        } else {
            eprintln!("⚠️  Warning: {:?} is not a file, skipping", model);
        }
    }

    // 如果指定了目录，扫描目录下的所有 .rs 文件
    if let Some(dir_path) = dir {
        if !dir_path.is_dir() {
            anyhow::bail!("Directory does not exist: {:?}", dir_path);
        }
        println!("📁 Scanning directory: {:?}", dir_path);
        let entries = fs::read_dir(&dir_path)
            .with_context(|| format!("Failed to read directory: {:?}", dir_path))?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
                model_files.push(path);
            }
        }
    }

    if model_files.is_empty() {
        anyhow::bail!("No model files found. Please specify files with -m/--model or a directory with -d/--dir");
    }

    println!("📄 Found {} model file(s):", model_files.len());
    for file in &model_files {
        println!("   - {:?}", file);
    }

    // 生成所有文件的 SQL
    println!("\n🔍 Parsing model files...");
    let mut all_sql = Vec::new();
    let mut successful_files = Vec::new();
    let mut ignored_files = Vec::new();
    let mut error_files = Vec::new();
    
    for model_file in &model_files {
        match sql_generator::SqlGenerator::generate_create_table(model_file, &db_lower) {
            Ok(sql) => {
                if !sql.trim().is_empty() {
                    all_sql.push(sql);
                    successful_files.push(model_file.clone());
                } else {
                    // 这种情况理论上不应该发生，因为 generate_create_table 会在没有 model 时返回错误
                    ignored_files.push((model_file.clone(), "Empty SQL generated".to_string()));
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                // 检查是否是"没有 model"的错误（应该忽略）
                if error_msg.contains("No model struct found") {
                    ignored_files.push((model_file.clone(), "No model struct found".to_string()));
                } else {
                    // 其他错误（解析错误等）
                    error_files.push((model_file.clone(), error_msg));
                }
            }
        }
    }

    // 输出处理结果摘要
    println!("\n📊 Processing Summary:");
    println!("   ✅ Successfully processed: {} file(s)", successful_files.len());
    println!("   ⏭️  Ignored (no model): {} file(s)", ignored_files.len());
    if !error_files.is_empty() {
        println!("   ❌ Errors: {} file(s)", error_files.len());
    }

    // 输出成功处理的文件列表
    if !successful_files.is_empty() {
        println!("\n✅ Successfully processed files:");
        for file in &successful_files {
            println!("   - {:?}", file);
        }
    }

    // 输出忽略的文件列表
    if !ignored_files.is_empty() {
        println!("\n⏭️  Ignored files (no model struct found):");
        for (file, reason) in &ignored_files {
            println!("   - {:?} ({})", file, reason);
        }
    }

    // 输出错误的文件列表
    if !error_files.is_empty() {
        println!("\n❌ Files with errors:");
        for (file, error) in &error_files {
            println!("   - {:?}", file);
            println!("     Error: {}", error);
        }
    }

    if all_sql.is_empty() {
        anyhow::bail!("No valid SQL generated from any model files");
    }

    let combined_sql = all_sql.join("\n\n");

    // 输出 SQL 到文件或标准输出
    if let Some(output_path) = output {
        fs::write(&output_path, &combined_sql)
            .with_context(|| format!("Failed to write SQL file: {:?}", output_path))?;
        println!("\n✅ Generated SQL file: {:?}", output_path);
        println!("   (Contains {} table(s) from {} file(s))", all_sql.len(), successful_files.len());
    } else {
        println!("\n📄 Generated SQL:\n");
        println!("{}", combined_sql);
    }

    Ok(())
}

/// 转换为 snake_case
fn to_snake_case(s: &str) -> String {
    s.to_lowercase()
}
