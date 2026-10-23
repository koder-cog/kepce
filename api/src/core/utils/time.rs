//! Türkiye (Europe/Istanbul - UTC+3) saat dilimi yardımcıları.
//!
//! Türkiye 2016 yılından bu yana kalıcı olarak UTC+3 saat dilimindedir (yaz/kış saati farkı yoktur).
//! Sistem genelinde sunucu yerel saatinden bağımsız olarak Türkiye gününü ve saatini garanti eder.

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};

/// Türkiye saat dilimi sabit ofseti (UTC+3, 10800 saniye).
#[inline]
pub fn istanbul_tz() -> FixedOffset {
    FixedOffset::east_opt(3 * 3600).expect("UTC+3 geçerli bir zaman dilimi ofsetidir")
}

/// Europe/Istanbul saat dilimindeki güncel tarih ve saat.
pub fn istanbul_now() -> DateTime<FixedOffset> {
    Utc::now().with_timezone(&istanbul_tz())
}

/// Europe/Istanbul saat dilimindeki bugünün tarihi (gece 00:00-03:00 UTC açığını önler).
pub fn istanbul_today() -> NaiveDate {
    istanbul_now().date_naive()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_istanbul_offset() {
        let tz = istanbul_tz();
        assert_eq!(tz.local_minus_utc(), 3 * 3600);
    }

    #[test]
    fn test_istanbul_today() {
        let today = istanbul_today();
        assert!(today.year() >= 2026);
    }
}
