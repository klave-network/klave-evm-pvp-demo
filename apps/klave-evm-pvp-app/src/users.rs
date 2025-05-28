use serde::{Deserialize, Serialize};
use klave;
use std::fmt::{self, Display, Formatter};
use serde_json::to_string;

use crate::user::User;

use crate::user::USER_TABLE;

#[derive(Serialize, Deserialize, Debug)]
pub struct Users {
    pub list: Vec<String>
}

impl Display for Users {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match serde_json::to_string(self) {
            Ok(s) => s,
            Err(e) => {
                format!("ERROR: failed to serialize Users: {}", e)
            }
        })
    }
}

impl Users {
    pub fn new() -> Users {
        Users {
            list: Vec::new()
        }
    }

    pub fn add_user(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {        
        //Check if network exists
        let user = User::get(id);
        let mut found = false;
        for u in &self.list {
            if u == &user.id {
                found = true;
                break;
            }
        }
        if found {
            return Err("user already exists".into());
        }

        self.list.push(user.id.clone());
        if let Err(e) = self.save() {
            return Err(format!("failed to save user list: {}", e).into());
        }
        Ok(())
    }

   pub fn remove_user(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        //Check if network exists
        let mut found = false;
        for u in &self.list {
            if u == id {
                found = true;
                break;
            }
        }
        if !found {
            return Err("user does not exist".into());
        }

        self.list.retain(|x| x != id);
        if let Err(e) = self.save() {
            return Err(format!("failed to save user list: {}", e).into());
        }
        Ok(())
    }

    pub fn list(&self) -> Vec<User> {
        let mut users = Vec::new();
        for id in &self.list {
            let user = match User::load(id) {
                Ok(n) => n,
                Err(e) => {
                    klave::notifier::send_string(&format!("ERROR: failed to load user: {}", e));
                    continue;
                }
            };
            users.push(user);
        }
        users
    }

    pub fn load() -> Result<Users, Box<dyn std::error::Error>> {
        match klave::ledger::get_table(USER_TABLE).get("ALL") {
            Ok(v) => {
                let user: Users = match serde_json::from_slice(&v) {
                    Ok(w) => w,
                    Err(e) => {
                        klave::notifier::send_string(&format!("ERROR: failed to deserialize user list: {}", e));
                        return Err(e.into());
                    }
                };
                Ok(user)
            },
            Err(e) => Err(e.into())
        }
    }

    pub fn get() -> Users {
        match Users::load() {
            Ok(nm) => nm,
            Err(_) => {
                Users::new()
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let serialized_users = match to_string(&self) {
            Ok(s) => s,
            Err(e) => {
                klave::notifier::send_string(&format!("ERROR: failed to serialize users list: {}", e));
                return Err(e.into());
            }
        };
        klave::ledger::get_table(USER_TABLE).set("ALL", &serialized_users.as_bytes())
    }
}
