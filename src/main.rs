use std::{
    collections::HashSet,
    fs::File,
    io::{self, Write},
};

use regex::Regex;
use reqwest::Client;
use tokio::task;

const LAST_PAGE: i32 = 154;
const BASE_URL: &str = "https://vipergirls.to/threads/5413100-Zishy-photos-that-leave-something-to-the-imagination-complete-amp-updated/page";

#[tokio::main]
async fn main() -> io::Result<()> {
    let client = Client::new();
    let mut handles = vec![];

    for i in 1..=LAST_PAGE {
        let page = format!("{BASE_URL}{i}");
        let client = client.clone();
        let handle = task::spawn(async move {
            let response = client.get(page).send().await?;
            let body = response.text().await?;
            let posts: Vec<_> = body.split("postcontainer").map(extract_imgs).collect();
            Ok::<_, reqwest::Error>(posts)
        });
        handles.push(handle);
    }

    let mut extracted_imgs = vec![];
    let mut other_imgs = HashSet::<String>::new();

    for handle in handles {
        match handle.await {
            Ok(Ok(posts)) => {
                let (extracted, other): (Vec<_>, Vec<_>) = posts.into_iter().unzip();
                extracted_imgs.extend(extracted);
                other.into_iter().for_each(|imgs| other_imgs.extend(imgs));
            }
            Ok(Err(e)) => eprintln!("Request error: {}", e),
            Err(e) => eprintln!("Task error: {}", e),
        }
    }

    for img in other_imgs {
        println!("{}", img);
    }

    let mut file = File::create("docs/data.js")?;
    file.write_all(b"let posts = [\n[\"")?;
    file.write_all(
        extracted_imgs
            .into_iter()
            .filter(|post| !post.is_empty())
            .map(|post| post.join("\", \""))
            .collect::<Vec<_>>()
            .join("\"],\n[\"")
            .as_bytes(),
    )?;
    file.write_all(b"\"]\n]")?;

    Ok(())
}

fn extract_imgs(post: &str) -> (Vec<String>, Vec<String>) {
    let img_re = Regex::new("<img src=\".*?\"").unwrap();
    let imgs: Vec<_> = img_re
        .find_iter(post)
        .map(|m| m.as_str().to_string())
        .collect();
    let mut extracted_imgs = vec![];

    let imgbox_re = Regex::new("https://thumbs2.imgbox.com/(.*?)_t.(jpe?g)").unwrap();
    let (imgbox_imgs, other_imgs): (Vec<_>, Vec<_>) =
        imgs.into_iter().partition(|img| imgbox_re.is_match(img));
    let imgbox_imgs: Vec<_> = imgbox_imgs
        .iter()
        .map(|img| imgbox_re.captures(img).unwrap())
        .map(|cap| {
            format!(
                "https://images2.imgbox.com/{}_o.{}",
                cap.get(1).unwrap().as_str(),
                cap.get(2).unwrap().as_str()
            )
        })
        .collect();
    extracted_imgs.extend(imgbox_imgs);

    let imx_re = Regex::new("https://imx.to/u/t/(.*?).jpg").unwrap();
    let (imx_imgs, other_imgs): (Vec<_>, Vec<_>) =
        other_imgs.into_iter().partition(|i| imx_re.is_match(i));
    let imx_imgs: Vec<_> = imx_imgs
        .iter()
        .map(|i| imx_re.captures(i).unwrap().get(1).unwrap().as_str())
        .map(|s| format!("https://imx.to/u/i/{}.jpg", s))
        .collect();
    extracted_imgs.extend(imx_imgs);

    (extracted_imgs, other_imgs)
}
