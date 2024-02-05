import { FluentValue } from "@fluent/bundle"
import { useEffect, useState } from "react"
import { useSearch } from "wouter"
import { z } from "zod"

import { useLocale } from "./locales"
import { useStore } from "./store"

// Get the query parameters from the URL.
export function useQuery() {
	const search = useSearch()
	const params = new URLSearchParams(search)
	return params
}

// Set the title of the page.
export function useTitle(
	title: string,
	props?: Record<string, FluentValue>,
	raw = false,
) {
	const t = useLocale()

	useEffect(() => {
		if (raw) {
			document.title = `${title} - ${t("app--title")}`
		} else {
			document.title = `${t(title, props)} - ${t("app--title")}`
		}
	}, [t, title, props, raw])
}

// Format seconds to MM:SS or HH:MM:SS depending if the duration is longer than an hour.
export function formatDuration(seconds: number) {
	const hours = Math.floor(seconds / 3600)
	const minutes = Math.floor((seconds % 3600) / 60)
	const secs = Math.ceil(seconds % 60)

	const hoursStr = hours > 0 ? `${String(hours).padStart(2, "0")}:` : ""
	const minutesStr = `${String(minutes).padStart(2, "0")}:`
	const secondsStr = String(secs).padStart(2, "0")

	return `${hoursStr}${minutesStr}${secondsStr}`
}

// Theme switcher
const ThemeSchema = z.enum(["light", "dark", "system"])
export type Theme = z.infer<typeof ThemeSchema>
const ThemeLocalStorageKey = "freya-theme"

export function getThemeFromLocalStorage() {
	let theme = "system" as Theme

	const localStorageTheme = localStorage.getItem(ThemeLocalStorageKey)
	if (localStorageTheme) {
		try {
			theme = ThemeSchema.parse(localStorageTheme)
		} catch {
			localStorage.removeItem(ThemeLocalStorageKey)
		}
	}

	return theme
}

type ThemePreference = "dark" | "light" | "no-preference"
export function useTheme() {
	// Get the system theme preference.
	const [preferredTheme, setPreferredTheme] = useState<ThemePreference>(() => {
		const isDark = window.matchMedia("(prefers-color-scheme: dark)")
		const isLight = window.matchMedia("(prefers-color-scheme: light)")
		return isDark.matches ? "dark" : isLight.matches ? "light" : "no-preference"
	})

	// Get the theme preference from the store.
	// The theme is decided in this order:
	// 1. Theme set by user manually.
	// 2. Theme preference declared by the user's system.
	// 3. Default to dark theme.
	const state = useStore()
	const theme =
		state.theme !== "system"
			? state.theme
			: preferredTheme !== "no-preference"
				? preferredTheme
				: "dark"
	document.documentElement.setAttribute("data-bs-theme", theme)

	// Setup event listeners to update the theme preference.
	useEffect(() => {
		const darkListener = ({ matches }: MediaQueryListEvent) => {
			matches && setPreferredTheme("dark")
		}

		const lightListener = ({ matches }: MediaQueryListEvent) => {
			matches && setPreferredTheme("light")
		}

		const isDark = window.matchMedia("(prefers-color-scheme: dark)")
		const isLight = window.matchMedia("(prefers-color-scheme: light)")

		isDark.addEventListener("change", darkListener)
		isLight.addEventListener("change", lightListener)

		return () => {
			isDark.removeEventListener("change", darkListener)
			isLight.removeEventListener("change", lightListener)
		}
	}, [])

	// Function to set user's theme preference.
	const setTheme = (theme: Theme) => {
		localStorage.setItem(ThemeLocalStorageKey, theme)
		state.setTheme(theme)
	}

	return [theme, setTheme] as const
}
