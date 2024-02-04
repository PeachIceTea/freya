import { FluentValue } from "@fluent/bundle"
import { useEffect } from "react"
import { useSearch } from "wouter"

import { useLocale } from "./locales"

export function useQuery() {
	const search = useSearch()
	const params = new URLSearchParams(search)
	return params
}

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
