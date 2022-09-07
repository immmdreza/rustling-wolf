use mongodb::{
    bson::{doc, oid::ObjectId},
    Client,
};

use crate::{
    model,
    world::person::{roles::Role, Person},
};

model! {
    struct PersonDoc {
        name: String,
        village_id: String,
        is_alive: bool,
        role_code: u8,
        eatable: bool
    }
}

pub async fn add_person_to_village(
    client: &Client,
    village_id: &str,
    name: &str,
) -> Option<Person> {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<PersonDoc>("persons");

    match collection
        .insert_one(
            PersonDoc::new(name.to_string(), village_id.to_string(), true, 0, false),
            None,
        )
        .await
    {
        Ok(inserted) => Some(Person::new(
            inserted.inserted_id.to_string(),
            &village_id.to_string(),
            0,
            false,
        )),
        Err(_) => None,
    }
}

pub async fn count_village_persons(client: &Client, village_id: &str) -> u64 {
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

pub async fn get_eatable_alive_persons(client: &Client, village_id: &str) -> Vec<Person> {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<PersonDoc>("persons");

    let mut persons = vec![];
    match collection
        .find(
            doc! {"village_id": village_id, "eatable": true, "is_alive": true},
            None,
        )
        .await
    {
        Ok(mut found) => {
            while found.advance().await.unwrap() {
                let curr = found.current();
                persons.push(Person::new(
                    curr.get_object_id("_id").unwrap().to_string(),
                    curr.get_str("village_id").unwrap(),
                    curr.get_i32("role_code").unwrap().try_into().unwrap(),
                    curr.get_bool("eatable").unwrap(),
                ));
            }
        }
        Err(_) => (),
    };
    persons
}

pub async fn get_all_alive_persons(client: &Client, village_id: &str) -> Vec<Person> {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<PersonDoc>("persons");

    let mut persons = vec![];
    match collection
        .find(doc! {"village_id": village_id, "is_alive": true}, None)
        .await
    {
        Ok(mut found) => {
            while found.advance().await.unwrap() {
                let curr = found.current();
                persons.push(Person::new(
                    curr.get_object_id("_id").unwrap().to_string(),
                    curr.get_str("village_id").unwrap(),
                    curr.get_i32("role_code").unwrap().try_into().unwrap(),
                    curr.get_bool("eatable").unwrap(),
                ));
            }
        }
        Err(_) => (),
    };
    persons
}

pub async fn cleanup_persons(
    client: &Client,
    village_id: &str,
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

pub async fn assign_roles(
    client: &Client,
    village_id: &str,
) -> Result<Vec<Role>, mongodb::error::Error> {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<PersonDoc>("persons");

    let mut persons = collection
        .find(doc! {"village_id": village_id}, None)
        .await?;

    // count the number of villagers in the village
    let villagers = count_village_persons(client, village_id).await;
    let roles = crate::world::person::assigner::roles(villagers.into());

    // Iterate over the results of the cursor.
    let mut counter = 0;
    while persons.advance().await? {
        let cur = persons.current();
        let id = cur.get_object_id("_id").unwrap();

        let role_code = roles[counter];
        let is_eatable = role_code.is_eatable();
        let _ = collection
            .update_one(
                doc! {"_id": id},
                doc! {"$set": {"role_code": role_code as u32, "eatable": is_eatable}},
                None,
            )
            .await?;
        counter += 1;
    }

    Ok(roles)
}

pub async fn mark_dead(client: &Client, person_id: &str) -> Result<(), mongodb::error::Error> {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<PersonDoc>("persons");

    let _ = collection
        .update_one(
            doc! {"_id": ObjectId::parse_str(person_id).unwrap() },
            doc! {"$set": {"is_alive": false}},
            None,
        )
        .await?;
    Ok(())
}

pub async fn get_person_role(client: &Client, person_id: &str) -> Role {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<PersonDoc>("persons");

    let found = collection
        .find_one(doc! {"_id": ObjectId::parse_str(person_id).unwrap()}, None)
        .await
        .unwrap();

    match found {
        Some(found) => Role::from(found.role_code),
        None => Role::NoRole,
    }
}
