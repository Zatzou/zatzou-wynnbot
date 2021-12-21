use std::{borrow::Cow, error::Error};

use image::{DynamicImage, ImageBuffer, Rgba};
use imageproc::{drawing, rect::Rect};
use once_cell::sync::OnceCell;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    http::AttachmentType,
    model::channel::Message,
};

use image::io::Reader as ImageReader;

use crate::{wynn::Gather::{self, GatherSpot}, config::get_config};
use crate::{
    error::create_error_msg, helpers::parse_command_args_raw, BOT_NAME, BOT_VERSION};

/// Static for the gray image file so we don't have to load it every time
static MAPBASE_GRAY: OnceCell<image::ImageBuffer<Rgba<u8>, Vec<u8>>> = OnceCell::new();

/// Helper function for getting the base map
fn get_mapbase_gray() -> Result<image::ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn Error + Send + Sync>>
{
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

#[command]
async fn gather(ctx: &Context, msg: &Message) -> CommandResult {
    let cmd_args = parse_command_args_raw(msg);

    // get the wanted resource
    let wanted = if let Some(s) = cmd_args {
        s.trim().to_uppercase()
    } else {
        return gather_usage(ctx, msg).await;
    };

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

    let spots = Gather::get_gatherspots().await?;

    let mut out = drawing::Blend(get_mapbase_gray()?);

    let mut count = 0;

    for spot in spots.woodCutting {
        if spot.r#type.contains(&wanted) || wanted.contains(&spot.r#type) {
            count += 1;
            add_rect(spot, Rgba([0, 255, 0, 255]), &mut out);
        }
    }
    for spot in spots.mining {
        if spot.r#type.contains(&wanted) || wanted.contains(&spot.r#type) {
            count += 1;
            add_rect(spot, Rgba([255, 0, 0, 255]), &mut out);
        }
    }
    for spot in spots.farming {
        if spot.r#type.contains(&wanted) || wanted.contains(&spot.r#type) {
            count += 1;
            add_rect(spot, Rgba([255, 255, 0, 255]), &mut out);
        }
    }
    for spot in spots.fishing {
        if spot.r#type.contains(&wanted) || wanted.contains(&spot.r#type) {
            count += 1;
            add_rect(spot, Rgba([0, 0, 255, 255]), &mut out);
        }
    }

    if count == 0 {
        // delete the processing message
        processingmsg.delete(&ctx.http).await?;
        create_error_msg(
            ctx,
            msg,
            "No matches",
            &format!(
                "The current filter `{}` did not match any known resources",
                wanted
            ),
        )
        .await;
        return Ok(());
    }

    // encode image as webp
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
                e.title(format!("{} matches", count));
                e.image("attachment://map.webp");
                e.footer(|f| {
                    f.text(format!("{} {}", BOT_NAME, BOT_VERSION));
                    f
                });
                e
            });
            m.add_file(AttachmentType::Bytes {
                data: cow,
                filename: String::from("map.webp"),
            });
            m
        })
        .await?;

    // delete the processing message
    processingmsg.delete(&ctx.http).await?;

    Ok(())
}

fn calc_x(x: f64) -> f64 {
    (x / 3.0) - 566.333 + 1364.0
}

fn calc_z(z: f64) -> f64 {
    (z / 3.0) + 41.0 + 2162.0
}

fn add_rect(
    spot: GatherSpot,
    color: Rgba<u8>,
    img: &mut drawing::Blend<ImageBuffer<Rgba<u8>, Vec<u8>>>,
) {
    let x = calc_x(spot.location.x) as i32;
    let y = calc_z(spot.location.z) as i32;

    let rect = Rect::at(x - 3, y - 3).of_size(5, 5);

    // create the element
    drawing::draw_filled_rect_mut(img, rect, color);
}

async fn gather_usage(ctx: &Context, msg: &Message) -> CommandResult {
    create_error_msg(
        ctx,
        msg,
        "Invalid command arguments",
        "correct usage: .gather (material)\nYou can also use a partial material name",
    )
    .await;

    Ok(())
}
