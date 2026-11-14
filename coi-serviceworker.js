/*
 * Cross-origin isolation for static hosts that cannot send COOP/COEP headers
 * (GitHub Pages, for example). Without those headers the browser hides
 * SharedArrayBuffer, and the multithreaded wasm build cannot start.
 *
 * The same file is the page-side loader and the service worker. It fetches
 * every same-origin response and re-sends it with the two headers added. On
 * the first visit the page reloads once, because a service worker cannot
 * intercept the navigation that registers it.
 *
 * Only used by the `threads` variant of .github/workflows/web.yml; the serial
 * build does not need it.
 */
"use strict";

const RELOAD_KEY = "coi-serviceworker-reloaded";

if (typeof window === "undefined") {
	/* ----------------------------- service worker ----------------------------- */

	self.addEventListener("install", () => {
		self.skipWaiting();
	});

	self.addEventListener("activate", (event) => {
		event.waitUntil(self.clients.claim());
	});

	self.addEventListener("fetch", (event) => {
		const request = event.request;
		if (request.cache === "only-if-cached" && request.mode !== "same-origin") {
			return;
		}
		// The application only loads same-origin resources. Cross-origin ones
		// would need their own CORP header, which is not under our control.
		if (new URL(request.url).origin !== self.location.origin) {
			return;
		}
		event.respondWith(
			(async () => {
				const response = await fetch(request);
				if (response.status === 0 || response.type === "opaque") {
					return response;
				}
				const headers = new Headers(response.headers);
				headers.set("Cross-Origin-Opener-Policy", "same-origin");
				headers.set("Cross-Origin-Embedder-Policy", "require-corp");
				headers.set("Cross-Origin-Resource-Policy", "cross-origin");
				return new Response(response.body, {
					status: response.status,
					statusText: response.statusText,
					headers,
				});
			})(),
		);
	});
} else {
	/* -------------------------------- page side ------------------------------- */

	const register = async () => {
		if (globalThis.crossOriginIsolated === true) {
			// Already isolated: nothing to do (and no reload).
			sessionStorage.removeItem(RELOAD_KEY);
			return;
		}
		if (!window.isSecureContext || !("serviceWorker" in navigator)) {
			return;
		}
		// A failed attempt must not reload forever.
		if (sessionStorage.getItem(RELOAD_KEY) === "1") {
			return;
		}
		try {
			await navigator.serviceWorker.register(
				new URL("coi-serviceworker.js", document.baseURI),
				{ scope: new URL("./", document.baseURI).pathname },
			);
			await navigator.serviceWorker.ready;
			sessionStorage.setItem(RELOAD_KEY, "1");
			location.reload();
		} catch (error) {
			console.error("Cross-origin isolation service worker:", error);
		}
	};

	void register();
}
