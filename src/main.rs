use user_seeder::*;

fn main() {
    println!("Seeding started...");

    let user_accounts = match load_user_accounts() {
        Ok(v) => v,
        Err(e) => panic!(e)
    };

    let roles = match get_roles() {
        Ok(v) => v,
        Err(e) => panic!(e)
    };

    let notifications = match get_notifications() {
        Ok(v) => v,
        Err(e) => panic!(e)
    };

    if let Err(e) = create_user(user_accounts.clone()) {
        panic!(e);
    }

    let user_ids = match get_inserted_user_ids(user_accounts) {
        Ok(v) => v,
        Err(e) => panic!(e)
    };

    if let Err(e) = create_user_roles(user_ids.clone(), roles) {
        panic!(e);
    }

    if let Err(e) = create_user_notifications(user_ids, notifications) {
        panic!(e);
    }

    println!("Successfully seeded users list");
}