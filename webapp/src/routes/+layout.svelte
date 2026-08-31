<script>
	import "../styles/main.css";
	import { onMount, onDestroy } from "svelte";
	import { fade } from "svelte/transition";
	import { beforeNavigate, afterNavigate, onNavigate, goto } from "$app/navigation";
	import { page, updated } from "$app/stores";
	import { globalState, authActions } from "@/state.svelte.js";
	import Nav from "@/components/layout/Nav.svelte";
	import Footer from "@/components/layout/Footer.svelte";
	import ToastContainer from "@/components/ui/ToastContainer.svelte";
	import AnnouncementBanner from "@/components/ui/AnnouncementBanner.svelte";
	import VerificationBanner from "@/components/ui/VerificationBanner.svelte";
	import holidays from "$lib/data/holidays.json";
	import { initScrollbar } from "@/lib/dom/scrollbar.js";
	import { initTooltipManager } from "@/lib/dom/tooltips.js";
	import { openMenuReportModal } from "@/components/features/report-modal.js";
	import ExternalLinkWarningModal from "@/components/features/ExternalLinkWarningModal.svelte";
	import OfflineBanner from "@/components/ui/OfflineBanner.svelte";

	let { children } = $props();

	const today = new Date();
	const month = String(today.getMonth() + 1).padStart(2, "0");
	const day = String(today.getDate()).padStart(2, "0");
	const year = today.getFullYear();

	const mmdd = `${month}-${day}`;
	const yyyymmdd = `${year}-${month}-${day}`;

	const currentHoliday = holidays[yyyymmdd] || holidays[mmdd];

	let navHeight = $state(64);

	let externalLinkModalOpen = $state(false);
	let pendingExternalUrl = $state("");

	function handleGlobalClick(e) {
		const a = e.target.closest("a");
		if (!a || !a.href) return;

		try {
			const url = new URL(a.href);
			if (url.protocol === "http:" || url.protocol === "https:") {
				if (url.hostname !== window.location.hostname) {
					const warningEnabled =
						!globalState.isApp &&
						localStorage.getItem("kepce_external_link_warning") !==
						"false";
					if (warningEnabled) {
						e.preventDefault();
						pendingExternalUrl = a.href;
						externalLinkModalOpen = true;
					} else {
						if (a.target !== "_blank") {
							e.preventDefault();
							window.open(
								a.href,
								"_blank",
								"noopener,noreferrer",
							);
						}
					}
				}
			}
		} catch (err) {
			// ignore invalid URLs
		}
	}

	import { nativeBridge } from "@/lib/native/bridge.js";

	function getRouteMetadata(pathname) {
		if (!pathname || pathname === "/" || pathname === "") {
			return { title: "Günün Menüsü", isRoot: true, hideBottomNav: false };
		}
		if (pathname === "/arsiv" || pathname.startsWith("/arsiv/")) {
			return { title: "Menü Arşivi", isRoot: true, hideBottomNav: false };
		}
		if (pathname === "/ben") {
			return { title: "Profilim", isRoot: true, hideBottomNav: false };
		}
		if (pathname === "/ayarlar") {
			return { title: "Ayarlar", isRoot: true, hideBottomNav: false };
		}
		if (pathname === "/bildirimler") {
			return { title: "Bildirimler", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/biri/")) {
			const parts = pathname.split("/").filter(Boolean);
			const username = parts[1] || "";
			return { title: username ? `@${username}` : "Kullanıcı Profili", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/menu/") || pathname.startsWith("/yorumlar/")) {
			return { title: "Menü Detayı", isRoot: false, hideBottomNav: true };
		}
		if (pathname.startsWith("/sehirler")) {
			return { title: "Şehirler", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/kyk-yemek-saatleri")) {
			return { title: "KYK Yemek Saatleri", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/kyk-beslenme-yardimi")) {
			return { title: "Beslenme Yardımı", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/sss")) {
			return { title: "Sıkça Sorulan Sorular", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/hakkinda")) {
			return { title: "Hakkında", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/iletisim")) {
			return { title: "İletişim", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/durum")) {
			return { title: "Sistem Durumu", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/gelistirici")) {
			return { title: "Geliştirici Panosu", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/istatistikler")) {
			return { title: "İstatistikler", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/kullanim-kosullari")) {
			return { title: "Kullanım Koşulları", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/gizlilik-politikasi")) {
			return { title: "Gizlilik Politikası", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/menu-gonder")) {
			return { title: "Menü Gönder", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/giris")) {
			return { title: "Giriş Yap", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/kayit")) {
			return { title: "Hesap Oluştur", isRoot: false, hideBottomNav: false };
		}
		if (pathname.startsWith("/sifre-yenile")) {
			return { title: "Şifre Yenile", isRoot: false, hideBottomNav: false };
		}
		return { title: "Günün Menüsü", isRoot: false, hideBottomNav: false };
	}

	beforeNavigate(() => {
		// Prepare for navigation if needed
	});

	afterNavigate(({ to }) => {
		if (typeof window !== "undefined") {
			const path = to?.url?.pathname || window.location.pathname;
			const { title, isRoot, hideBottomNav } = getRouteMetadata(path);
			nativeBridge.sendRoute({
				path,
				title,
				canGoBack: !isRoot,
				isRoot,
				hideBottomNav
			});
			// Sayfa geçişinde overlay durumunu sıfırla
			nativeBridge.sendOverlayToggle(false);
		}
	});

	// Tema ve Görsel Efektler Durumunu Yerel Kabukla Senkronize Et
	$effect(() => {
		if (typeof document === "undefined") return;

		const isDark = document.documentElement.classList.contains("dark") ||
			(!document.documentElement.classList.contains("light") &&
				window.matchMedia("(prefers-color-scheme: dark)").matches);

		const effectsEnabled = localStorage.getItem("kepce_effects") !== "false";
		const bgColorHex = isDark ? "#242828" : "#F9F5E5";

		const computed = getComputedStyle(document.documentElement);
		const rawOpacity = computed.getPropertyValue("--nav-bg-opacity").trim();
		const navBgOpacity = effectsEnabled ? (parseFloat(rawOpacity) || 0.8) : 1.0;
		const colorSurface = computed.getPropertyValue("--color-surface").trim() || (isDark ? "#141414" : "#FFFFFF");
		const colorBorder = computed.getPropertyValue("--color-border").trim() || (isDark ? "#3A3D3D" : "#E0DDD0");

		nativeBridge.sendState({
			isDark,
			effectsEnabled,
			bgColorHex,
			navBgOpacity,
			colorSurface,
			colorBorder
		});
	});

	onNavigate((navigation) => {
		if (typeof document.startViewTransition !== "function") return;

		return new Promise((resolve) => {
			try {
				const transition = document.startViewTransition(async () => {
					resolve();
					await navigation.complete;
				});
				if (transition) {
					if (transition.ready) transition.ready.catch(() => {});
					if (transition.finished) transition.finished.catch(() => {});
				}
			} catch (err) {
				// Eşzamanlı (hızlı) geçişlerde viewTransition hata fırlatabilir.
				resolve();
			}
		});
	});

	// Handle initial mount effects
	beforeNavigate(({ willUnload, to }) => {
		if ($updated && !willUnload && to?.url) {
			location.href = to.url.href;
		}
	});

	let updateCheckTimer;
	onDestroy(() => {
		if (updateCheckTimer) clearInterval(updateCheckTimer);
	});

	onMount(async () => {
		updateCheckTimer = setInterval(() => {
			updated.check().catch(() => {});
		}, 15 * 60 * 1000);
		// Remove disabled state on load
		document.documentElement.classList.remove("no-js");

		const animationsEnabled =
			localStorage.getItem("kepce_animations") !== "false";
		const effectsEnabled =
			localStorage.getItem("kepce_effects") !== "false";

		document.documentElement.classList.toggle(
			"disable-animations",
			!animationsEnabled,
		);
		document.documentElement.classList.toggle(
			"disable-effects",
			!effectsEnabled,
		);

		requestAnimationFrame(() => {
			initScrollbar();
			initTooltipManager();

			// `_scrollbar.css` özel scrollbar'ın görünürlüğünü
			// `body.is-page-ready` sınıfına bağlıyor. Bu sınıf
			// daha önce hiçbir bileşenden eklenmediği için scrollbar
			// `opacity: 0` olarak kalıyordu. Burada ekleyerek
			// auto-hide (kaydırma sırasında göster, 2 sn sonra gizle)
			// mantığını çalışır hale getiriyoruz.
			document.body.classList.add("is-page-ready");
		});

		const showIndicators =
			localStorage.getItem("kepce_show_indicators") === "true";
		// CSS `body.show-indicators` seçicisi bekliyor; sınıfı hem <html>
		// hem <body> üzerinde tutmak sayfa ilk yüklendiğinde de indikatörlerin
		// doğru render edilmesini sağlar (ayarlar sayfasına gidip geri
		// dönmek gerekmez).
		document.documentElement.classList.toggle(
			"show-indicators",
			showIndicators,
		);
		document.body.classList.toggle("show-indicators", showIndicators);

		const showBot = localStorage.getItem("kepce_show_bot") !== "false";
		document.documentElement.classList.toggle("hide-ai", !showBot);

		const devMode = localStorage.getItem("kepce_dev_mode") === "true";
		globalState.devMode = devMode;

		if (typeof window !== "undefined") {
			window.__sveltekit_goto = (url) => goto(url);
			window.KepceNative = {
				goto: (url) => goto(url),
				setTitle: (title) => nativeBridge.sendTitle(title),
				onNativeEvent: (type, payload) => {
					if (type === "NAVIGATE" && payload?.url) {
						goto(payload.url);
					}
				}
			};

			window.__kepceHandleBack = () => {
				// 1. Standart Modal / Lightbox / Dialog kontrolü
				const openModal = document.querySelector(".c-modal--open, .modal.is-open, dialog[open]");
				if (openModal) {
					const escEvent = new KeyboardEvent("keydown", { key: "Escape", code: "Escape", bubbles: true, cancelable: true });
					document.dispatchEvent(escEvent);
					window.dispatchEvent(escEvent);
					return true;
				}

				// 2. Mobil Dropdown / Bottom Sheet kontrolü
				const openSheet = document.querySelector(".c-menu--modal.c-menu--open, .c-menu__overlay--open, .c-menu--open");
				if (openSheet) {
					const escEvent = new KeyboardEvent("keydown", { key: "Escape", code: "Escape", bubbles: true, cancelable: true });
					document.dispatchEvent(escEvent);
					window.dispatchEvent(escEvent);
					return true;
				}

				return false;
			};

			// Çift Kilitli Modal / Overlay İzleyicisi (Tab-Bar senkronizasyonu)
			let overlayDebounce;
			const checkOverlays = () => {
				clearTimeout(overlayDebounce);
				overlayDebounce = setTimeout(() => {
					const hasOpenOverlay = !!document.querySelector(
						".c-modal--open, .modal.is-open, dialog[open], .modal-open, .c-menu--modal.c-menu--open, .c-menu__overlay--open"
					);
					nativeBridge.sendOverlayToggle(hasOpenOverlay);
				}, 50);
			};

			const overlayObserver = new MutationObserver(checkOverlays);
			overlayObserver.observe(document.body, { attributes: true, childList: true, subtree: true, attributeFilter: ["class", "open"] });
			overlayObserver.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
		}

		await authActions.refreshUser();
	});
</script>

<svelte:window onclick={handleGlobalClick} />

<div id="app" class:is-app={globalState.isApp}>
	{#if !globalState.isApp}
		<!-- #70: Klavye kullanıcıları navigasyonu atlayabilsin -->
		<a href="#page-content" class="skip-link">Ana içeriğe geç</a>
		<nav id="main-nav" class="nav-bar" bind:clientHeight={navHeight}>
			<div class="nav-bar__inner">
				<Nav />
			</div>
			<OfflineBanner />
			{#if currentHoliday}
				<AnnouncementBanner
					id={`holiday-banner-${year}-${mmdd}`}
					text={currentHoliday.message}
					ctaText=""
					theme={currentHoliday.theme || "accent-primary"}
				/>
			{/if}
			<VerificationBanner />
		</nav>
	{:else}
		<OfflineBanner />
	{/if}
	<div
		id="page-wrapper"
		class="page-wrapper"
		style="--nav-height: {globalState.isApp ? 0 : navHeight}px"
	>
		<main id="page-content" class="page-container">
			{@render children()}
		</main>
		{#if !globalState.isApp}
			<footer id="site-footer" class="site-footer">
				<Footer />
			</footer>
		{/if}
	</div>
</div>

{#if externalLinkModalOpen}
	<ExternalLinkWarningModal
		url={pendingExternalUrl}
		onClose={() => (externalLinkModalOpen = false)}
		onContinue={() => {
			window.open(pendingExternalUrl, "_blank", "noopener,noreferrer");
			externalLinkModalOpen = false;
		}}
	/>
{/if}

<ToastContainer />

<style>
	#page-wrapper {
		padding-top: var(--nav-height, 0);
	}
	:global(html.is-app) #page-wrapper,
	:global(body.is-app) #page-wrapper {
		padding-top: 0 !important;
	}
</style>
