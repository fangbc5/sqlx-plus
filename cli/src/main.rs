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
        /// Model file path (Rust file with #[model(...)] struct)
        #[arg(short, long)]
        model: PathBuf,

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
            database,
            output,
        } => handle_sql(model, database, output),
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

fn handle_sql(model: PathBuf, database: String, output: Option<PathBuf>) -> Result<()> {
    println!("🚀 sqlx-plus CLI SQL Generator");
    println!("📄 Model file: {:?}", model);
    println!("🗄️  Database: {}", database);

    // 验证数据库类型
    let db_lower = database.to_lowercase();
    if !matches!(db_lower.as_str(), "mysql" | "postgres" | "sqlite") {
        anyhow::bail!("Unsupported database type: {}. Supported: mysql, postgres, sqlite", database);
    }

    // 生成 SQL
    println!("\n🔍 Parsing model file...");
    let sql = sql_generator::SqlGenerator::generate_create_table(&model, &db_lower)
        .context("Failed to generate SQL")?;

    // 输出 SQL
    if let Some(output_path) = output {
        fs::write(&output_path, &sql)
            .with_context(|| format!("Failed to write SQL file: {:?}", output_path))?;
        println!("✅ Generated SQL file: {:?}", output_path);
    } else {
        println!("\n📄 Generated SQL:\n");
        println!("{}", sql);
    }

    Ok(())
}

/// 转换为 snake_case
fn to_snake_case(s: &str) -> String {
    s.to_lowercase()
}
