use std::{borrow::Cow, collections::HashMap};

use image::{DynamicImage, ImageBuffer, Rgba};
use imageproc::{drawing, rect::Rect};
use once_cell::sync::OnceCell;

use image::io::Reader as ImageReader;
use poise::serenity_prelude::{AttachmentType, Color};

use crate::gen_embed_footer;
use crate::error::create_error_msg;
use crate::{
    wynn::Gather::{self, GatherSpot, GatherSpots},
    Context, Error,
};

/// Static for the gray image file so we don't have to load it every time
static MAPBASE_GRAY: OnceCell<image::ImageBuffer<Rgba<u8>, Vec<u8>>> = OnceCell::new();

/// Helper function for getting the base map
fn get_mapbase_gray() -> Result<image::ImageBuffer<Rgba<u8>, Vec<u8>>, Error> {
    let out = if let Some(pm) = MAPBASE_GRAY.get() {
        pm.clone()
    } else {
        let map: image::ImageBuffer<Rgba<u8>, Vec<u8>> = if let DynamicImage::ImageRgba8(img) =
            ImageReader::open("./resources/main-map-gray.png")?.decode()?
        {
            img
        } else {
            panic!("main-map-gray.png invalid!!!");
        };
        MAPBASE_GRAY.set(map.clone()).unwrap();
        map
    };
    Ok(out)
}

/// Finds gather spots and renders them to a map
#[poise::command(prefix_command, slash_command)]
pub async fn gather(
    ctx: Context<'_>,
    #[rest]
    #[description = "Name of the material you want to query"]
    material: String,
) -> Result<(), Error> {
    let wanted = material.to_ascii_uppercase();

    // defer here so we can respond with an image and discord knows that it might take a while before we respond
    ctx.defer().await?;

    let spots = Gather::get_gatherspots().await?;

    let mut count = 0;
    let mut spots_wood = Vec::new();
    let mut spots_mining = Vec::new();
    let mut spots_farming = Vec::new();
    let mut spots_fishing = Vec::new();

    for spot in &spots.woodCutting {
        if spot.r#type.contains(&wanted) || wanted.contains(&spot.r#type) {
            count += 1;
            spots_wood.push(spot.clone());
            // add_rect(spot, Rgba([0, 255, 0, 255]), &mut out);
        }
    }
    for spot in &spots.mining {
        if spot.r#type.contains(&wanted) || wanted.contains(&spot.r#type) {
            count += 1;
            spots_mining.push(spot.clone());
            // add_rect(spot, Rgba([255, 0, 0, 255]), &mut out);
        }
    }
    for spot in &spots.farming {
        if spot.r#type.contains(&wanted) || wanted.contains(&spot.r#type) {
            count += 1;
            spots_farming.push(spot.clone());
            // add_rect(spot, Rgba([255, 255, 0, 255]), &mut out);
        }
    }
    for spot in &spots.fishing {
        if spot.r#type.contains(&wanted) || wanted.contains(&spot.r#type) {
            count += 1;
            spots_fishing.push(spot.clone());
            // add_rect(spot, Rgba([0, 0, 255, 255]), &mut out);
        }
    }

    if count == 0 {
        let mut alltypes = String::new();
        let types = get_all_res(&spots);

        for t in types {
            alltypes.push_str(&format!("`{}`, ", t.0.to_lowercase()));
        }

        create_error_msg(
            ctx,
            "No matches",
            &format!(
                "The current filter `{}` did not match any known resources\nCurrent known resource types are:\n{}",
                wanted,
                alltypes
            ),
        )
        .await;
        return Ok(());
    }

    let mut out = drawing::Blend(get_mapbase_gray()?);

    // render the spots
    for spot in &spots_wood {
        add_rect(spot, Rgba([0, 255, 0, 255]), &mut out);
    }
    for spot in &spots_mining {
        add_rect(spot, Rgba([255, 0, 0, 255]), &mut out);
    }
    for spot in &spots_farming {
        add_rect(spot, Rgba([255, 255, 0, 255]), &mut out);
    }
    for spot in &spots_fishing {
        add_rect(spot, Rgba([0, 0, 255, 255]), &mut out);
    }

    // encode image as webp
    let img_data: Vec<u8>;

    {
        let img = &DynamicImage::ImageRgba8(out.0);

        let encoder = webp::Encoder::from_image(img)?;
        let encoded = encoder.encode(ctx.data().config.image.webp_quality);

        img_data = (*encoded).to_vec();
    }

    // serenity wants a cow for whatever reason
    let cow = Cow::from(img_data);

    // construct reply message
    ctx.send(|m| {
        m.embed(|e| {
            e.title(format!("{} matches", count));
            e.image("attachment://map.webp");
            gen_embed_footer(e, &ctx.data().config.bot.name);
            e
        });
        m.attachment(AttachmentType::Bytes {
            data: cow,
            filename: String::from("map.webp"),
        });
        m
    })
    .await?;

    Ok(())
}

/// calculate the x offset
fn calc_x(x: f64) -> f64 {
    (x / 3.0) - 566.333 + 1364.0
}

/// calculate the y offset
fn calc_z(z: f64) -> f64 {
    (z / 3.0) + 41.0 + 2162.0
}

/// Adds a spot to the image
fn add_rect(
    spot: &GatherSpot,
    color: Rgba<u8>,
    img: &mut drawing::Blend<ImageBuffer<Rgba<u8>, Vec<u8>>>,
) {
    let x = calc_x(spot.location.x) as i32;
    let y = calc_z(spot.location.z) as i32;

    let rect = Rect::at(x - 3, y - 3).of_size(5, 5);

    // create the element
    drawing::draw_filled_rect_mut(img, rect, color);
}

/// Gets all of the resource types
fn get_all_res<'a>(spots: &GatherSpots) -> HashMap<String, i32> {
    let mut out: HashMap<String, i32> = HashMap::new();

    for s in &spots.woodCutting {
        append_spots(&mut out, &s);
    }
    for s in &spots.mining {
        append_spots(&mut out, &s);
    }
    for s in &spots.farming {
        append_spots(&mut out, &s);
    }
    for s in &spots.fishing {
        append_spots(&mut out, &s);
    }

    out
}

fn append_spots(map: &mut HashMap<String, i32>, spot: &GatherSpot) {
    if let Some(v) = map.get_mut(&spot.r#type) {
        *v += 1;
    } else {
        map.insert(spot.r#type.clone(), 0);
    }
}
