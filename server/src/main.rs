#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;

use chrono::prelude::*;
use diesel::prelude::*;
use diesel::{insert_into, update};
use jian_ai_server::schema::photos;
use rocket::response::Debug;
use rocket::*;
use rocket_contrib::json::Json as RJson;
use rocket_contrib::serve::StaticFiles;
use std::fs::create_dir_all;
use std::path::PathBuf;
//use jian_ai_server::schema::names;

#[database("jian_ai")]
struct DbConn(diesel::SqliteConnection);

#[derive(Queryable, Insertable)]
struct Photo {
    datetime: Option<NaiveDateTime>,
    filename: String,
    camera_id: String,
    food_weight: i16,
    name: Option<String>,
}

#[post("/new_image?<camera_id>&<food_weight>", data = "<data>")]
fn new_image(
    db: DbConn,
    camera_id: String,
    food_weight: i16,
    data: Data,
) -> Result<(), Debug<Box<dyn std::error::Error>>> {
    use jian_ai_server::schema::photos::dsl as photo;
    let utc: DateTime<Utc> = Utc::now();
    let filename = format!("{}-{}.jpg", camera_id, utc.format("%Y%m%d_%H%M%S_%f"));
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "pics", &filename]
        .iter()
        .collect();
    eprintln!("{}", path.as_path().display());
    data.stream_to_file(path).map_err(|x| Debug(x.into()))?;
    // identify -> name
    let pic = Photo {
        datetime: None,
        filename,
        camera_id,
        food_weight,
        name: None,
    };
    insert_into(photo::photos)
        .values(&pic)
        .execute(&*db)
        .map_err(|x| Debug(x.into()))?;
    Ok(())
}

#[get("/names")]
fn names(db: DbConn) -> Result<RJson<Vec<String>>, Debug<Box<dyn std::error::Error>>> {
    use jian_ai_server::schema::names::dsl as name;
    let vec = name::names
        .select(name::name)
        .load(&*db)
        .map_err(|x| Debug(x.into()))?;
    Ok(RJson(vec))
}

#[get("/unnamed_images")]
fn unnamed_images(db: DbConn) -> Result<RJson<Vec<String>>, Debug<Box<dyn std::error::Error>>> {
    use jian_ai_server::schema::photos::dsl as photo;
    let vec = photo::photos
        .select(photo::filename)
        .filter(photo::name.is_null())
        .load(&*db)
        .map_err(|x| Debug(x.into()))?;
    Ok(RJson(vec))
}

#[post("/name_image?<photo_filename>&<name>")]
fn name_image(
    db: DbConn,
    photo_filename: String,
    name: String,
) -> Result<(), Debug<Box<dyn std::error::Error>>> {
    use jian_ai_server::schema::photos::dsl as photo;
    update(photo::photos.filter(photo::filename.eq(photo_filename)))
        .set(photos::name.eq(Some(name)))
        .execute(&*db)
        .map_err(|x| Debug(x.into()))?;
    Ok(())
}

#[post("/new_names?<names>")]
fn new_names(db: DbConn, names: String) -> Result<(), Debug<Box<dyn std::error::Error>>> {
    use jian_ai_server::schema::names::dsl as name;
    names
        .split(',')
        .map(|name| {
            insert_into(name::names)
                .values(name::name.eq(name))
                .execute(&*db)
        })
        .collect::<QueryResult<Vec<usize>>>()
        .map_err(|x| Debug(x.into()))?;
    Ok(())
}

#[derive(QueryableByName)]
struct Useless {
    #[sql_type = "diesel::sql_types::Text"]
    _ret: String,
}

#[get("/init")]
fn db_init(db: DbConn) -> Result<String, Debug<Box<dyn std::error::Error>>> {
    // let confs = ["PRAGMA journal_mode = WAL", "PRAGMA synchronous = NORMAL", "PRAGMA foreign_keys = ON", "PRAGMA busy_timeout = 15"];
    // for conf in confs {
    //     let _x = diesel::sql_query(conf).load::<Useless>(&*db);
    // }
    // Ok(())
    Ok(["journal_mode", "busy_timeout", "foreign_keys"]
        .iter()
        .map(|t| {
            let ret: Result<Vec<String>, diesel::result::Error> =
                diesel::dsl::sql::<diesel::sql_types::Text>(&format!("select * from pragma_{}", t))
                    .load(&*db);
            ret.unwrap().first().unwrap().clone()
        })
        .collect::<Vec<String>>()
        .join("..."))
}

// #[get("/<file..>")]
// fn pics(file: PathBuf) -> Result<rocket::response::NamedFile, Box<dyn std::error::Error>> {
//     let content = rocket::response::NamedFile::open([env!("CARGO_MANIFEST_DIR"), "pics"].iter().collect::<PathBuf>().join(file))?;
//     Ok(content)
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    create_dir_all(
        [env!("CARGO_MANIFEST_DIR"), "pics"]
            .iter()
            .collect::<PathBuf>(),
    )?;
    rocket::ignite()
        .attach(DbConn::fairing())
        .mount("/db", routes![db_init])
        .mount(
            "/apis",
            routes![new_image, names, unnamed_images, name_image, new_names],
        )
        .mount(
            "/pics",
            StaticFiles::from(concat!(env!("CARGO_MANIFEST_DIR"), "/pics")),
        )
        .mount(
            "/",
            YewFiles {
                root: [env!("CARGO_MANIFEST_DIR"), "webpages", "static"]
                    .iter()
                    .collect::<PathBuf>(),
                default_page: ["index.html"].iter().collect::<PathBuf>(),
                rank: 100,
            },
        ) // StaticFiles::from([env!("CARGO_MANIFEST_DIR"), "webpages", "static"].iter().collect::<PathBuf>()))
        .launch();

    Ok(())
}

use rocket::handler::Outcome;
use rocket::http::uri::Segments;
use rocket::http::{Method, Status};
use rocket::response::NamedFile;

#[derive(Clone)]
pub struct YewFiles {
    root: PathBuf,
    default_page: PathBuf,
    rank: isize,
}

type Routes = Vec<Route>;
impl From<YewFiles> for Routes {
    fn from(yf: YewFiles) -> Self {
        vec![Route::ranked(yf.rank, Method::Get, "/<path..>", yf)]
    }
}

impl Handler for YewFiles {
    fn handle<'r>(&self, req: &'r Request<'_>, _data: Data) -> Outcome<'r> {
        if let Some(Ok(segs)) = req.get_segments::<Segments>(0) {
            if let Ok(path) = segs.into_path_buf(false) {
                let full_path = self.root.join(path);
                if full_path.is_file() {
                    Outcome::from(req, NamedFile::open(&full_path).ok())
                } else {
                    Outcome::from(
                        req,
                        NamedFile::open(&self.root.join(&self.default_page)).ok(),
                    )
                }
            } else {
                Outcome::Failure(Status::NotFound)
            }
        } else {
            Outcome::Failure(Status::NotFound)
        }
    }
}
