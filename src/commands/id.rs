use std::collections::BTreeMap;
use std::ops::RangeInclusive;

use once_cell::sync::OnceCell;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::interactions::message_component::ButtonStyle;
use serenity::model::prelude::*;
use serenity::prelude::*;
use tokio::fs;

use crate::error::create_error_msg;
use crate::helpers::parse_command_args_raw;
use crate::wynn::items::{Identification, ItemList, Powders, StatusType, IDGROUPS};
use crate::{BOT_NAME, BOT_VERSION};

const START_CHAR: char = '󵿰';
const END_CHAR: char = '󵿱';
const SEPARATOR: char = '󵿲';

const OFFSET: i32 = 0xF5000;
const EARTH: &str = "<:earth:899381388762025984>";
const THUNDER: &str = "<:thunder:899382018452889610>";
const WATER: &str = "<:water:899382254948737077>";
const FIRE: &str = "<:fire1:899382464882044948>";
const AIR: &str = "<:air:899382632532570123>";

static ITEMDB: OnceCell<ItemList> = OnceCell::new();

#[command]
async fn id(ctx: &Context, msg: &Message) -> CommandResult {
    // read and parse the input string
    let mut temp = if let Some(item) = parse_command_args_raw(msg) {
        item.trim_start_matches(START_CHAR)
            .trim_end_matches(END_CHAR)
            .split_terminator(SEPARATOR)
    } else {
        create_error_msg(
            ctx,
            msg,
            "Invalid comamnd arguments",
            "Usage: .id (item id string)",
        )
        .await;
        return Ok(());
    };

    let name = if let Some(v) = temp.next() {
        v.to_string()
    } else {
        create_error_msg(ctx, msg, "Invalid id string", "the given string is invalid").await;
        return Ok(());
    };
    let ids = if let Some(v) = temp.next() {
        v
    } else {
        create_error_msg(ctx, msg, "Invalid id string", "the given string is invalid").await;
        return Ok(());
    }; // https://github.com/Wynntils/Wynntils/blob/development/src/main/java/com/wynntils/modules/utilities/managers/ChatItemManager.java
    let powders = temp.next();

    // Read rerolls either from the id section or the powder section
    let rerolls = if let Some(powders) = powders {
        powders.chars().last().unwrap() as i32 - OFFSET
    } else {
        ids.chars().last().unwrap() as i32 - OFFSET
    };

    // get the list of all items and cache them after the first time
    let itemlist = if let Some(db) = ITEMDB.get() {
        db
    } else {
        let itemlist: ItemList =
            serde_json::from_slice(&fs::read("./resources/item_list.json").await?)?;
        ITEMDB.set(itemlist).unwrap();
        ITEMDB.get().unwrap()
    };
    let items = &itemlist.items;

    // find the item and make sure it exists
    let item = if let Some(item) = items.iter().find(|f| f.displayName == name) {
        item
    } else {
        create_error_msg(
            ctx,
            msg,
            "Invalid item",
            "the given item was not found in the current database",
        )
        .await;
        return Ok(());
    };

    // Parse powders
    let mut powdercount = 0;
    let mut parsedpowders = Vec::new();
    if item.powderAmount > 0 {
        if let Some(powders) = powders {
            let mut powders: Vec<i32> = powders.chars().map(|c| c as i32 - OFFSET).collect();
            // Remove the reroll char from powder chars
            powders.pop();
            powders.reverse();

            for mut p in powders.into_iter() {
                while p > 0 {
                    parsedpowders.push(Powders::from_i32(p % 6 - 1));
                    powdercount += 1;

                    p /= 6;
                }
            }
        }
    }
    parsedpowders.reverse();

    let mut desc = String::new();

    if let Some(speed) = item.get_speed() {
        desc.push_str(&format!("{} Attack Speed\n\n", speed));
    }

    if let Some(damages) = &item.damageTypes {
        if let Some(d) = &damages.neutral {
            desc.push_str(&format!("Neutral Damage: {}\n", d));
        }
        if let Some(d) = &damages.fire {
            desc.push_str(&format!("{} Fire Damage: {}\n", FIRE, d));
        }
        if let Some(d) = &damages.water {
            desc.push_str(&format!("{} Water Damage: {}\n", WATER, d));
        }
        if let Some(d) = &damages.air {
            desc.push_str(&format!("{} Air Damage: {}\n", AIR, d));
        }
        if let Some(d) = &damages.thunder {
            desc.push_str(&format!("{} Thunder Damage: {}\n", THUNDER, d));
        }
        if let Some(d) = &damages.earth {
            desc.push_str(&format!("{} Earth Damage: {}\n", EARTH, d));
        }
    }
    if let Some(defenses) = &item.defenseTypes {
        if let Some(d) = &defenses.health {
            desc.push_str(&format!("❤ Health: {}\n", d));
        }
        if let Some(d) = &defenses.fire {
            desc.push_str(&format!("{} Fire Defence: {}\n", FIRE, d));
        }
        if let Some(d) = &defenses.water {
            desc.push_str(&format!("{} Water Defence: {}\n", WATER, d));
        }
        if let Some(d) = &defenses.air {
            desc.push_str(&format!("{} Air Defence: {}\n", AIR, d));
        }
        if let Some(d) = &defenses.thunder {
            desc.push_str(&format!("{} Thunder Defence: {}\n", THUNDER, d));
        }
        if let Some(d) = &defenses.earth {
            desc.push_str(&format!("{} Earth Defence: {}\n", EARTH, d));
        }
    }
    desc.push('\n');

    // requirements
    {
        let requirements = &item.requirements;
        if let Some(d) = requirements.level {
            if d != 0 {
                desc.push_str(&format!("Combat Lv. Min: {}\n", d));
            }
        }
        if let Some(d) = requirements.strength {
            if d != 0 {
                desc.push_str(&format!("Strength Min: {}\n", d));
            }
        }
        if let Some(d) = requirements.dexterity {
            if d != 0 {
                desc.push_str(&format!("Dexterity Min: {}\n", d));
            }
        }
        if let Some(d) = requirements.intelligence {
            if d != 0 {
                desc.push_str(&format!("Intelligence Min: {}\n", d));
            }
        }
        if let Some(d) = requirements.defence {
            if d != 0 {
                desc.push_str(&format!("Defence Min: {}\n", d));
            }
        }
        if let Some(d) = requirements.agility {
            if d != 0 {
                desc.push_str(&format!("Agility Min: {}\n", d));
            }
        }
        desc.push('\n');
    }

    // decode the id chars into numbers
    let ids: Vec<i32> = ids.chars().map(|c| c as i32 - OFFSET).collect();

    // sort ids so their read correctly
    let mut finalids = BTreeMap::new();
    for (id, ord) in itemlist.identificationOrder.order.iter() {
        if let Some(sid) = item.statuses.get(id) {
            finalids.insert(
                ord,
                Id {
                    id: *id,
                    idtype: sid.r#type,
                    fixed: sid.isFixed,
                    baseval: sid.baseValue,
                },
            );
        }
    }

    // ids
    let mut ididx: i32 = 0;
    let mut lastgroup: Option<RangeInclusive<i32>> = Option::None;
    let mut idprosentit: Vec<f64> = Vec::new();
    for (ord, id) in finalids.iter() {
        if let Some(group) = &lastgroup {
            if !group.contains(ord) {
                desc.push('\n')
            }
        }

        let end = match id.idtype {
            StatusType::PERCENTAGE => "%",
            StatusType::INTEGER => "",
            StatusType::TIER => "",
            StatusType::FOUR_SECONDS => "/4s",
            StatusType::THREE_SECONDS => "/3s",
        };

        if id.fixed {
            desc.push_str(&format!(
                "{}{} {}\n",
                formatnum(id.baseval),
                end,
                id.id.name()
            ));
            ididx -= 1;
        } else {
            let idstats = ids[ididx as usize];
            let encodedval = idstats / 4;
            let value;
            let prosentti;

            // wynntils api sux
            // https://github.com/Wynntils/Wynntils/blob/development/src/main/java/com/wynntils/webapi/profiles/item/objects/IdentificationContainer.java#L38
            // https://github.com/Wynntils/Wynntils/blob/development/src/main/java/com/wynntils/modules/utilities/managers/ChatItemManager.java#L267
            if i32::abs(id.baseval) > 100 {
                value = f64::round(((encodedval as f64 + 30.0) / 100.0) * id.baseval as f64) as i32
            } else {
                value = encodedval + id.min_id();
            }

            prosentti = get_percent(value, id, &itemlist.identificationOrder.inverted);

            if id.fixed || (-1 <= id.baseval && id.baseval <= 1) {
                desc.push_str(&format!("{}{} {}\n", formatnum(value), end, id.id.name()));
            } else {
                desc.push_str(&format!(
                    "{}{} {} [{:.3}%]\n",
                    formatnum(value),
                    end,
                    id.id.name(),
                    prosentti
                ));
                idprosentit.push(prosentti);
            }
        }
        ididx += 1;
        for group in IDGROUPS {
            if group.contains(ord) {
                lastgroup = Some(group);
            }
        }
    }

    desc.push('\n');

    desc.push_str(&format!(
        "[{}/{}] Powder Slots",
        powdercount, item.powderAmount
    ));

    if powdercount != 0 {
        desc.push('[');
        for p in parsedpowders {
            match p {
                Powders::EARTH => desc.push_str(EARTH),
                Powders::THUNDER => desc.push_str(THUNDER),
                Powders::WATER => desc.push_str(WATER),
                Powders::FIRE => desc.push_str(FIRE),
                Powders::AIR => desc.push_str(AIR),
            }
        }
        desc.push(']');
    }
    desc.push('\n');

    // Footer with ids
    if rerolls != 0 {
        desc.push_str(&format!(
            "{} {} [{}]",
            item.get_rarity(),
            item.get_type(),
            rerolls
        ));
    } else {
        desc.push_str(&format!("{} {}", item.get_rarity(), item.get_type()));
    }

    // make item name with id % if needed
    let mut itemname = item.displayName.clone();
    if !idprosentit.is_empty() {
        itemname.push_str(&format!(
            " [{:.3}%]",
            idprosentit.iter().sum::<f64>() / idprosentit.len() as f64
        ))
    }

    // send final message
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.color(item.get_color());
                e.title(itemname);
                e.description(desc);
                e.footer(|f| {
                    f.text(format!("{} {}", BOT_NAME, BOT_VERSION));
                    f
                });
                e
            });
            m.components(|c| {
                c.create_action_row(|ar| {
                    ar.create_button(|b| {
                        b.style(ButtonStyle::Link);
                        b.label("Open item on Wynnbuilder");
                        b.url(format!(
                            "https://wynnbuilder.github.io/item.html#{}",
                            &item.displayName.replace(" ", "%20")
                        ));
                        b.disabled(false);
                        b
                    });
                    ar
                });
                c
            });
            m
        })
        .await?;
    Ok(())
}

struct Id {
    id: Identification,
    idtype: StatusType,
    fixed: bool,
    baseval: i32,
}

impl Id {
    fn max_id(&self) -> i32 {
        if self.fixed || (-1 <= self.baseval && self.baseval <= 1) {
            self.baseval
        } else if self.baseval < 1 {
            f64::floor(self.baseval as f64 * 0.7) as i32
        } else {
            f64::floor(self.baseval as f64 * 1.3) as i32
        }
    }
    fn min_id(&self) -> i32 {
        if self.fixed || (-1 <= self.baseval && self.baseval <= 1) {
            self.baseval
        } else if self.baseval < 1 {
            f64::floor(self.baseval as f64 * 1.3) as i32
        } else {
            f64::floor(self.baseval as f64 * 0.3) as i32
        }
    }
}

fn formatnum(num: i32) -> String {
    if num > 0 {
        return format!("+{}", num);
    } else {
        return format!("{}", num);
    }
}

fn get_percent(value: i32, id: &Id, inverted: &Vec<Identification>) -> f64 {
    let percent =
        ((value as f64 - id.min_id() as f64) / (id.max_id() as f64 - id.min_id() as f64)) * 100.0;

    return if inverted.contains(&id.id) {
        100.0 - percent
    } else {
        percent
    };
}

#[command]
async fn maxid(ctx: &Context, msg: &Message) -> CommandResult {
    // read and parse the input string
    let mut temp = if let Some(item) = parse_command_args_raw(msg) {
        item.trim_start_matches(START_CHAR)
            .trim_end_matches(END_CHAR)
            .split_terminator(SEPARATOR)
    } else {
        create_error_msg(
            ctx,
            msg,
            "Invalid comamnd arguments",
            "Usage: .maxid (wynntils item id string or item name)",
        )
        .await;
        return Ok(());
    };
    let name = if let Some(v) = temp.next() {
        v.to_string()
    } else {
        create_error_msg(ctx, msg, "Invalid id string", "the given string is invalid").await;
        return Ok(());
    };

    // get the list of all items and cache them after the first time
    let itemlist = if let Some(db) = ITEMDB.get() {
        db
    } else {
        let itemlist: ItemList =
            serde_json::from_slice(&fs::read("./resources/item_list.json").await?)?;
        ITEMDB.set(itemlist).unwrap();
        ITEMDB.get().unwrap()
    };
    let items = &itemlist.items;

    // find the item from the database
    let item = if let Some(item) = items.iter().find(|f| f.displayName == name) {
        item
    } else {
        create_error_msg(
            ctx,
            msg,
            "Invalid item",
            "the given item was not found in the current database",
        )
        .await;
        return Ok(());
    };

    // sort ids so their read correctly
    let mut finalids = BTreeMap::new();
    for (id, ord) in itemlist.identificationOrder.order.iter() {
        if let Some(sid) = item.statuses.get(id) {
            finalids.insert(
                ord,
                Id {
                    id: *id,
                    idtype: sid.r#type,
                    fixed: sid.isFixed,
                    baseval: sid.baseValue,
                },
            );
        }
    }

    let mut perfids: String = String::new();

    for (_, id) in finalids.iter() {
        if id.fixed {
            continue;
        }

        let value;
        if itemlist.identificationOrder.inverted.contains(&id.id) {
            if i32::abs(id.baseval) > 100 {
                // idk if this is correct but idk if there even are items that have above -100 baseval and invert
                value = f64::round((id.max_id() as f64 * 0.0 / id.baseval as f64) - 30.0) as i32;
            } else {
                value = 0;
            }
        } else {
            if i32::abs(id.baseval) > 100 {
                value = f64::round((id.max_id() as f64 * 100.0 / id.baseval as f64) - 30.0) as i32;
            } else {
                value = id.max_id() - id.min_id();
            }
        }

        perfids.push(char::from_u32((value * 4 + OFFSET) as u32).unwrap());
    }

    let output = format!("󵿰{}󵿲{}󵀀󵿱", name, perfids);

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.content(output);
            m
        })
        .await?;

    Ok(())
}
