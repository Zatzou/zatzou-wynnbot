use std::{collections::HashMap, error::Error, sync::Arc};

use chrono::{DateTime, TimeZone, Utc};
use serenity::prelude::*;
use tokio::sync::RwLockWriteGuard;
use tracing::info;

use crate::wynn::world::Territories;

pub async fn create_cache(data: &mut RwLockWriteGuard<'_, TypeMap>) {
    let terrinfo: CachedData<Territories> = CachedData {
        data: Territories { territories: HashMap::new() },
        time: Utc.timestamp(0, 0), //REEE
    };

    data.insert::<WorldTerritories>(Arc::new(RwLock::new(terrinfo)));
}

struct CachedData<T> {
    data: T,
    time: DateTime<Utc>,
}

pub async fn get_territories(ctx: &Context) -> Result<Territories, Box<dyn Error + Send + Sync>> {
    let data = ctx.data.read().await;

    let terrs = data.get::<WorldTerritories>();

    if let Some(terrs) = terrs {
        let terrs = terrs.read().await;

        let time = &terrs.time;
        if time.timestamp() > Utc::now().timestamp() + 60000 {
            return Ok(terrs.data.clone());
        }
    }
    info!("Getting new territory data from wynntils");
    let newterrs: Territories = reqwest::get("https://athena.wynntils.com/cache/get/territoryList")
        .await?
        .json()
        .await?;

    let newcachedata = CachedData{
        data: newterrs,
        time: Utc::now(),
    };

    if let Some(terrs) = terrs {
        let mut write = terrs.write().await;

        write.data = newcachedata.data;
        write.time = newcachedata.time;
    }

    let terrs = terrs.unwrap().read().await;

    return Ok(terrs.data.clone());
}

struct WorldTerritories;

impl TypeMapKey for WorldTerritories {
    type Value = Arc<RwLock<CachedData<Territories>>>;
}