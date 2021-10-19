use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;

use serenity::framework::standard::{macros::command, CommandResult};
use serenity::http::AttachmentType;
use serenity::model::prelude::*;
use serenity::prelude::*;
use usvg::NodeExt;

use tracing::{error, info};

use crate::wynn::world::{Territories, Territory};
use crate::{BOT_NAME, BOT_VERSION};

#[command]
async fn map(ctx: &Context, msg: &Message) -> CommandResult {
    // load territory data from wynntils api TODO: caching
    let terrs: Territories = reqwest::get("https://athena.wynntils.com/cache/get/territoryList")
        .await?
        .json()
        .await?;
    // let terrs = cache::get_territories(ctx).await?; // TODO: get this bs working

    // ouput image
    let mut out = tiny_skia::Pixmap::load_png("./main-map2.png")?;

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
        for (name, terr) in terrs.territories.iter() {
            let loc = &terr.location;

            // widths
            let width = f64::abs(loc.endX - loc.startX) / 3.0;
            let height = f64::abs(loc.endZ - loc.startZ) / 3.0;

            // position calculations
            let x = if loc.startX < loc.endX {
                (loc.startX / 3.0) - 566.333 + 1364.0
            } else {
                (loc.endX / 3.0) - 566.333 + 1364.0
            };
            let y = if loc.startZ < loc.endZ {
                (loc.startZ / 3.0) + 41.0 + 2162.0
            } else {
                (loc.endZ / 3.0) + 41.0 + 2162.0
            };

            // guild color calculations
            let col = if terr.guildColor != "" {
                &terr.guildColor[1..]
            } else {
                "ffffff"
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
                stroke: stroke,
                fill: fill,
                data: Rc::new(usvg::PathData::from_rect(usvg::Rect::new(
                    x, y, width, height,
                ).unwrap())),
                ..usvg::Path::default()
            }));
        }

        let mut guilds: HashMap<String, Vec<Territory>> = HashMap::new();

        for (name, terr) in terrs.territories.into_iter() {
            if let Some(t) = guilds.get_mut(&terr.guildPrefix) {
                t.push(terr);
            } else {
                let guild = terr.guildPrefix.clone();
                let mut vec = Vec::new();
                vec.push(terr);
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
            let svege = usvg::Tree::from_str(&format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1364 2162"><text x="{}" y="{}">{}</text></svg>"#, pos.0, pos.1, name), &usvg::Options::default().to_ref()).unwrap();
            info!("{} at {} {}", name, pos.0, pos.1);
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



fn median(list: &[f64]) -> f64 {
    let len = list.len();
    let mid = len / 2;
    if len % 2 == 0 {
        mean(&list[(mid - 1)..(mid + 1)])
    } else {
        f64::from(list[mid])
    }
}

fn mean(list: &[f64]) -> f64 {
    let sum: f64 = Iterator::sum(list.iter());
    f64::from(sum) / (list.len() as f64)
}
