-- 05_badges.sql
-- Kepçe 18 Temel Rozet Kataloğu

INSERT INTO badges (slug, name, description, category, karma_reward, icon, is_repeatable)
VALUES
    ('ilk_kepce', 'İlk Kepçe', 'Menü yayınlandığı an ilk 5 dakika içinde ilk yorumu giren işsiz.', 'sadakat', 25, 'soup', true),
    ('demir_mide', 'Demir Mide', '7 gün üst üste siteye giriş yapan felaketzede.', 'sadakat', 50, 'shield', false),
    ('kurumsal_caresizlik', 'Kurumsal Çaresizlik', '30 gün üst üste siteye giren amansız.', 'sadakat', 150, 'calendar', false),
    ('stokholm_sendromu', 'Stokholm Sendromu', '100 gün üst üste siteye giren yurt kuşu.', 'sadakat', 400, 'lock', false),
    ('demirbas', 'Demirbaş', 'Bir eğitim-öğretim dönemi boyunca (ekim-haziran) üst üste siteye giren gariban.', 'sadakat', 750, 'crown', true),
    ('hucre_hapsi', 'Hücre Hapsi', '365 gün (1 yıl) aralıksız siteye giren hayatsız.', 'sadakat', 1500, 'lock', false),
    ('vefakar', 'Vefakar', 'Bir ay boyunca bir şehirdeki tüm menülere upvote veren polyanna.', 'sadakat', 100, 'starFilled', true),
    ('klavyesor', 'Klavyeşör', 'Toplam 100 yoruma ulaşan.', 'sosyal', 100, 'chat', false),
    ('halkin_adami', 'Halkın Adamı', 'Tek bir yorumuyla 50 upvote alan.', 'sosyal', 200, 'voteUpFilled', true),
    ('muzmin_muhalif', 'Müzmin Muhalif', 'Üst üste herhangi bir şeye 50 downvote atan.', 'sosyal', 50, 'voteDown', false),
    ('kanaat_onderi', 'Kanaat Önderi', 'Toplam 500 upvote alan.', 'sosyal', 500, 'crown', false),
    ('linc_kurbani', 'Linç Kurbanı', 'Tek bir yorumunda 50 downvote yiyen.', 'sosyal', 75, 'warning', true),
    ('caylak_gammaz', 'Çaylak Gammaz', 'İlk şikayetini/raporunu yapan hevesli ispiyoncu.', 'denetim', 25, 'search', false),
    ('fahri_mufettis', 'Fahri Müfettiş', '10 başarılı şikayetle sanal egosu okşanan.', 'denetim', 500, 'check-circle', false),
    ('kacak_asci', 'Kaçak Aşçı', 'İlk menü fotoğrafı veya veri girdisi onaylanan gizli kahraman.', 'veri', 50, 'utensils', false),
    ('bas_muhbir', 'Baş Muhbir', '10 onaylanan menü girişi yapan veri kaçakçısı.', 'veri', 250, 'database', false),
    ('bakanlik_ajani', 'Bakanlık Ajanı', '50 onaylanan menü girişi yapan yurt istihbaratı.', 'veri', 1000, 'shield', false),
    ('derin_devlet', 'Derin Devlet', 'Kendi yurt veya şehrinde menüyü ilk kez sisteme kazandıran öncü.', 'veri', 300, 'compass', false)
ON CONFLICT (slug) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    category = EXCLUDED.category,
    karma_reward = EXCLUDED.karma_reward,
    icon = EXCLUDED.icon,
    is_repeatable = EXCLUDED.is_repeatable;
