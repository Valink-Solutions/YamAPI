use worker::*;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {

    let router = Router::new();

    router
        .get_async("/images/:category/:id", get_image_simple)
        .run(req, env).await
}


async fn get_image_simple(_req: worker::Request, ctx: RouteContext<()>) -> Result<Response> {
    let category = ctx.param("category").unwrap().to_lowercase();
    let id = ctx.param("id").unwrap();
    let split_id: Vec<&str> = id.split(':').collect();
    let pack = split_id[0];
    let item = split_id[1];

    let image_path = format!("{}/{}/{}.png", pack, category, item);

    let bucket = match ctx.bucket("YAMAPI_IMAGES_BUCKET") {
        Ok(bucket) => bucket,
        Err(_e) => return Response::error(format!("Image service down."), 500),
    };
    
    match bucket.get(&image_path).execute().await {
        Ok(resp) => {
            match resp {
                Some(object) => {
                    let body = object.body();
                    let bytes = match body {
                        Some(body) => body.bytes().await.unwrap(),
                        None => return Response::error("No body found in object", 404),
                    };
                    
                    let mut response = Response::from_bytes(bytes).unwrap();

                    response.headers_mut().set("Content-Type", "image/png").unwrap();

                    Ok(response)
                    
                },
                None => Response::error(format!("Image for {id} not found"), 404)
            }
        },
        Err(_e) => Response::error(format!("No image found"), 404),
    }
}