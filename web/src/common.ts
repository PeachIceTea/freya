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
			document.title = title
		} else {
			document.title = `${t(title, props)} - ${t("app--title")}`
		}
	}, [t, title, props, raw])
}
