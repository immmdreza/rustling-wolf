use mongodb::Client;

use crate::{model, world::village::periods::RawPeriod};
use mongodb::bson::doc;

model! {
    pub struct VillagePeriod {
        village_id: String,
        period: i32
    }
}

pub(crate) async fn get_village_period(client: &Client, village_id: &String) -> Option<RawPeriod> {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<VillagePeriod>("village_periods");

    match collection
        .find_one(doc! {"village_id": village_id}, None)
        .await
        .unwrap()
    {
        Some(vp) => Some(RawPeriod::from_num(vp.period)),
        None => None,
    }
}

pub(crate) async fn set_or_update_village_period(
    client: &Client,
    village_id: &String,
    period: &RawPeriod,
) {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<VillagePeriod>("village_periods");

    match collection
        .find_one(doc! {"village_id": village_id.clone()}, None)
        .await
        .unwrap()
    {
        Some(_) => {
            collection
                .update_one(
                    doc! {"village_id": village_id.clone()},
                    doc! {"$set": {"period": period.to_num()}},
                    None,
                )
                .await
                .unwrap();
        }
        None => {
            // Insert some documents into the "mydb.books" collection.
            collection
                .insert_one(
                    VillagePeriod::new(village_id.to_string(), period.to_num()),
                    None,
                )
                .await
                .unwrap();
        }
    }
}

pub(crate) async fn cleanup_village_period(
    client: &Client,
    village_id: &String,
) -> Result<mongodb::results::DeleteResult, mongodb::error::Error> {
    let db = client.database("rustling");
    // Get a handle to a collection in the database.
    let collection = db.collection::<VillagePeriod>("village_periods");

    collection
        .delete_one(doc! {"village_id": village_id}, None)
        .await
}
