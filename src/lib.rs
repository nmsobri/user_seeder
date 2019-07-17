#![allow(unused)]

use std::error::Error;
use std::{fs, process, fmt};
use std::collections::HashMap;
use yaml_rust::{YamlLoader, Yaml};
use postgres::{Connection, TlsMode};
use bcrypt::{DEFAULT_COST, hash, verify};
use postgres::params::{ConnectParams, Host};

type BoxResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Debug)]
pub struct UserAccount {
    info: HashMap<String, String>
}


#[derive(Debug)]
struct MyError {
    details: String
}


impl MyError {
    fn new(msg: &str) -> MyError {
        MyError { details: msg.to_string() }
    }
}


impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}


impl Error for MyError {
    fn description(&self) -> &str {
        &self.details
    }
}


pub fn create_user_notifications(user_ids: Vec<i32>, notifications: Vec<i32>) -> BoxResult<()> {
    let conn = match get_db_connection() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let stmt = conn.prepare("INSERT INTO users_notifications (user_id, notification_id) VALUES ($1,$2)")?;

    for user_id in user_ids {
        for notification in &notifications {
            stmt.execute(&[&user_id, &notification]);
        }
    }
    Ok(())
}


pub fn create_user_roles(user_ids: Vec<i32>, roles: Vec<i32>) -> BoxResult<()> {
    let conn = match get_db_connection() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let stmt = conn.prepare("INSERT INTO users_roles (user_id, role_id) VALUES ($1,$2)")?;

    for user_id in user_ids {
        for role in &roles {
            stmt.execute(&[&user_id, &role]);
        }
    }
    Ok(())
}


pub fn create_user(user_accounts: HashMap<String, Vec<UserAccount>>) -> BoxResult<()> {
    let conn = match get_db_connection() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let stmt = conn.prepare("INSERT INTO users (email, password) VALUES ($1, $2)")?;

    for (_group, accounts) in user_accounts {
        for account in accounts {
            let email = &account.info["email"];
            let password = &hash(&account.info["password"], DEFAULT_COST)?;
            stmt.execute(&[email, password]);
        }
    }

    Ok(())
}


pub fn get_inserted_user_ids(user_accounts: HashMap<String, Vec<UserAccount>>) -> BoxResult<Vec<i32>> {
    let mut user_emails = Vec::new();

    for (_group, accounts) in user_accounts {
        for account in accounts {
            user_emails.push(account.info["email"].clone())
        }
    }

    let conn = match get_db_connection() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let mut user_ids = Vec::new();
    let user_emails_str = format!("'{}'", user_emails.join("','"));
    let sql = format!("SELECT id FROM users WHERE email IN ({})", user_emails_str);

    for row in &conn.query(sql.as_str(), &[])? {
        let id: i32 = row.get(0);
        user_ids.push(id);
    }

    Ok(user_ids)
}


pub fn get_roles() -> BoxResult<Vec<i32>> {
    let conn = match get_db_connection() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let mut roles = Vec::new();

    for row in &conn.query("SELECT id FROM roles", &[])? {
        let id: i32 = row.get(0);
        roles.push(id);
    }

    Ok(roles)
}


pub fn get_notifications() -> BoxResult<Vec<i32>> {
    let conn = match get_db_connection() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let mut notifications = Vec::new();

    for row in &conn.query("SELECT id FROM notifications", &[])? {
        let id: i32 = row.get(0);
        notifications.push(id);
    }

    Ok(notifications)
}


pub fn load_user_accounts() -> BoxResult<HashMap<String, Vec<UserAccount>>> {
    let yaml = match load_yaml() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let yaml = &yaml[0];
    let mut account_group: HashMap<String, Vec<UserAccount>> = HashMap::new();

    for (key, _) in yaml["accounts"].as_hash().ok_or("Cannot convert to hash")? {
        let mut user_accounts = Vec::new();

        for account in &yaml["accounts"][key.as_str().ok_or("Cannot convert to string")?].as_vec() {
            let mut user_account = UserAccount { info: HashMap::new() };

            for account in account.to_vec() {
                for (key, val) in account.as_hash().ok_or("Cannot convert to hash")? {
                    user_account.info.insert(key.as_str().ok_or("Cannot convert to string")?.to_string(), val.as_str().ok_or("Cannot convert to string")?.to_string());
                }

                user_accounts.push(user_account.clone());
            }
        }

        account_group.insert(key.as_str().ok_or("Cannot convert to string")?.to_string(), user_accounts);
    }

    Ok(account_group)
}


pub fn get_db_connection() -> BoxResult<Connection> {
    let db_info = match load_db_info() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let mut connection_builder = ConnectParams::builder();
    connection_builder.user(db_info["user"].as_str(), Some(db_info["pass"].as_str()));
    connection_builder.port(db_info["port"].parse()?);
    connection_builder.database(db_info["name"].as_str());

    let connection_str = connection_builder.build(Host::Tcp(db_info["host"].clone()));
    let conn = match Connection::connect(connection_str, TlsMode::None) {
        Ok(v) => v,
        Err(e) => return Err(Box::new(MyError::new(e.description())))
    };

    Ok(conn)
}


pub fn load_db_info() -> BoxResult<HashMap<String, String>> {
    let yaml = match load_yaml() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let yaml = &yaml[0];
    let mut database_info: HashMap<String, String> = HashMap::new();

    for (key, val) in yaml["database"].as_hash().ok_or("Cannot convert to hash")? {
        database_info.insert(key.as_str().ok_or("Cannot convert to string")?.to_string(), val.as_str().ok_or("Cannot convert to string")?.to_string());
    }

    return Ok(database_info);
}


pub fn load_yaml() -> BoxResult<Vec<Yaml>> {
    let yaml = fs::read_to_string("./migration.yml").map_err(|e| format!("migration.yml, {}", e.to_string()))?;
    let docs = YamlLoader::load_from_str(yaml.as_str())?;
    return Ok(docs);
}


pub fn exit_process(msg: String) -> ! {
    eprintln!("Seeding aborted: {}", msg);
    process::exit(1);
}