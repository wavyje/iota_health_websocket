

use std::str::FromStr;

use rusqlite::{Connection, Result, Error};
use sha2::digest::Reset;

/// **creates database blacklist**
/// **table "doctors"**
/// - lanr
/// - name
/// - password
/// **table blacklist**
/// - lanr
pub fn init_database() -> Result<()> {
    let conn = Connection::open("blacklist.db")?;
    let conn_doct = Connection::open("doctors.db")?;

    conn_doct.execute(
        "create table if not exists doctors (
             lanr text primary key,
             name text not null,
             password text not null
         )",
        []
    )?;
    conn.execute(
        "create table if not exists blacklist (
             lanr text primary key
         )",
        []
    )?;

    insert_doctor("DrHouse".to_string(), "324654977".to_string(), "sajd5sdf54sdf21s2f5ad".to_string())?;
    blacklist_doctor("324654977".to_string());

    Ok(())
}

#[derive(Debug)]
struct Doctor {
    lanr: String,
    name: String,                   
    password: String
}

/// use LANR (Lebenslange Arztnummer) as key
/// - name
/// - encrypted password
pub fn insert_doctor(name: String, lanr: String, password: String) -> Result<()> {
    let conn = Connection::open("doctors.db")?;

    conn.execute(
        "INSERT INTO doctors (lanr, name, password) values (?1, ?2, ?3)",
        [lanr.to_string(), name, password],
    )?;

    Ok(())
}

/// puts lanr in blacklist table
pub fn blacklist_doctor(lanr: String) -> bool {

    let conn = Connection::open("blacklist.db");

    match conn {
        Err(_e) => false,
        Ok(conn) => {

            let exec = conn.execute(
                "INSERT INTO blacklist (lanr) values (?1)",
                [lanr.to_string()],
            );

            match exec {
                Ok(u) => {println!("pimm");true},
                Err(_e) => {println!("hh");false}
            }

        }
    }
}

/// searches for lanr in table "doctors"
/// returns error if not found
pub fn search_doctor(lanr: String) -> Result<()> {
    let conn = Connection::open("doctors.db")?;

    let mut stmt = conn.prepare(
        "SELECT * from doctors d
         WHERE lanr = (?)",
    )?;


    let mut result = stmt.query_map([lanr.to_string()], |rows| {
        Ok( Doctor {
            lanr: rows.get(0)?,
            name: rows.get(1)?,
            password: rows.get(2)?
        } )
    })?;

    let row = result.next();
    let doctor;

    match row {
        Some(x) => doctor = x?,
        None => {
            doctor = Doctor {
                lanr: String::from(""),
                name: String::from(""),
                password: String::from("")
            }
        }
    }


    if doctor.lanr.eq(&lanr) {
        Ok(())
    }
    else {
        Err(Error::QueryReturnedNoRows)
    }
}

/// searches for lanr in table "blacklist"
/// returns Ok() if found
pub fn search_blacklist(lanr: String) -> Result<()> {
    let conn = Connection::open("blacklist.db")?;

    let mut stmt = conn.prepare(
        "SELECT * from blacklist d
         WHERE lanr = ?",
    )?;

    let mut result = stmt.query([lanr.to_string()])?;

    let blacklisted_lanr = result.next()?;

    match blacklisted_lanr {
        Some(s) => {
            let bl: String = s.get_unwrap(0);
            
            //if lanr on blacklist equals the lanr to check, return OK
            if bl.eq(&lanr) {
                return Ok(());
            }
            else {
                return Err(Error::QueryReturnedNoRows);
            }
        },
        None => {return Err(Error::QueryReturnedNoRows);}
    }
   
}

pub fn remove_doctor_from_blacklist(lanr: String) -> Result<()> {
    let conn = Connection::open("blacklist.db")?;

    let mut stmt = conn.prepare(
        "DELETE from blacklist
            WHERE lanr = ?",
    )?;

    let result = stmt.execute([lanr.to_string()]);

    match result {
        Ok(rows) => Ok(()),
        Err(_e) => Err(Error::InvalidParameterName(lanr.to_string()))
    }
}

pub fn remove_doctor(lanr: String) -> Result<()> {
    let conn = Connection::open("doctors.db")?;

    let mut stmt = conn.prepare(
        "DELETE from doctors
            WHERE lanr = ?",
    )?;

    let result = stmt.execute([lanr.to_string()]);

    match result {
        Ok(rows) => Ok(()),
        Err(_e) => Err(Error::InvalidParameterName(lanr.to_string()))
    }
}

/// searches for lanr in table "doctors"
/// tries to match the password
/// if matching returns Ok()
pub fn login(lanr: String, password: String) -> Result<()> {
    let conn = Connection::open("doctors.db")?;

    let mut stmt = conn.prepare(
        "SELECT * from doctors d
         WHERE lanr = ?",
    )?;

    let result = stmt.query_map([lanr.to_string()], |row| {
        Ok( Doctor {
            lanr: row.get(0)?,
            name: row.get(1)?,
            password: row.get(2)?
        }
        )
    })?;

    let mut doctor = Doctor {
        lanr: String::from(""),
        name: String::from(""),
        password: String::from("")
    };

    for row in result {
        doctor = row?;
    }

    if doctor.password.eq(&password) {
        Ok(())
    }
    else {
        Err(Error::InvalidQuery)
    }
    
    
}