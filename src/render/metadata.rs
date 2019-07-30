/*
 * render/mod.rs
 *
 * ftml - Convert Wikidot code to HTML
 * Copyright (C) 2019 Ammon Smith
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

// data structure for storing metadata
// written by not_a_seagull

use std::{
  borrow::Cow,
  collections::HashSet,
  convert::TryFrom,
  io::prelude::*,
  fmt::{Debug, Formatter},
  fs::File,
  path::Path
};

use std::fmt::Result as fmtResult;

use rusqlite::{
  //types::{FromSql, ToSql},
  Connection,
  NO_PARAMS
};

use crate::Error as ftmlError;
use crate::Result as ftmlResult;

use serde_json::Value;

lazy_static! {
  static ref USER_DATABASE: String = {
    let mut file = match File::open("../config.json") {
      Ok(f) => f,
      Err(_e) => panic!("Unable to locate confiugration file"),
    };

    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents).unwrap();
    let contents: Value = serde_json::from_str(&file_contents).unwrap();

    match &contents["sql_db_location"] {
      Value::String(s) => String::from(s),
      _ => panic!("Invalid configuration"),
    }
  };
}

#[derive(Clone)]
pub struct User<'a> {
  pub name: Cow<'a, str>,
  pub id: u64,
  pub avatar: String,
}

impl Debug for User<'_> {
  fn fmt(&self,  f: &mut Formatter<'_>) -> fmtResult {
    write!(f, "User {{ name: {}, id: {}, avatar: {} }}", self.name, self.id, self.avatar)
  }
}

#[derive(Clone)]
pub struct MetadataObject {
  pub url: String, // url (e.g. the-little-robot-that-could)
  pub title: String, // title (e.g. The Little Robot that Could)
  pub rating: i32,
  pub tags: HashSet<String>,
}

impl Debug for MetadataObject {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmtResult {
   let mut tag_list = String::from("[");
   for tag in &self.tags {
     tag_list = format!("{} {},", tag_list, tag);
   }
   tag_list = format!("{}]", tag_list);

    write!(f, "MetadataObject {{ url: {}, title: {}, rating: {}, tags: {} }}", self.url,
           self.title, self.rating, tag_list)
  }
}

pub fn get_user<'a>(name: &'a str) -> ftmlResult<Option<User<'a>>> {
  let connection = Connection::open(Path::new(&format!("{}{}", *USER_DATABASE, name))).unwrap();

  let read_user_sql = format!("{}{}{}", 
                        "SELECT user_id, username, avatar FROM Users WHERE username='",
                        name,
                        "';");
  let mut stmt = match connection.prepare(&read_user_sql) {
    Ok(t) => t,
    Err(_e) => return Err(ftmlError::StaticMsg("Unable to prepare SQL"))
  };

  let mut user_iter = match stmt.query_map(NO_PARAMS, |row| Ok(User {
    name: Cow::Borrowed(name),
    id: u64::try_from(row.get::<usize, i64>(0).expect("Unable to find ID")).unwrap(),
    avatar: row.get(2).expect("Unable to find avatar"),
  })) {
    Ok(s) => s,
    Err(_) => return Err(ftmlError::StaticMsg("Row glob failed"))
  };

  match user_iter.any(|_x| true) {
    true => Ok(Some(user_iter.next().unwrap().unwrap())),
    false => Ok(None)
  }
}

