<script>
	import "../styles/main.css";
	import { onMount } from "svelte";
	import { fade } from "svelte/transition";
	import { beforeNavigate, afterNavigate, onNavigate } from "$app/navigation";
	import { page } from "$app/stores";
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

	beforeNavigate(() => {
		// Prepare for navigation if needed
	});

	afterNavigate(() => {
		// Cleanup if needed
	});

	onNavigate((navigation) => {
		if (!document.startViewTransition) return;

		return new Promise((resolve) => {
			try {
				document.startViewTransition(async () => {
					resolve();
					await navigation.complete;
				});
			} catch (err) {
				// Eşzamanlı (hızlı) geçişlerde viewTransition hata fırlatabilir.
				resolve();
			}
		});
	});

	// Handle initial mount effects
	onMount(async () => {
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

		await authActions.refreshUser();
	});
</script>

<svelte:window onclick={handleGlobalClick} />

<div id="app">
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
	<div
		id="page-wrapper"
		class="page-wrapper"
		style="--nav-height: {navHeight}px"
	>
		<main id="page-content" class="page-container">
			{@render children()}
		</main>
		<footer id="site-footer" class="site-footer">
			<Footer />
		</footer>
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
</style>
