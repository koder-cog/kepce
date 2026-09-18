-- 05_badges.sql
-- Kepçe 18 Temel Rozet Kataloğu

INSERT INTO badges (slug, name, description, category, karma_reward, icon, tier, is_hidden, is_repeatable)
VALUES
    ('ilk_kepce', 'İlk Kepçe', 'Menü yayınlandığı an ilk 5 dakika içinde ilk yorumu giren işsiz.', 'sadakat', 10, 'stopwatch', 'other', false, true),
    ('demir_mide', 'Demir Mide', '7 gün üst üste siteye giriş yapan felaketzede.', 'sadakat', 20, 'pepper', 'bronze', false, false),
    ('kurumsal_caresizlik', 'Kurumsal Çaresizlik', '30 gün üst üste siteye giren amansız.', 'sadakat', 50, 'calendar', 'silver', false, false),
    ('stokholm_sendromu', 'Stokholm Sendromu', '100 gün üst üste siteye giren yurt kuşu.', 'sadakat', 100, 'bed', 'silver', false, false),
    ('demirbas', 'Demirbaş', 'Bir eğitim-öğretim dönemi boyunca (ekim-haziran) üst üste siteye giren gariban.', 'sadakat', 200, 'stamp', 'gold', false, true),
    ('hucre_hapsi', 'Hücre Hapsi', '365 gün aralıksız siteye giren hayatsız.', 'sadakat', 300, 'building', 'gold', false, false),
    ('vefakar', 'Vefakar', 'Bir ay boyunca bir şehirdeki tüm menülere upvote veren polyanna.', 'sadakat', 150, 'starFilled', 'gold', false, true),
    ('klavyesor', 'Klavyeşör', 'Toplam 100 yoruma ulaşan.', 'sosyal', 30, 'keyboard', 'bronze', false, false),
    ('halkin_adami', 'Halkın Adamı', 'Tek bir yorumuyla 50 upvote alan.', 'sosyal', 50, 'voteUpFilled', 'silver', false, true),
    ('muzmin_muhalif', 'Müzmin Muhalif', 'Üst üste herhangi bir şeye 50 downvote atan.', 'sosyal', 15, 'voteDownFilled', 'other', true, false),
    ('kanaat_onderi', 'Kanaat Önderi', 'Toplam 500 upvote alan.', 'sosyal', 150, 'trophy', 'gold', false, false),
    ('linc_kurbani', 'Linç Kurbanı', 'Tek bir yorumunda 50 downvote yiyen.', 'sosyal', 15, 'warning', 'other', true, true),
    ('caylak_gammaz', 'Çaylak Gammaz', 'İlk şikayetini/raporunu yapan hevesli ispiyoncu.', 'denetim', 10, 'search', 'other', false, false),
    ('fahri_mufettis', 'Fahri Müfettiş', '10 başarılı şikayetle sanal egosu okşanan.', 'denetim', 100, 'policeBadge', 'silver', false, false),
    ('kacak_asci', 'Kaçak Aşçı', 'İlk menü fotoğrafı veya veri girdisi onaylanan gizli kahraman.', 'veri', 25, 'incognito', 'other', false, false),
    ('bas_muhbir', 'Baş Muhbir', '10 onaylanan menü girişi yapan veri kaçakçısı.', 'veri', 75, 'upload', 'silver', false, false),
    ('bakanlik_ajani', 'Bakanlık Ajanı', '50 onaylanan menü girişi yapan yurt istihbaratı.', 'veri', 250, 'eyeLooking', 'gold', false, false),
    ('derin_devlet', 'Derin Devlet', 'Kendi yurt veya şehrinde menüyü ilk kez sisteme kazandıran öncü.', 'veri', 100, 'key', 'silver', false, false)
ON CONFLICT (slug) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    category = EXCLUDED.category,
    karma_reward = EXCLUDED.karma_reward,
    icon = EXCLUDED.icon,
    tier = EXCLUDED.tier,
    is_hidden = EXCLUDED.is_hidden,
    is_repeatable = EXCLUDED.is_repeatable;
