use user_seeder::*;

fn main() {
    println!("Seeding started...");

    let user_accounts = match load_user_accounts() {
        Ok(v) => v,
        Err(e) => exit_process(e.to_string())
    };

    let roles = match get_roles() {
        Ok(v) => v,
        Err(e) => exit_process(e.to_string())
    };

    let notifications = match get_notifications() {
        Ok(v) => v,
        Err(e) => exit_process(e.to_string())
    };

    if let Err(e) = create_user(user_accounts.clone()) {
        exit_process(e.to_string());
    }

    let user_ids = match get_inserted_user_ids(user_accounts) {
        Ok(v) => v,
        Err(e) => exit_process(e.to_string())
    };

    if let Err(e) = create_user_roles(user_ids.clone(), roles) {
        exit_process(e.to_string());
    }

    if let Err(e) = create_user_notifications(user_ids, notifications) {
        exit_process(e.to_string());
    }

    println!("Successfully seeded users list");
}