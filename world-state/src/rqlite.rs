use crate::{utils::curl, appconfig::CONFIG};


pub struct RQLite;

impl RQLite {
  pub fn create_tables() {
    let tx_table = format!("CREATE TABLE IF NOT EXISTS {} (trie_key TEXT PRIMARY KEY UNIQUE, trie_value TEXT NULL)", CONFIG.get::<String>("TX_KEY").unwrap());

    let receipt_table = format!("CREATE TABLE IF NOT EXISTS {} (trie_key TEXT PRIMARY KEY UNIQUE, trie_value TEXT NULL)", CONFIG.get::<String>("TX_RECEIPT_KEY").unwrap());

    let metacontract_table = format!("CREATE TABLE IF NOT EXISTS {} (trie_key TEXT PRIMARY KEY UNIQUE, trie_value TEXT NULL)", CONFIG.get::<String>("METACONTRACT_KEY").unwrap());

    let metadata_table = format!("CREATE TABLE IF NOT EXISTS {} (trie_key TEXT PRIMARY KEY UNIQUE, trie_value TEXT NULL)", CONFIG.get::<String>("METADATA_KEY").unwrap());

    let cron_table = format!("CREATE TABLE IF NOT EXISTS {} (trie_key TEXT PRIMARY KEY UNIQUE, trie_value TEXT NULL)", CONFIG.get::<String>("CRON_KEY").unwrap());

    let roots_table = "CREATE TABLE IF NOT EXISTS roots (root_key TEXT PRIMARY KEY UNIQUE, root_value TEXT NULL)";

    Self::execute(tx_table.as_str());
    Self::execute(receipt_table.as_str());
    Self::execute(metacontract_table.as_str());
    Self::execute(metadata_table.as_str());
    Self::execute(cron_table.as_str());
    Self::execute(roots_table);
  
  }

  pub fn execute(statement: &str) {
    let args = vec![
            "-s".to_string(),
            "-XPOST".to_string(),
            CONFIG.get::<String>("SQL_EXECUTE").unwrap(),
            "-H".to_string(),
            "Content-Type: application/json".to_string(),
            "-d".to_string(),
            format!(r#"["{}"]"#, statement),
        ];
    // println!("args: {:?}", args);
    curl(args);

    // println!("execute result: {:?}", result);
  }

  pub fn query(statement: &str) -> String {
    let args = vec![
            "-s".to_string(),
            "-XPOST".to_string(),
            CONFIG.get::<String>("SQL_QUERY").unwrap(),
            "-H".to_string(),
            "Content-Type: application/json".to_string(),
            "-d".to_string(),
            format!(r#"["{}"]"#, statement),
        ];

    let result = curl(args);

    // println!("query result: {:?}", result);

    result
  }
}