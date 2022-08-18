use mongodb::{
    bson::{doc, Document},
    Client,
};

use crate::world::person::Person;

pub async fn add_person_to_village(client: &Client, village_id: String) -> Option<Person> {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<Document>("persons");

    match collection
        .insert_one(
            doc! {"village_id": village_id.clone(), "is_alive": true},
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
    let collection = db.collection::<Document>("persons");

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
    let collection = db.collection::<Document>("persons");

    collection
        .delete_many(doc! {"village_id": village_id}, None)
        .await
}
