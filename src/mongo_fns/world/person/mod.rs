use mongodb::{bson::doc, Client};

use crate::{model, world::person::Person};

model! {
    struct PersonDoc {
        name: String,
        village_id: String,
        is_alive: bool,
        role_code: u8
    }
}

pub async fn add_person_to_village(
    client: &Client,
    village_id: &String,
    name: &str,
) -> Option<Person> {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<PersonDoc>("persons");

    match collection
        .insert_one(
            PersonDoc::new(name.to_string(), village_id.to_string(), true, 0),
            None,
        )
        .await
    {
        Ok(inserted) => Some(Person::new(inserted.inserted_id.to_string(), &village_id)),
        Err(_) => None,
    }
}

pub async fn count_village_persons(client: &Client, village_id: String) -> u64 {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<PersonDoc>("persons");

    match collection
        .count_documents(doc! {"village_id": village_id}, None)
        .await
    {
        Ok(inserted) => inserted,
        Err(_) => 0,
    }
}

pub async fn cleanup_persons(
    client: &Client,
    village_id: &String,
) -> Result<mongodb::results::DeleteResult, mongodb::error::Error> {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<PersonDoc>("persons");

    collection
        .delete_many(doc! {"village_id": village_id}, None)
        .await
}

pub async fn person_name_exists(client: &Client, village_id: &str, person_name: &str) -> bool {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<PersonDoc>("persons");

    collection
        .find_one(
            doc! {"village_id": village_id,"name": person_name.to_string()},
            None,
        )
        .await
        .unwrap()
        .is_some()
}
