use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;

use crc32fast::Hasher;
use once_cell::sync::OnceCell;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::http::AttachmentType;
use serenity::model::prelude::*;
use serenity::prelude::*;
use tiny_skia::Pixmap;
use usvg::{NodeExt, Tree};

use tracing::{error, info};

use crate::error::create_error_msg;
use crate::helpers::parse_command_args;
use crate::wynn::Gather::{self, GatherSpot};
use crate::wynn::world::{Territories, Territory};
use crate::{BOT_NAME, BOT_VERSION};
use cached::proc_macro::cached;

/// Static for the image file so we don't have to load it every time
static MAPBASE: OnceCell<Pixmap> = OnceCell::new();

/// Helper function for getting the base map
fn get_mapbase() -> Result<Pixmap, Box<dyn Error + Send + Sync>> {
    let out = if let Some(pm) = MAPBASE.get() {
        pm.clone()
    } else {
        let map = tiny_skia::Pixmap::load_png("./main-map2.png")?;
        MAPBASE.set(map.clone()).unwrap();
        map
    };
    Ok(out)
}

// allow getting a new map every 30s otherwise use a cached one
#[cached(time=30, result = true)]
async fn get_map() -> Result<Territories, reqwest::Error> {
    info!("Getting new map data from wynntils");
    let terrs: Territories = reqwest::get("https://athena.wynntils.com/cache/get/territoryList")
        .await?
        .json()
        .await?;
    Ok(terrs)
}

#[command]
async fn map(ctx: &Context, msg: &Message) -> CommandResult {
    // load territory data from wynntils api
    let terrs = get_map().await?;

    // ouput image
    let mut out = get_mapbase()?;

    // all of this rendering stuff has to be in a closure because the ownership and async don't work otherwise
    {
        // create base svg
        let svg = usvg::Svg {
            size: usvg::Size::new(out.width() as f64, out.height() as f64).unwrap(),
            view_box: usvg::ViewBox {
                rect: usvg::Rect::new(0.0, 0.0, out.width() as f64, out.height() as f64).unwrap(),
                aspect: usvg::AspectRatio::default(),
            },
        };
        let svgtree = usvg::Tree::create(svg);

        // go thru all territories and render the rects for them
        for (_, terr) in terrs.territories.iter() {
            let loc = &terr.location;

            // widths
            let width = f64::abs(loc.endX - loc.startX) / 3.0;
            let height = f64::abs(loc.endZ - loc.startZ) / 3.0;

            // position calculations
            let x = if loc.startX < loc.endX {
                calc_x(loc.startX)
            } else {
                calc_x(loc.endX)
            };
            let y = if loc.startZ < loc.endZ {
                calc_z(loc.startZ)
            } else {
                calc_z(loc.endZ)
            };

            // guild color calculations
            let col = if !terr.guildColor.is_empty() {
                terr.guildColor[1..].to_owned()
            } else {
                hex::encode(guild_color(terr.guild.clone()).to_ne_bytes())[0..=5].to_owned()
            };

            // hex to rgb
            let color = colorsys::Rgb::from_hex_str(&col)?;

            // usvg stuff
            let stroke = Some(usvg::Stroke {
                paint: usvg::Paint::Color(usvg::Color::new_rgb(
                    color.red() as u8,
                    color.green() as u8,
                    color.blue() as u8,
                )),
                opacity: 1.0.into(),
                width: 2.5.into(),
                ..usvg::Stroke::default()
            });

            let fill = Some(usvg::Fill {
                paint: usvg::Paint::Color(usvg::Color::new_rgb(
                    color.red() as u8,
                    color.green() as u8,
                    color.blue() as u8,
                )),
                opacity: 0.5.into(),
                ..usvg::Fill::default()
            });

            // create the element
            svgtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
                stroke,
                fill,
                data: Rc::new(usvg::PathData::from_rect(usvg::Rect::new(
                    x, y, width, height,
                ).unwrap())),
                ..usvg::Path::default()
            }));
        }

        let mut guilds: HashMap<String, Vec<Territory>> = HashMap::new();

        for (_name, terr) in terrs.territories.into_iter() {
            if let Some(t) = guilds.get_mut(&terr.guildPrefix) {
                t.push(terr);
            } else {
                let guild = terr.guildPrefix.clone();
                let vec: Vec<Territory> = Vec::from([terr]);
                guilds.insert(guild, vec);
            };
        }

        let mut names: HashMap<String, (f64, f64)> = HashMap::new();

        for (name, terrs) in guilds.iter() {
            let mut avg_x: Vec<f64> = Vec::new();
            let mut avg_z: Vec<f64> = Vec::new();

            for area in terrs {
                let loc = &area.location;
                let x_start = if loc.startX < loc.endX {
                    (loc.startX / 3.0) - 566.333 + 1364.0
                } else {
                    (loc.endX / 3.0) - 566.333 + 1364.0
                };
                let y_start = if loc.startZ < loc.endZ {
                    (loc.startZ / 3.0) + 41.0 + 2162.0
                } else {
                    (loc.endZ / 3.0) + 41.0 + 2162.0
                };
                let x_end = if loc.startX > loc.endX {
                    (loc.startX / 3.0) - 566.333 + 1364.0
                } else {
                    (loc.endX / 3.0) - 566.333 + 1364.0
                };
                let y_end = if loc.startZ > loc.endZ {
                    (loc.startZ / 3.0) + 41.0 + 2162.0
                } else {
                    (loc.endZ / 3.0) + 41.0 + 2162.0
                };
                avg_x.push(x_end - x_start);
                avg_z.push(y_end - y_start);
            }

            names.insert(name.to_string(), (median(&avg_x), median(&avg_z)));
        }

        // render out the svg
        resvg::render(&svgtree, usvg::FitTo::Original, out.as_mut());

        // name stuff TODO: not worki
        for (name, pos) in names {
            let svege = usvg::Tree::from_str(&format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1364 2162"><text x="{}" y="{}">{}</text></svg>"#, pos.0, pos.1, name), &usvg::Options::default().to_ref())?;
            //info!("{} at {} {}", name, pos.0, pos.1);
            resvg::render(&svege, usvg::FitTo::Original, out.as_mut());
        }
    }

    //let opt: Options = Options::default();
    //resvg::render(&usvg::Tree::from_data(&std::fs::read("./map.svg").unwrap(), &opt.to_ref()).unwrap(), usvg::FitTo::Original, out.as_mut());

    // get the png
    let png_data = out.encode_png()?;

    // serenity wants a cow for whatever reason
    let cow = Cow::from(png_data);

    // construct reply message
    let msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.image("attachment://map.png");
                e.footer(|f| {
                    f.text(format!("{} {}", BOT_NAME, BOT_VERSION));
                    f
                });
                e
            });
            m.add_file(AttachmentType::Bytes {
                data: cow,
                filename: String::from("map.png"),
            });
            m
        })
        .await;

    if let Err(why) = msg {
        error!("Error sending message: {:?}", why);
    }
    Ok(())
}

/// Gets the guilds color from it's name
#[cached]
fn guild_color(name: String) -> u32 {
    // hash the guilds nane with crc32
    let mut hasher = Hasher::new();
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();
    // bitwise and it with 0xFFFFFF cuz wynntils did it like that
    hash & 0xFFFFFF
}

fn calc_x(x: f64) -> f64 {
    (x / 3.0) - 566.333 + 1364.0
}

fn calc_z(z: f64) -> f64 {
    (z / 3.0) + 41.0 + 2162.0
}

fn median(list: &[f64]) -> f64 {
    let len = list.len();
    let mid = len / 2;
    if len % 2 == 0 {
        mean(&list[(mid - 1)..(mid + 1)])
    } else {
        list[mid]
    }
}

fn mean(list: &[f64]) -> f64 {
    let sum: f64 = Iterator::sum(list.iter());
    sum / (list.len() as f64)
}

#[command]
async fn gather(ctx: &Context, msg: &Message) -> CommandResult {
    let cmd_args = parse_command_args(msg);
    
    // get resource types
    let types = if let Some(s) = cmd_args.get(1) {
        s.clone().trim().to_uppercase()
    } else {
        return gather_usage(ctx, msg).await;
    };

    // get the wanted resource
    let wanted = if let Some(s) = cmd_args.get(2) {
        s.clone().trim().to_uppercase()
    } else {
        return gather_usage(ctx, msg).await;
    };

    let spots = Gather::get_gatherspots().await?;

    let mut out = get_mapbase()?;

    {
        // create base svg
        let svg = usvg::Svg {
            size: usvg::Size::new(out.width() as f64, out.height() as f64).unwrap(),
            view_box: usvg::ViewBox {
                rect: usvg::Rect::new(0.0, 0.0, out.width() as f64, out.height() as f64).unwrap(),
                aspect: usvg::AspectRatio::default(),
            },
        };
        let mut svgtree = usvg::Tree::create(svg);

        if types.contains("W") {
            for spot in spots.woodCutting {
                if spot.r#type == wanted {
                    add_rect(spot, &mut svgtree);
                }
            }
        }
        if types.contains("M") {
            for spot in spots.mining {
                if spot.r#type == wanted {
                    add_rect(spot, &mut svgtree);
                }
            }
        }
        if types.contains("G") {
            for spot in spots.farming {
                if spot.r#type == wanted {
                    add_rect(spot, &mut svgtree);
                }
            }
        }
        if types.contains("F") {
            for spot in spots.fishing {
                if spot.r#type == wanted {
                    add_rect(spot, &mut svgtree);
                }
            }
        }

        // render svg
        resvg::render(&svgtree, usvg::FitTo::Original, out.as_mut());
    }

    // get the png
    let png_data = out.encode_png()?;

    // serenity wants a cow for whatever reason
    let cow = Cow::from(png_data);

    // construct reply message
    msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.image("attachment://map.png");
                e.footer(|f| {
                    f.text(format!("{} {}", BOT_NAME, BOT_VERSION));
                    f
                });
                e
            });
            m.add_file(AttachmentType::Bytes {
                data: cow,
                filename: String::from("map.png"),
            });
            m
        })
        .await?;
    Ok(())
}

fn add_rect(spot: GatherSpot, svgtree: &mut Tree) {
    let x = calc_x(spot.location.x);
    let y = calc_z(spot.location.z);

    let fill = Some(usvg::Fill {
        paint: usvg::Paint::Color(usvg::Color::new_rgb(
            255,
            0,
            0,
        )),
        opacity: 1.0.into(),
        ..usvg::Fill::default()
    });

    // create the element
    svgtree.root().append_kind(usvg::NodeKind::Path(usvg::Path {
        fill,
        data: Rc::new(usvg::PathData::from_rect(usvg::Rect::new(
            x - 2.5, y - 2.5, 5.0, 5.0,
        ).unwrap())),
        ..usvg::Path::default()
    }));
}

async fn gather_usage(ctx: &Context, msg: &Message) -> CommandResult {
    create_error_msg(ctx, msg, "Invalid command arguments", "correct usage: .gather (gather type(s)) (material)\nValid types: (W)oodcutting, (M)ining, (G)rowing and (F)ishing").await;
    
    Ok(())
}
