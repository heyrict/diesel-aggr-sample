#![feature(test)]

#[macro_use]
extern crate diesel;
extern crate test;

pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[cfg(test)]
mod bench {
    use super::*;
    use crate::models::*;
    use test::Bencher;

    fn setup_database() -> PgConnection {
        static NUM_USERS: i32 = 10;
        static NUM_POSTS_PER_USER: i32 = 30;

        let mut conn = establish_connection();

        for _ in 0..NUM_USERS {
            use crate::schema::users::dsl::*;
            let user_inst = diesel::insert_into(users)
                .default_values()
                .get_result::<User>(&mut conn)
                .expect("Failed to insert into `users`");
            let user_inst_id = user_inst.id;

            for _ in 0..NUM_POSTS_PER_USER {
                use crate::schema::posts::dsl::*;
                diesel::insert_into(posts)
                    .values(user_id.eq(user_inst_id))
                    .execute(&mut conn)
                    .expect("Failed to insert into `posts`");
            }
        }

        conn
    }

    #[bench]
    fn bench_aggregation(b: &mut Bencher) {
        let mut conn = setup_database();
        let one_user: User = {
            use crate::schema::users::dsl::*;
            users.first(&mut conn).expect("Unable to fetch one user")
        };
        b.iter(|| {
            use crate::schema::posts::dsl::*;
            posts
                .filter(user_id.eq(one_user.id))
                .count()
                .get_result::<i64>(&mut conn)
                .expect("Unable to get post count");
        })
    }
}
