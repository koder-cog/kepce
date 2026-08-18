import { api } from "@/api/index.js";
import { getCurrentCity, setCurrentCity, getCitiesData } from "@/stores/city.svelte.js";
import { wait, getDuration, runNextTick, isMotionEnabled } from "@/lib/dom/motion.js";


export function createTimelineStore() {
    let cities = $state([]);
    let selectedDate = $state(new Date());
    let viewMonth = $state(new Date().getMonth());
    let viewYear = $state(new Date().getFullYear());
    let viewType = $state("timeline");
    let currentDietMode = $state(
        typeof window !== 'undefined' ? localStorage.getItem("kepce_diet_mode") || "standard" : "standard"
    );

    let isLoading = $state(false);
    let errorState = $state(null);
    let menusState = $state([]);
    // Prerender sırasında enjekte edilen veriyi takip et:
    // init() çağrıldığında aynı city/date ise tekrar API'ye gitmez.
    let prerenderedMeta = null;
    let currentLoadToken = 0;
    let currentCity = $derived(getCurrentCity());

    // Görev #21-24: Ay bazlı çölyak menüsü mevcudiyeti.
    // "city:YYYY:M" → boolean. Problar aylık endpoint ile yapılır ve
    // önbelleğe alınır (her gün değişiminde tekrar istek atılmaz).
    let celiacAvailability = $state({});
    let celiacProbeKey = null;
    // Kullanıcı URL (?diyet=celiac) ile zorladıysa true olur; bu durumda
    // çölyaksız ayda bile otomatik standart'a dönülmez, empty-state
    // gösterilir (#24). Kullanıcı selector'den seçim yapınca sıfırlanır.
    let dietForcedViaUrl = $state(false);

    const START_YEAR = 2026; // Kullanıcı isteği doğrultusunda başlangıç Ocak 2026
    const MAY_2026 = new Date(2026, 4, 1);
    
    // Derived states
    let cityHasCeliac = $derived(cities.find(c => c.slug === currentCity)?.has_celiac ?? false);
    let viewMonthHasCeliac = $derived.by(() => {
        if (!currentCity) return undefined;
        return celiacAvailability[`${currentCity}:${selectedDate.getFullYear()}:${selectedDate.getMonth()}`];
    });
    // #21: Şehir çölyak desteklese bile o ay çölyak menüsü yoksa
    // selector gizlenir. Görev #26: Probu sonucu henüz bilinmiyorsa (undefined)
    // yanlış pozitif (layout shift) olmaması için gizli tutulur.
    let isDietVisible = $derived(cityHasCeliac && viewMonthHasCeliac === true);
    let daysInMonth = $derived(new Date(viewYear, viewMonth + 1, 0).getDate());
    let firstDayOffset = $derived((new Date(viewYear, viewMonth, 1).getDay() + 6) % 7); // 0 is Monday
    let breakfastData = $derived(menusState.filter((m) => m.meal_type === "breakfast"));
    let dinnerData = $derived(menusState.filter((m) => m.meal_type === "dinner"));
    let isWeekend = $derived(selectedDate.getDay() === 0 || selectedDate.getDay() === 6);
    let isBeforeMay2026 = $derived(selectedDate < MAY_2026);
    let breakfastEnd = $derived(isWeekend ? "12:30" : "12:00");
    let dinnerEnd = $derived(isBeforeMay2026 ? "22:00" : "23:00");



    let loadDebounceTimer = null;

    async function loadMenus(debounceMs = 0) {
        if (!currentCity) return;

        errorState = null;
        const token = ++currentLoadToken;
        
        if (loadDebounceTimer) clearTimeout(loadDebounceTimer);

        const executeLoad = async () => {
            if (currentLoadToken !== token) return;
            
            let timeoutId = setTimeout(() => {
                if (currentLoadToken === token) {
                    isLoading = true;
                }
            }, 150);

            const year = selectedDate.getFullYear();
            const month = String(selectedDate.getMonth() + 1).padStart(2, "0");
            const day = String(selectedDate.getDate()).padStart(2, "0");
            const dateQuery = `${year}-${month}-${day}`;

            try {
                const menus = await api.getMenusByDate(currentCity, dateQuery, currentDietMode);
                if (currentLoadToken !== token) return;
                menusState = menus || [];
            } catch (err) {
                if (currentLoadToken !== token) return;
                console.error("loadMenus failed:", err);
                errorState = {
                    statusCode: parseInt(err.message.match(/\d{3}/)?.[0]) || 500,
                    desc: err.message,
                };
            } finally {
                clearTimeout(timeoutId);
                if (currentLoadToken === token) {
                    isLoading = false;
                }
            }
        };

        if (debounceMs > 0) {
            loadDebounceTimer = setTimeout(executeLoad, debounceMs);
        } else {
            executeLoad();
        }
    }

    async function updateView(newMonth, newYear, newType, debounceMs = 0) {
        if (newMonth !== undefined) viewMonth = newMonth;
        if (newYear !== undefined) viewYear = newYear;
        if (newType !== undefined) viewType = newType;
    }

    function prevMonth() {
        if (viewYear === START_YEAR && viewMonth <= 0) return;
        if (viewYear < START_YEAR) return;
        let m = viewMonth - 1;
        let y = viewYear;
        if (m < 0) {
            m = 11;
            y--;
        }
        updateView(m, y, viewType, 250);
        if (y === new Date().getFullYear() && m === new Date().getMonth()) {
            selectDate(new Date().getDate(), 250);
        } else {
            selectDate(1, 250);
        }
    }

    function nextMonth() {
        if (viewYear >= new Date().getFullYear() && viewMonth >= new Date().getMonth()) return;
        let m = viewMonth + 1;
        let y = viewYear;
        if (m > 11) {
            m = 0;
            y++;
        }
        updateView(m, y, viewType, 250);
        if (y === new Date().getFullYear() && m === new Date().getMonth()) {
            selectDate(new Date().getDate(), 250);
        } else {
            selectDate(1, 250);
        }
    }

    function selectDate(day, debounceMs = 0) {
        selectedDate = new Date(viewYear, viewMonth, day);
        loadMenus(debounceMs);
        probeCeliacMonth();
        scrollToActiveDay(false);
    }

    // Timeline selector'ı tarafından çağrılır — SESSION bazlıdır.
    // currentDietMode'u bellekte günceller ancak localStorage'a yazmaz;
    // sayfa yeniden yüklendiğinde kalıcı ayara (kepce_diet_mode) döner.
    function selectDietMode(mode) {
        dietForcedViaUrl = false; // Kullanıcı bilinçli seçim yaptı
        if (currentDietMode === mode) return false;
        currentDietMode = mode;
        loadMenus(0);
        return true;
    }

    // Ayarlar sayfasındaki "Çölyak dostu mod" toggle'ı tarafından çağrılır —
    // KALICIdır. currentDietMode'u günceller VE localStorage'a yazar.
    function setPermanentDietMode(mode) {
        dietForcedViaUrl = false;
        if (currentDietMode !== mode) {
            currentDietMode = mode;
            loadMenus(0);
        }
        if (typeof window !== 'undefined') {
            localStorage.setItem("kepce_diet_mode", mode);
        }
    }

    // #24: URL üzerinden (?diyet=celiac) zorlanan diyet modu.
    // Ay probu başarısız olsa bile mod korunur; TimelineView'daki
    // "Bugün çölyak menüsü yok" empty-state'i devreye girer.
    function forceDietMode(mode) {
        dietForcedViaUrl = true;
        if (currentDietMode === mode) return;
        currentDietMode = mode;
        if (typeof window !== 'undefined') {
            localStorage.setItem("kepce_diet_mode", mode);
        }
        loadMenus(0);
    }

    // Görüntülenen ayda çölyak menüsü olup olmadığını aylık endpoint'ten
    // problar. Sonuç önbelleğe yazılır ve #22 kuralı uygulanır.
    async function probeCeliacMonth() {
        if (!currentCity || cities.length === 0) return;
        const key = `${currentCity}:${selectedDate.getFullYear()}:${selectedDate.getMonth()}`;
        if (celiacProbeKey === key) return;
        celiacProbeKey = key;

        const cityObj = cities.find(c => c.slug === currentCity);
        if (!cityObj?.has_celiac) {
            celiacAvailability[key] = false;
            applyCeliacRules(key);
            return;
        }
        if (celiacAvailability[key] !== undefined) {
            applyCeliacRules(key);
            return;
        }

        try {
            const menus = await api.getMonthlyMenus(
                currentCity,
                selectedDate.getFullYear(),
                selectedDate.getMonth() + 1,
                'celiac'
            );
            celiacAvailability[key] =
                Array.isArray(menus) && menus.some(m => m.items?.length > 0);
        } catch {
            // Probu başarısız: yanlış negatif üretmemek için o ay
            // bilinmiyor olarak kalsın, bir sonraki gezinimde tekrar denenir.
            celiacProbeKey = null;
            return;
        }
        applyCeliacRules(key);
    }

    function applyCeliacRules(key) {
        // #22: Çölyaklı menüden çölyaksız bir aya geçildiyse buton
        // gizlenir ve standart menüye otomatik dönülür. #23: hedef ayda
        // da çölyak menüsü varsa mod korunur (burada bir şey yapılmaz).
        if (celiacAvailability[key] === false && currentDietMode === 'celiac' && !dietForcedViaUrl) {
            selectDietMode('standard');
        }
    }

    function scrollToActiveDay(forceCenter = false) {
        if (typeof window === "undefined") return;
        runNextTick(() => {
            setTimeout(() => {
                window.dispatchEvent(new CustomEvent('timeline-scroll-to-active', { detail: { forceCenter } }));
            }, getDuration(50));
        });
    }

    /**
     * SSR/prerender sırasında çağrılır (onMount'tan ÖNCE).
     * Menü verisini store'a enjekte eder → HTML'de menü kartları render edilir.
     */
    function setPrerenderedData(menus, city, dateStr) {
        menusState = Array.isArray(menus) ? menus : [];
        prerenderedMeta = { city, date: dateStr };
        if (dateStr) {
            const parts = dateStr.split('-').map(Number);
            if (parts.length === 3 && !isNaN(parts[0]) && !isNaN(parts[1]) && !isNaN(parts[2])) {
                selectedDate = new Date(parts[0], parts[1] - 1, parts[2]);
                viewMonth = parts[1] - 1;
                viewYear = parts[0];
            }
        }
    }

    async function init() {
        // Prerender verisi mevcut city/date ile eşleşiyorsa ilk açılışta tekrar API'ye gitme
        const today = new Date();
        const todayStr = `${today.getFullYear()}-${String(today.getMonth() + 1).padStart(2, '0')}-${String(today.getDate()).padStart(2, '0')}`;
        const skipInitialLoad = prerenderedMeta
            && prerenderedMeta.city === currentCity
            && prerenderedMeta.date === todayStr;

        if (!skipInitialLoad) {
            loadMenus();
        }
        // Prerender meta'yı temizle — sonraki tarih/şehir değişimlerinde tekrar çekilsin
        prerenderedMeta = null;

        try {
            cities = await getCitiesData();
        } catch {
            cities = [{ id: 1, name: "İstanbul", slug: "istanbul", has_celiac: false }];
        }

        const initialCityObj = cities.find(c => c.slug === currentCity);
        if (initialCityObj && !initialCityObj.has_celiac && currentDietMode === 'celiac') {
             currentDietMode = 'standard';
             if (typeof window !== 'undefined') localStorage.setItem("kepce_diet_mode", "standard");
        }

        // Açılışta ana içeriğin (LCP) boyanmasını engellememek için çölyak kontrolünü idle anına ertele
        if (typeof window !== 'undefined' && 'requestIdleCallback' in window) {
            window.requestIdleCallback(() => probeCeliacMonth(), { timeout: 3000 });
        } else {
            setTimeout(() => probeCeliacMonth(), 1000);
        }
        scrollToActiveDay(true);
    }

    function setCity(slug) {
        setCurrentCity(slug);
        
        const newCityObj = cities.find(c => c.slug === slug);
        if (newCityObj && !newCityObj.has_celiac && currentDietMode === 'celiac') {
             selectDietMode('standard');
        } else {
             loadMenus();
        }
        
        probeCeliacMonth();
        scrollToActiveDay(true);
    }


    return {
        get cities() { return cities; },
        get selectedDate() { return selectedDate; },
        get viewMonth() { return viewMonth; },
        get viewYear() { return viewYear; },
        get viewType() { return viewType; },
        get currentDietMode() { return currentDietMode; },
        get viewMonthHasCeliac() { return viewMonthHasCeliac; },
        get dietForcedViaUrl() { return dietForcedViaUrl; },
        get canPrevMonth() { return viewYear > START_YEAR || (viewYear === START_YEAR && viewMonth > 0); },
        get canNextMonth() { return viewYear < new Date().getFullYear() || (viewYear === new Date().getFullYear() && viewMonth < new Date().getMonth()); },

        get isLoading() { return isLoading; },
        get errorState() { return errorState; },
        get currentCity() { return currentCity; },
        get isDietVisible() { return isDietVisible; },
        get daysInMonth() { return daysInMonth; },
        get firstDayOffset() { return firstDayOffset; },
        get breakfastData() { return breakfastData; },
        get dinnerData() { return dinnerData; },
        get breakfastEnd() { return breakfastEnd; },
        get dinnerEnd() { return dinnerEnd; },

        set currentCity(val) { setCity(val); },
        set viewType(val) { updateView(undefined, undefined, val); },
        
        init,
        prevMonth,
        nextMonth,
        selectDate,
        selectDietMode,
        setPermanentDietMode,
        forceDietMode,
        updateView,
        setPrerenderedData,
        scrollToActiveDay
    };
}

export const timelineState = createTimelineStore();
