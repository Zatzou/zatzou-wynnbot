use std::borrow::Cow;
use std::error::Error;

use crc32fast::Hasher;
use image::{DynamicImage, Rgba};
use imageproc::drawing;
use imageproc::rect::Rect;
use once_cell::sync::OnceCell;
use rusttype::{Font, Scale};
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::http::AttachmentType;
use serenity::model::prelude::*;
use serenity::prelude::*;

use image::io::Reader as ImageReader;

use tracing::info;

use crate::config::get_config;
use crate::wynn::world::Territories;
use crate::{BOT_NAME, BOT_VERSION};
use cached::proc_macro::cached;

/// Static for the image file so we don't have to load it every time
static MAPBASE: OnceCell<image::ImageBuffer<Rgba<u8>, Vec<u8>>> = OnceCell::new();

/// Helper function for getting the base map
fn get_mapbase() -> Result<image::ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error + Send + Sync>> {
    let out = if let Some(pm) = MAPBASE.get() {
        pm.clone()
    } else {
        let map: image::ImageBuffer<Rgba<u8>, Vec<u8>> = if let DynamicImage::ImageRgba8(img) =
            ImageReader::open("./resources/main-map.png")?.decode()?
        {
            img
        } else {
            panic!("main-map.png invalid!!!");
        };
        MAPBASE.set(map.clone()).unwrap();
        map
    };
    Ok(out)
}

// allow getting a new map every 30s otherwise use a cached one
#[cached(time = 30, result = true)]
async fn get_map() -> Result<Territories, reqwest::Error> {
    info!("Getting new map data from wynntils");
    let terrs: Territories = reqwest::get("https://athena.wynntils.com/cache/get/territoryList")
        .await?
        .json()
        .await?;
    Ok(terrs)
}

#[command]
#[description("Renders the guild map")]
#[help_available]
#[only_in(guilds)]
#[bucket("image")]
async fn map(ctx: &Context, msg: &Message) -> CommandResult {
    let processingmsg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Processing");
                e.description(
                    "Your request is currently processing it may take a second to complete",
                );
                e
            });
            m
        })
        .await?;

    // load territory data from wynntils api
    let terrs = get_map().await?;

    // ouput image
    let mut out = drawing::Blend(get_mapbase()?);

    // name rendering stuff
    let font_data: &[u8] = include_bytes!("../../resources/Roboto-Bold.ttf");
    let font: Font<'static> = Font::try_from_bytes(font_data).unwrap();

    // go thru all territories and render the rects for them
    for (_, terr) in terrs.territories.iter() {
        let loc = &terr.location;

        // widths
        let width = (f64::abs(loc.endX - loc.startX) / 3.0) as u32;
        let height = (f64::abs(loc.endZ - loc.startZ) / 3.0) as u32;

        // position calculations
        let x = if loc.startX < loc.endX {
            calc_x(loc.startX) as i32
        } else {
            calc_x(loc.endX) as i32
        };
        let y = if loc.startZ < loc.endZ {
            calc_z(loc.startZ) as i32
        } else {
            calc_z(loc.endZ) as i32
        };

        // guild color calculations
        let col = if !terr.guildColor.is_empty() {
            let col = hex::decode(terr.guildColor[1..].to_owned())?;
            (col[0], col[1], col[2])
        } else {
            guild_color(terr.guild.clone())
        };

        let fillcol = Rgba([col.0, col.1, col.2, 127]);
        let edgecol = Rgba([col.0, col.1, col.2, 255]);

        let area = Rect::at(x, y).of_size(width, height);

        drawing::draw_filled_rect_mut(&mut out, area, fillcol);
        drawing::draw_hollow_rect_mut(&mut out, area, edgecol);

        // maybe find a nice way to get a good color for this
        let textcol = Rgba([255, 255, 255, 255]);

        drawing::draw_text_mut(
            &mut out,
            textcol,
            x as u32 + 3,
            y as u32 + 3,
            Scale::uniform((width as f32 / 2.5).min(height as f32 / 1.5)),
            &font,
            &terr.guildPrefix,
        );
    }

    // encode image as webp of quality 80
    let img_data: Vec<u8>;

    {
        let img = &DynamicImage::ImageRgba8(out.0);

        let encoder = webp::Encoder::from_image(img)?;
        let encoded = encoder.encode(get_config().image.webp_quality);

        img_data = (*encoded).to_vec();
    }

    // serenity wants a cow for whatever reason
    let cow = Cow::from(img_data);

    // construct reply message
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.image("attachment://map.webp");
                e.footer(|f| {
                    f.text(format!("{} {}", BOT_NAME, BOT_VERSION));
                    f
                });
                e.timestamp(chrono::Utc::now().to_rfc3339());
                e
            });
            m.add_file(AttachmentType::Bytes {
                data: cow,
                filename: String::from("map.webp"),
            });
            m
        })
        .await?;

    processingmsg.delete(&ctx.http).await?;

    Ok(())
}

/// Gets the guilds color from it's name
#[cached]
fn guild_color(name: String) -> (u8, u8, u8) {
    // hash the guilds nane with crc32
    let mut hasher = Hasher::new();
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();

    let bytes: Vec<u8> = hash.to_ne_bytes().into_iter().rev().collect();

    (bytes[1], bytes[2], bytes[3])
}

fn calc_x(x: f64) -> f64 {
    (x / 3.0) - 566.333 + 1364.0
}

fn calc_z(z: f64) -> f64 {
    (z / 3.0) + 41.0 + 2162.0
}
