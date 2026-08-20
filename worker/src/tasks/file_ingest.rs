use anyhow::Result;
use chrono::Local;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use shared::entities::cities;
use std::env;
use std::path::PathBuf;

pub async fn process_local_files(db: &DatabaseConnection, reqwest_client: &reqwest::Client, gemini_api_key: Option<&str>) -> Result<()> {
    let base_dir = env::var("WORKER_MENU_DIR").unwrap_or_else(|_| "../data/menuler".to_string());
    
    let configs = ["admin", "kullanici"];
    let mut processed = 0;

    for folder in configs.iter() {
        let bekleyen_path = PathBuf::from(&base_dir).join(folder).join("bekleyen");
        if !bekleyen_path.exists() {
            continue;
        }

        let mut cities_iter = match tokio::fs::read_dir(&bekleyen_path).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        while let Ok(Some(city_entry)) = cities_iter.next_entry().await {
            let city_path = city_entry.path();
            if !city_path.is_dir() {
                continue;
            }

            let city_slug = city_entry.file_name().to_string_lossy().to_string();
            
            // Get city ID from slug
            let city_opt = cities::Entity::find()
                .filter(cities::Column::Slug.eq(&city_slug))
                .one(db)
                .await?;
                
            let city_id = match city_opt {
                Some(c) => c.id,
                None => {
                    tracing::warn!("Lokal dosya taraması: '{}' adlı şehir veritabanında bulunamadı, atlanıyor.", city_slug);
                    continue;
                }
            };

            let mut files_iter = match tokio::fs::read_dir(&city_path).await {
                Ok(f) => f,
                Err(_) => continue,
            };

            while let Ok(Some(file_entry)) = files_iter.next_entry().await {
                let path = file_entry.path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                tracing::info!("Lokal dosya ayrıştırılıyor: {}/{}", city_slug, filename);

                let path_str = path.to_string_lossy().to_string();
                
                let source_type = format!("kepce-{}", folder);
                
                let result = if ext.to_lowercase() == "xlsx" {
                    let mut file_db = crate::parser::models::MenuDatabase::new();
                    match crate::parser::excel::parse_excel(&path_str, &mut file_db) {
                        Ok(_) => {
                            for day_data in file_db.values_mut() {
                                crate::parser::validation::finalize_day_metadata(day_data);
                            }
                            crate::parser::save_menu_database(db, city_id, &source_type, file_db, &city_slug).await
                        }
                        Err(e) => Err(anyhow::anyhow!("Excel parse hatası: {}", e))
                    }
                } else if ext.to_lowercase() == "pdf" {
                    if let Some(key) = gemini_api_key {
                        match crate::parser::llm::parse_pdf_with_llm(reqwest_client, key, std::path::Path::new(&path_str)).await {
                            Ok(file_db) => {
                                crate::parser::save_menu_database(db, city_id, &source_type, file_db, &city_slug).await
                            }
                            Err(e) => Err(anyhow::anyhow!("PDF parse hatası: {}", e))
                        }
                    } else {
                        tracing::warn!("{}: PDF parsing devre dışı - GEMINI_API_KEY ayarlanmamış, atlanıyor.", filename);
                        continue;
                    }
                } else {
                    continue;
                };

                let success = match result {
                    Ok(_) => {
                        tracing::info!("{}: Başarıyla veritabanına işlendi.", filename);
                        processed += 1;
                        true
                    },
                    Err(e) => {
                        let err_msg = format!("{:?}", e).to_lowercase();
                        if err_msg.contains("timeout") || err_msg.contains("geçici api hatası") || err_msg.contains("istek atılamadı") {
                            tracing::error!("{}: Geçici ağ/API hatası, dosya kuyrukta bekletilecek: {:?}", filename, e);
                            continue; // Dosyayı bekleyen klasöründe bırak (taşıma)
                        } else {
                            tracing::error!("{}: Kalıcı ayrıştırma/kaydetme hatası: {:?}", filename, e);
                            false
                        }
                    }
                };

                // Move file
                if success {
                    let vault_base = PathBuf::from(&base_dir).join("vault");
                    let vault_dir = vault_base.join(ext.to_lowercase()).join(folder).join(&city_slug);
                    let _ = tokio::fs::create_dir_all(&vault_dir).await;
                    let dest = vault_dir.join(format!("{}_{}", Local::now().format("%Y%m%d_%H%M%S"), filename));
                    if let Err(e) = tokio::fs::rename(&path, &dest).await {
                        tracing::warn!("Dosya taşınamadı ({:?}). Kopyalama + silme deneniyor...", e);
                        if let Err(copy_err) = tokio::fs::copy(&path, &dest).await {
                            tracing::error!("Kopyalama başarısız ({:?}). Sonsuz döngüyü önlemek için dosya uzantısı .failed yapılıyor...", copy_err);
                            let failed_dest = path.with_extension(format!("{}.failed", ext));
                            if let Err(rename_err) = tokio::fs::rename(&path, &failed_dest).await {
                                tracing::error!("Dosya .failed olarak yeniden adlandırılamadı: {:?}", rename_err);
                            }
                        } else {
                            if let Err(remove_err) = tokio::fs::remove_file(&path).await {
                                tracing::error!("Kaynak dosya silinemedi ({:?}): {:?}. Yeniden işlenmemesi için .processed yapılıyor...", path, remove_err);
                                let _ = tokio::fs::rename(&path, path.with_extension(format!("{}.processed", ext))).await;
                            }
                        }
                    }
                } else {
                    let err_dir = PathBuf::from(&base_dir).join(folder).join("hatali");
                    let _ = tokio::fs::create_dir_all(&err_dir).await;
                    let dest = err_dir.join(filename);
                    if let Err(e) = tokio::fs::rename(&path, &dest).await {
                        tracing::warn!("Hatalı dosya taşınamadı ({:?}). Kopyalama + silme deneniyor...", e);
                        if let Err(copy_err) = tokio::fs::copy(&path, &dest).await {
                            tracing::error!("Kopyalama başarısız ({:?}). Sonsuz döngüyü önlemek için dosya uzantısı .failed yapılıyor...", copy_err);
                            let failed_dest = path.with_extension(format!("{}.failed", ext));
                            if let Err(rename_err) = tokio::fs::rename(&path, &failed_dest).await {
                                tracing::error!("Dosya .failed olarak yeniden adlandırılamadı: {:?}", rename_err);
                            }
                        } else {
                            if let Err(remove_err) = tokio::fs::remove_file(&path).await {
                                tracing::error!("Kaynak dosya silinemedi ({:?}): {:?}. Yeniden işlenmemesi için .failed yapılıyor...", path, remove_err);
                                let _ = tokio::fs::rename(&path, path.with_extension(format!("{}.failed", ext))).await;
                            }
                        }
                    }
                }
            }
        }
    }

    tracing::info!("Lokal dosya taraması tamamlandı. {} dosya işlendi.", processed);
    Ok(())
}
