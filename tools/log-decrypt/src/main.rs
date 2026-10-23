use crypto_box::{SecretKey, aead::OsRng};
use std::env;
use std::process;

fn print_usage() {
    eprintln!(
        "Kullanım:\n\
         cargo run --bin log-decrypt -- generate-keys\n\
         cargo run --bin log-decrypt -- decrypt <encrypted_hex> <private_key_hex>\n\
         cargo run --bin log-decrypt -- test-roundtrip"
    );
}

fn generate_keypair() {
    let secret_key = SecretKey::generate(&mut OsRng);
    let public_key = secret_key.public_key();

    println!("=== KEPÇE 5651 SIFIR BİLGİ ANAHTAR ÇİFTİ ===");
    println!("\n1. FRANSA SUNUCUSU .env İÇİN AÇIK ANAHTAR:");
    println!("TRAFFIC_LOG_PUBLIC_KEY=\"{}\"", hex::encode(public_key.as_bytes()));

    println!("\n2. TÜRKİYE ÇEVRİMDİŞİ USB / SOĞUK DEPOLAMA İÇİN ÖZEL ANAHTAR:");
    println!("(DİKKAT: Bu anahtarı ASLA Fransa sunucusuna veya git reposuna yüklemeyin!)");
    println!("TRAFFIC_LOG_PRIVATE_KEY=\"{}\"", hex::encode(secret_key.to_bytes()));
}

fn decrypt_log(encrypted_hex: &str, private_key_hex: &str) {
    let sk_bytes_vec = match hex::decode(private_key_hex.trim()) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Hata: Özel anahtar hex formatında değil: {}", e);
            process::exit(1);
        }
    };

    let sk_bytes: [u8; 32] = match sk_bytes_vec.try_into() {
        Ok(b) => b,
        Err(_) => {
            eprintln!("Hata: Özel anahtar 32 bayt (64 hex karakteri) olmalıdır.");
            process::exit(1);
        }
    };

    let ciphertext = match hex::decode(encrypted_hex.trim()) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Hata: Şifreli veri hex formatında değil: {}", e);
            process::exit(1);
        }
    };

    let secret_key = SecretKey::from(sk_bytes);
    match secret_key.unseal(&ciphertext) {
        Ok(plaintext) => {
            match String::from_utf8(plaintext) {
                Ok(json_str) => {
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        println!("{}", serde_json::to_string_pretty(&parsed).unwrap());
                    } else {
                        println!("{}", json_str);
                    }
                }
                Err(_) => {
                    eprintln!("Hata: Çözülen veri geçerli UTF-8 metni değil.");
                    process::exit(1);
                }
            }
        }
        Err(_) => {
            eprintln!("Hata: Mühür çözülemedi. Veri bozuk veya yanlış özel anahtar kullanıldı.");
            process::exit(1);
        }
    }
}

fn test_roundtrip() {
    let secret_key = SecretKey::generate(&mut OsRng);
    let public_key = secret_key.public_key();

    let sample_payload = serde_json::json!({
        "ip": "193.140.1.50",
        "port": 54321,
        "user_id": "00000000-0000-0000-0000-000000000001",
        "reference_id": "11111111-1111-1111-1111-111111111111",
        "service_type": "comment",
        "payload_bytes": 142,
        "started_at": "2026-10-23T12:00:00Z",
        "finished_at": "2026-10-23T12:00:01Z"
    });

    let plaintext = serde_json::to_vec(&sample_payload).unwrap();
    let ciphertext = public_key.seal(&mut OsRng, &plaintext).expect("Şifreleme başarısız");

    let unsealed = secret_key.unseal(&ciphertext).expect("Deşifre başarısız");
    assert_eq!(plaintext, unsealed);

    println!("Kriptografik SealedBox doğrulaması başarılı:");
    println!("Ham Veri Boyutu: {} bayt", plaintext.len());
    println!("Şifreli Veri Boyutu: {} bayt", ciphertext.len());
    println!("Çözülen Metin:\n{}", String::from_utf8(unsealed).unwrap());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "generate-keys" => generate_keypair(),
        "decrypt" => {
            if args.len() < 4 {
                eprintln!("Hata: encrypted_hex ve private_key_hex parametreleri zorunludur.");
                print_usage();
                process::exit(1);
            }
            decrypt_log(&args[2], &args[3]);
        }
        "test-roundtrip" => test_roundtrip(),
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}
