<script>
    import { page } from "$app/stores";
    import { globalState, authActions } from "../../state.svelte.js";

    import { icon } from "../ui/icons.js";
    import { api } from "../../api/index.js";
    import { onMount, onDestroy } from "svelte";
    import { getCookie } from "../../utils/cookie.js";

    const LINKS = [
        { path: "/", label: "Günlük", id: "nav-daily" },
        { path: "/arsiv", label: "Arşiv", id: "nav-archive" },
    ];

    let user = $derived(globalState?.user || null);
    let isModerator = $derived(globalState?.isModerator || false);

    // Oturum çerezi var ama kullanıcı henüz yüklenmediyse "Yükleniyor..."
    // göster. Eğer API isteği ağ hatası nedeniyle başarısız olursa (örneğin
    // anlık kopukluk), state'deki hasSession `true` kalmaya devam edeceği için
    // kullanıcı yanlışlıkla "Misafir" (ve giriş yap/kayıt ol butonları) görmez.
    // Auth hatası durumunda çerez ve hasSession temizlendiğinden sorun çıkmaz.
    let isLoading = $derived(!user && globalState.hasSession);

    let dropdownOpen = $state(false);
    let menuDropdownOpen = $state(false);
    let activePath = $derived($page.url.pathname);
    let shakingId = $state(null);
    let shakeTimeout = null;
    let hasUnreadNotifications = $state(false);

    async function checkNotifications() {
        if (!user) return;
        try {
            const notifications = await api.getNotifications();
            hasUnreadNotifications = notifications.some((n) => !n.is_read);
        } catch (err) {
            console.error(err);
        }
    }

    onMount(() => {
        const closeDropdown = () => {
            dropdownOpen = false;
            menuDropdownOpen = false;
        };
        window.addEventListener("scroll", closeDropdown, { passive: true });

        // Dropdown.svelte veya başka bir menü açıldığında Nav menülerini kapat
        const onExternalDropdown = () => {
            dropdownOpen = false;
            menuDropdownOpen = false;
        };
        window.addEventListener("kepce:dropdown-open", onExternalDropdown);

        return () => {
            window.removeEventListener("scroll", closeDropdown);
            window.removeEventListener(
                "kepce:dropdown-open",
                onExternalDropdown,
            );
        };
    });

    $effect(() => {
        if (user) {
            checkNotifications();
        } else {
            hasUnreadNotifications = false;
        }
    });

    function toggleDropdown(e) {
        e.stopPropagation();
        dropdownOpen = !dropdownOpen;
        if (dropdownOpen) {
            menuDropdownOpen = false;
            window.dispatchEvent(new CustomEvent("kepce:nav-menu-open"));
        }
    }

    function toggleMenuDropdown(e) {
        e.stopPropagation();
        menuDropdownOpen = !menuDropdownOpen;
        if (menuDropdownOpen) {
            dropdownOpen = false;
            window.dispatchEvent(new CustomEvent("kepce:nav-menu-open"));
        }
    }

    let userContainer = $state();
    let menuContainer = $state();

    function closeDropdownOnOutsideClick(e) {
        if (
            dropdownOpen &&
            userContainer &&
            !userContainer.contains(e.target)
        ) {
            dropdownOpen = false;
        }
        if (
            menuDropdownOpen &&
            menuContainer &&
            !menuContainer.contains(e.target)
        ) {
            menuDropdownOpen = false;
        }
    }

    function handleKeydown(e) {
        if (e.key === "Escape") {
            dropdownOpen = false;
            menuDropdownOpen = false;
        }
    }

    function logout() {
        dropdownOpen = false;
        authActions.logout();
    }

    // Direct DOM manipulation for the shake effect without forced reflow
    function shakeElement(node) {
        node.classList.remove("anim-shake"); // Reset if already shaking
        requestAnimationFrame(() => {
            node.classList.add("anim-shake");
        });

        if (node._shakeTimeout) clearTimeout(node._shakeTimeout);
        node._shakeTimeout = setTimeout(() => {
            node.classList.remove("anim-shake");
        }, 350);
    }
</script>

<svelte:window
    onclick={closeDropdownOnOutsideClick}
    onkeydown={handleKeydown}
/>

<a
    class="nav-bar__brand"
    href="/"
    id="nav-brand"
    data-link
    onclick={(e) => {
        if (activePath === "/") {
            e.preventDefault();
        }
    }}
>
    <div class="nav-bar__logo-large">
        {@html icon("logoExperimental", null, "Kepçe Logosu")}
    </div>
    <div class="nav-bar__logo-small">
        {@html icon("logoSmallExperimental", null, "Kepçe Logosu")}
    </div>
</a>

<div class="nav-bar__links">
    {#each LINKS as l}
        <a
            class="nav-bar__link {activePath === l.path
                ? 'nav-bar__link--active'
                : ''}"
            id={l.id}
            href={l.path}
            onclick={(e) => {
                if (activePath === l.path) {
                    e.preventDefault();
                    shakeElement(e.currentTarget);
                }
            }}
            data-link
        >
            {l.label}
        </a>
    {/each}
</div>

<div class="nav-bar__actions">
    <div class="nav-bar__menu-container" bind:this={menuContainer}>
        <button
            class="nav-bar__menu-btn"
            aria-label="menü"
            aria-expanded={menuDropdownOpen}
            onclick={toggleMenuDropdown}
        >
            {@html icon("menuHamburger", 24)}
        </button>
        <div
            class="c-menu {menuDropdownOpen ? 'c-menu--open' : ''}"
            id="nav-main-dropdown"
            aria-hidden={!menuDropdownOpen}
        >
            {#each LINKS as l}
                <a
                    href={l.path}
                    class="c-menu__item"
                    data-link
                    onclick={(e) => {
                        menuDropdownOpen = false;
                        if (activePath === l.path) {
                            e.preventDefault();
                        }
                    }}>{l.label}</a
                >
            {/each}
        </div>
    </div>

    <div class="nav-bar__user-container" bind:this={userContainer}>
        <button
            class="nav-bar__user"
            id="nav-user-btn"
            aria-label="Kullanıcı Menüsü"
            onclick={toggleDropdown}
        >
            <div class="nav-bar__user-avatar-container">
                <div class="nav-bar__user-avatar">
                    {#if user?.avatar_url}
                        <img
                            class="nav-bar__user-avatar-img"
                            src={api.getAvatarUrl(user.avatar_url)}
                            alt="Profil"
                            onerror={(e) => (e.target.style.display = "none")}
                        />
                    {:else}
                        {@html icon("avatarEmpty", 38)}
                    {/if}
                </div>
                {#if hasUnreadNotifications}
                    <div class="nav-bar__user-notification-dot"></div>
                {/if}
            </div>
            <span class="nav-bar__user-name">
                {#if user}
                    {user.username}
                {:else if isLoading}
                    Yükleniyor...
                {:else}
                    Misafir
                {/if}
            </span>
        </button>

        <div
            class="c-menu {dropdownOpen ? 'c-menu--open' : ''}"
            id="nav-user-dropdown"
            aria-hidden={!dropdownOpen}
        >
            {#if !globalState.hasSession}
                <a
                    href="/giris"
                    class="c-menu__item"
                    id="dropdown-login"
                    data-link
                    onclick={() => (dropdownOpen = false)}>Giriş yap</a
                >
                <a
                    href="/kayit"
                    class="c-menu__item"
                    id="dropdown-register"
                    data-link
                    onclick={() => (dropdownOpen = false)}>Kayıt ol</a
                >
                <div class="c-menu__divider"></div>
            {/if}

            {#if globalState.hasSession}
                <a
                    href={user ? `/biri/${user.username}` : "#"}
                    class="c-menu__item"
                    id="dropdown-profile"
                    data-link
                    onclick={() => (dropdownOpen = false)}>Profilim</a
                >
                <a
                    href="/bildirimler"
                    class="c-menu__item"
                    id="dropdown-notifications"
                    data-link
                    onclick={() => (dropdownOpen = false)}
                >
                    Bildirimler
                    {#if hasUnreadNotifications}
                        <span class="nav-dropdown-badge u-ml-auto">1</span>
                    {/if}
                </a>

                {#if isModerator}
                    <div class="c-menu__divider"></div>
                    <a
                        href="/moderasyon"
                        class="c-menu__item"
                        id="dropdown-mod"
                        data-link
                        onclick={() => (dropdownOpen = false)}
                    >
                        Moderasyon
                    </a>
                {/if}
                {#if globalState.devMode}
                    <a
                        href="/gelistirici"
                        class="c-menu__item"
                        id="dropdown-developer"
                        data-link
                        onclick={() => (dropdownOpen = false)}>Geliştirici</a
                    >
                    <div class="c-menu__divider"></div>
                {/if}
            {/if}

            <a
                href="/ayarlar"
                class="c-menu__item"
                id="dropdown-settings"
                data-link
                onclick={() => (dropdownOpen = false)}>Ayarlar</a
            >

            {#if globalState.hasSession}
                <div class="c-menu__divider"></div>
                <button
                    class="c-menu__item c-menu__item--danger"
                    id="dropdown-auth-action"
                    onclick={logout}
                >
                    Çıkış yap
                </button>
            {/if}
        </div>
    </div>
</div>
