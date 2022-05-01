#![feature(test)]

#[macro_use]
extern crate diesel;
extern crate test;

pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> DbPool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}

#[cfg(test)]
mod bench {
    use super::*;
    use crate::models::*;
    use test::Bencher;

    fn setup_database(pool: DbPool) {
        static NUM_USERS: i32 = 5;
        static NUM_POSTS_PER_USER: i32 = 30;

        let mut conn = pool.get().expect("Failed to get connection from pool");

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
    }

    fn cleanup_database(pool: DbPool) {
        use crate::schema::*;

        let mut conn = pool.get().expect("Failed to get connection from pool");
        diesel::delete(posts::table)
            .execute(&mut conn)
            .expect("Failed to delete posts table");
        diesel::delete(users::table)
            .execute(&mut conn)
            .expect("Failed to delete users table");
    }

    #[bench]
    fn bench_aggregation(b: &mut Bencher) {
        let pool = establish_connection();
        setup_database(pool.clone());

        let mut conn = pool.get().expect("Failed to get connection from pool");
        let one_user: User = {
            use crate::schema::users::dsl::*;
            users.first(&mut conn).expect("Unable to fetch one user")
        };
        b.iter(|| {
            use crate::schema::posts::dsl::*;
            let pool = pool.clone();
            let mut conn = pool.get().expect("Failed to get connection from pool");
            posts
                .filter(user_id.eq(one_user.id))
                .count()
                .get_result::<i64>(&mut conn)
                .expect("Unable to get post count");
        });

        cleanup_database(pool);
    }
}
