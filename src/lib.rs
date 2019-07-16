#![allow(unused)]

use std::fs;
use std::collections::HashMap;
use yaml_rust::{YamlLoader, Yaml};
use postgres::{Connection, TlsMode};
use bcrypt::{DEFAULT_COST, hash, verify};
use postgres::params::{ConnectParams, Host};

#[derive(Clone, Debug)]
pub struct UserAccount {
    info: HashMap<String, String>
}


pub fn create_user_notifications(user_ids: Vec<i32>, notifications: Vec<i32>) -> Result<(), String> {
    let conn = match get_db_connection() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let stmt = conn.prepare("INSERT INTO users_notifications (user_id, notification_id) VALUES ($1,$2)").unwrap();

    for user_id in user_ids {
        for notification in &notifications {
            stmt.execute(&[&user_id, &notification]);
        }
    }
    Ok(())
}


pub fn create_user_roles(user_ids: Vec<i32>, roles: Vec<i32>) -> Result<(), String> {
    let conn = match get_db_connection() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let stmt = conn.prepare("INSERT INTO users_roles (user_id, role_id) VALUES ($1,$2)").unwrap();

    for user_id in user_ids {
        for role in &roles {
            stmt.execute(&[&user_id, &role]);
        }
    }
    Ok(())
}


pub fn create_user(user_accounts: HashMap<String, Vec<UserAccount>>) -> Result<(), String> {
    let conn = match get_db_connection() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let stmt = conn.prepare("INSERT INTO users (email, password) VALUES ($1, $2)").unwrap();

    for (_group, accounts) in user_accounts {
        for account in accounts {
            let email = &account.info["email"];
            let password = &hash(&account.info["password"], DEFAULT_COST).unwrap();
            stmt.execute(&[email, password]);
        }
    }

    Ok(())
}


pub fn get_inserted_user_ids(user_accounts: HashMap<String, Vec<UserAccount>>) -> Result<Vec<i32>, String> {
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

    for row in &conn.query(sql.as_str(), &[]).unwrap() {
        let id: i32 = row.get(0);
        user_ids.push(id);
    }

    Ok(user_ids)
}


pub fn get_roles() -> Result<Vec<i32>, String> {
    let conn = match get_db_connection() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let mut roles = Vec::new();

    for row in &conn.query("SELECT id FROM roles", &[]).unwrap() {
        let id: i32 = row.get(0);
        roles.push(id);
    }

    Ok(roles)
}


pub fn get_notifications() -> Result<Vec<i32>, String> {
    let conn = match get_db_connection() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let mut notifications = Vec::new();

    for row in &conn.query("SELECT id FROM notifications", &[]).unwrap() {
        let id: i32 = row.get(0);
        notifications.push(id);
    }

    Ok(notifications)
}


pub fn load_user_accounts() -> Result<HashMap<String, Vec<UserAccount>>, String> {
    let yaml = match load_yaml() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let yaml = &yaml[0];
    let mut account_group: HashMap<String, Vec<UserAccount>> = HashMap::new();

    for (key, _) in yaml["accounts"].as_hash().unwrap() {
        let mut user_accounts = Vec::new();

        for account in &yaml["accounts"][key.as_str().unwrap()].as_vec() {
            let mut user_account = UserAccount { info: HashMap::new() };

            for account in account.to_vec() {
                for (key, val) in account.as_hash().unwrap() {
                    user_account.info.insert(key.as_str().unwrap().to_string(), val.as_str().unwrap().to_string());
                }

                user_accounts.push(user_account.clone());
            }
        }

        account_group.insert(key.as_str().unwrap().to_string(), user_accounts);
    }

    Ok(account_group)
}


pub fn get_db_connection() -> Result<Connection, String> {
    let db_info = match load_db_info() {
        Ok(v) => v,
        Err(e) => panic!(e)
    };

    let mut connection_builder = ConnectParams::builder();
    connection_builder.user(db_info["user"].as_str(), Some(db_info["pass"].as_str()));
    connection_builder.port(db_info["port"].parse().unwrap());
    connection_builder.database(db_info["name"].as_str());

    let connection_str = connection_builder.build(Host::Tcp(db_info["host"].clone()));
    let conn = match Connection::connect(connection_str, TlsMode::None) {
        Ok(v) => v,
        Err(e) => return Err(e.to_string())
    };

    Ok(conn)
}


pub fn load_db_info() -> Result<HashMap<String, String>, String> {
    let yaml = match load_yaml() {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    let yaml = &yaml[0];
    let mut database_info: HashMap<String, String> = HashMap::new();

    for (key, val) in yaml["database"].as_hash().unwrap() {
        database_info.insert(key.as_str().unwrap().to_string(), val.as_str().unwrap().to_string());
    }

    return Ok(database_info);
}


pub fn load_yaml() -> Result<Vec<Yaml>, String> {
    let yaml = fs::read_to_string("./migration.yml").expect("Cannot load yml file");
    let docs = YamlLoader::load_from_str(yaml.as_str()).unwrap();
    return Ok(docs);
}