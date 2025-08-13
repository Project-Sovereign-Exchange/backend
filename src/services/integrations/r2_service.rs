use std::io::Cursor;
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::{config::Region, primitives::ByteStream, types, Client, Error};
use std::path::Path;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use image::codecs::jpeg::JpegEncoder;
use image::DynamicImage;
use image::ImageFormat::Jpeg;
use crate::app_state::AppState;

pub struct R2Client {
    client: Client,
    custom_domain: String,
}

impl R2Client {
    pub async fn new(
        account_id: &str,
        access_key_id: &str,
        secret_access_key: &str,
        custom_domain: &str,
    ) -> Result<Self, Error> {
        let region_provider = RegionProviderChain::default_provider().or_else("auto");

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region_provider)
            .endpoint_url(format!("https://{}.r2.cloudflarestorage.com", account_id))
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                access_key_id,
                secret_access_key,
                None,
                None,
                "r2-credentials",
            ))
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(s3_config);
        let custom_domain = custom_domain.to_string();

        Ok(R2Client {
            client,
            custom_domain,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VariantImageUrls {
    pub front: Option<ImageSizeUrls>,
    pub back: Option<ImageSizeUrls>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageSizeUrls {
    pub original: String,
    pub medium: String,
    pub thumbnail: String,
}

pub struct R2Service {
    state: AppState,
}

impl R2Service {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn upload_product_image(
        &self,
        product_id: &str,
        image_data: &[u8],
        user_id: &str,
    ) -> Result<(), anyhow::Error> {
        let timestamp = Utc::now();

        let img = image::load_from_memory(image_data)?;
        let original = img.clone();
        let medium = img.resize(600, 450, image::imageops::FilterType::Lanczos3);
        let thumbnail = img.resize(200, 150, image::imageops::FilterType::Lanczos3);

        let original_bytes = image_to_jpeg_bytes(&original, 95)?;
        let medium_bytes = image_to_jpeg_bytes(&medium, 85)?;
        let thumbnail_bytes = image_to_jpeg_bytes(&thumbnail, 80)?;

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), "product".to_string());
        metadata.insert("product_id".to_string(), product_id.to_string());
        metadata.insert("uploaded_by".to_string(), user_id.to_string());
        metadata.insert("uploaded_at".to_string(), timestamp.to_rfc3339());

        let original_key = format!("products/{}/original.jpg", product_id);
        let medium_key = format!("products/{}/medium.jpg", product_id);
        let thumbnail_key = format!("products/{}/thumbnail.jpg", product_id);

        let bucket_name = "images";

        self.upload_from_bytes(bucket_name, original_bytes, &original_key, "image/jpeg", Some(metadata.clone())).await?;
        self.upload_from_bytes(bucket_name, medium_bytes, &medium_key, "image/jpeg", Some(metadata.clone())).await?;
        self.upload_from_bytes(bucket_name, thumbnail_bytes, &thumbnail_key, "image/jpeg", Some(metadata)).await?;

        Ok(())
    }

    async fn upload_listing_image(
        &self,
        image_data: &[u8],
        listing_id: &str,
        image_path: &str,
        user_id: &str,
        timestamp: &DateTime<Utc>,
    ) -> Result<(), anyhow::Error> {
        let img = image::load_from_memory(image_data)?;
        let original = img.clone();
        let thumbnail = img.resize(300, 225, image::imageops::FilterType::Lanczos3);

        let original_bytes = image_to_jpeg_bytes(&original, 95)?;
        let thumbnail_bytes = image_to_jpeg_bytes(&thumbnail, 80)?;

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), "listing".to_string());
        metadata.insert("listing_id".to_string(), listing_id.to_string());
        metadata.insert("image_path".to_string(), image_path.to_string());
        metadata.insert("uploaded_by".to_string(), user_id.to_string());
        metadata.insert("uploaded_at".to_string(), timestamp.to_rfc3339());

        let original_key = format!("listings/{}/{}/original.jpg", listing_id, image_path);
        let thumbnail_key = format!("listings/{}/{}/thumbnail.jpg", listing_id, image_path);

        let bucket_name = "images";

        self.upload_from_bytes(bucket_name, original_bytes, &original_key, "image/jpeg", Some(metadata.clone())).await?;
        self.upload_from_bytes(bucket_name, thumbnail_bytes, &thumbnail_key, "image/jpeg", Some(metadata)).await?;

        Ok(())
    }

    async fn upload_from_bytes(
        &self,
        bucket_name: &str,
        data: Vec<u8>,
        key: &str,
        content_type: &str,
        metadata: Option<std::collections::HashMap<String, String>>,
    ) -> Result<String, anyhow::Error> {
        let mut put_object = self
            .state
            .r2_client
            .client
            .put_object()
            .bucket(bucket_name)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type);

        if let Some(meta) = metadata {
            for (k, v) in meta {
                put_object = put_object.metadata(k, v);
            }
        }

        let _result = put_object.send().await?;
        Ok(key.to_string())
    }

    pub async fn delete_product_images(&self, product_id: &str) -> Result<(), anyhow::Error> {
        let prefix = format!("products/{}/", product_id);
        let bucket_name = "images";
        self.delete_objects_with_prefix(bucket_name, &prefix).await
    }

    pub async fn delete_listing_images(&self, listing_id: &str) -> Result<(), anyhow::Error> {
        let prefix = format!("listings/{}/", listing_id);
        let bucket_name = "images";
        self.delete_objects_with_prefix(bucket_name, &prefix).await
    }

    async fn delete_objects_with_prefix(
        &self,
        bucket_name: &str,
        prefix: &str
    ) -> Result<(), anyhow::Error> {
        let list_response = self
            .state
            .r2_client
            .client
            .list_objects_v2()
            .bucket(bucket_name)
            .prefix(prefix)
            .send()
            .await?;

        if let Some(contents) = list_response.contents {
            for object in contents {
                if let Some(key) = object.key() {
                    self.state
                        .r2_client
                        .client
                        .delete_object()
                        .bucket(bucket_name)
                        .key(key)
                        .send()
                        .await?;
                }
            }
        }

        Ok(())
    }

    pub async fn upload_product_variant_images(
        &self,
        product_id: &Uuid,
        user_id: &Uuid,
        game: &str,
        variant_name: &str,
        front_image: Option<&[u8]>,
        back_image: Option<&[u8]>,
    ) -> Result<VariantImageUrls, anyhow::Error> {
        let timestamp = Utc::now();
        let sanitized_game = sanitize_for_path(game);

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), "product_variant".to_string());
        metadata.insert("product_id".to_string(), product_id.to_string());
        metadata.insert("variant_id".to_string(), variant_name.to_string());
        metadata.insert("game".to_string(), game.to_string());
        metadata.insert("uploaded_by".to_string(), user_id.to_string());
        metadata.insert("uploaded_at".to_string(), timestamp.to_rfc3339());

        let bucket_name = "images";
        let mut result = VariantImageUrls {
            front: None,
            back: None,
        };

        if let Some(front_data) = front_image {
            let front_urls = self.process_and_upload_variant_image(
                front_data,
                &sanitized_game,
                &product_id.to_string(),
                variant_name,
                "front",
                bucket_name,
                metadata.clone(),
            ).await?;
            result.front = Some(front_urls);
        }

        if let Some(back_data) = back_image {
            let back_urls = self.process_and_upload_variant_image(
                back_data,
                &sanitized_game,
                &product_id.to_string(),
                variant_name,
                "back",
                bucket_name,
                metadata.clone(),
            ).await?;
            result.back = Some(back_urls);
        }

        Ok(result)
    }

    async fn process_and_upload_variant_image(
        &self,
        image_data: &[u8],
        game: &str,
        product_id: &str,
        variant_name: &str,
        face: &str,
        bucket_name: &str,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<ImageSizeUrls, anyhow::Error> {
        let img = image::load_from_memory(image_data)?;
        let original = img.clone();
        let medium = img.resize(600, 450, image::imageops::FilterType::Lanczos3);
        let thumbnail = img.resize(200, 150, image::imageops::FilterType::Lanczos3);

        let original_bytes = image_to_jpeg_bytes(&original, 95)?;
        let medium_bytes = image_to_jpeg_bytes(&medium, 85)?;
        let thumbnail_bytes = image_to_jpeg_bytes(&thumbnail, 80)?;

        // Key format: products/{game}/{product_id}/{variant_id}/{face}/{size}.jpg
        let base_path = format!("products/{}/{}/{}/{}", game, product_id, variant_name, face);
        let original_key = format!("{}/original.jpg", base_path);
        let medium_key = format!("{}/medium.jpg", base_path);
        let thumbnail_key = format!("{}/thumbnail.jpg", base_path);

        // Upload all sizes
        self.upload_from_bytes(bucket_name, original_bytes, &original_key, "image/jpeg", Some(metadata.clone())).await?;
        self.upload_from_bytes(bucket_name, medium_bytes, &medium_key, "image/jpeg", Some(metadata.clone())).await?;
        self.upload_from_bytes(bucket_name, thumbnail_bytes, &thumbnail_key, "image/jpeg", Some(metadata)).await?;

        Ok(ImageSizeUrls {
            original: format!("{}/{}", self.state.r2_client.custom_domain, original_key),
            medium: format!("{}/{}", self.state.r2_client.custom_domain, medium_key),
            thumbnail: format!("{}/{}", self.state.r2_client.custom_domain, thumbnail_key),
        })
    }

    pub async fn delete_product_variant_images(
        &self,
        product_id: Uuid,
        game: &str,
        variant_id: Option<&str>,
    ) -> Result<(), anyhow::Error> {
        let sanitized_game = sanitize_for_path(game);
        let bucket_name = "images";

        let prefix = if let Some(variant) = variant_id {
            // Delete specific variant
            format!("products/{}/{}/{}/", sanitized_game, product_id, variant)
        } else {
            // Delete all variants for this product
            format!("products/{}/{}/", sanitized_game, product_id)
        };

        self.delete_objects_with_prefix(bucket_name, &prefix).await
    }

    pub async fn get_product_variant_image_urls(
        &self,
        product_id: Uuid,
        game: &str,
        variant_id: &str,
    ) -> VariantImageUrls {
        let sanitized_game = sanitize_for_path(game);
        let base_url = &self.state.r2_client.custom_domain;

        let front_base = format!("products/{}/{}/{}/front", sanitized_game, product_id, variant_id);
        let back_base = format!("products/{}/{}/{}/back", sanitized_game, product_id, variant_id);

        VariantImageUrls {
            front: Some(ImageSizeUrls {
                original: format!("{}/{}/original.jpg", base_url, front_base),
                medium: format!("{}/{}/medium.jpg", base_url, front_base),
                thumbnail: format!("{}/{}/thumbnail.jpg", base_url, front_base),
            }),
            back: Some(ImageSizeUrls {
                original: format!("{}/{}/original.jpg", base_url, back_base),
                medium: format!("{}/{}/medium.jpg", base_url, back_base),
                thumbnail: format!("{}/{}/thumbnail.jpg", base_url, back_base),
            }),
        }
    }
}

fn image_to_jpeg_bytes(img: &image::DynamicImage, quality: u8) -> Result<Vec<u8>, anyhow::Error> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);

    let mut encoder = JpegEncoder::new_with_quality(&mut cursor, quality);

    encoder.encode_image(img)?;

    Ok(bytes)
}

fn sanitize_for_path(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|c| match c {
            'a'..='z' | '0'..='9' => c,
            ' ' | '-' | '_' => '-',
            _ => '_',
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

