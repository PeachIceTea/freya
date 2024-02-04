import { FluentBundle, FluentResource } from "@fluent/bundle"
import { negotiateLanguages } from "@fluent/langneg"
import { ReactLocalization, useLocalization } from "@fluent/react"

import FluentEn from "../locales/en.ftl?raw"

const RESOURCES = {
	en: new FluentResource(FluentEn),
}

const availableLocales = Object.keys(RESOURCES)

function* generateBundles(userLocales: readonly string[]) {
	// Choose locales that are best for the user.
	const currentLocales = negotiateLanguages(userLocales, availableLocales, {
		defaultLocale: "en",
	})

	for (const locale of currentLocales) {
		const bundle = new FluentBundle(locale)

		if (!availableLocales.includes(locale)) {
			throw new Error(`Locale ${locale} is not available`)
		}

		bundle.addResource(RESOURCES[locale as keyof typeof RESOURCES])
		yield bundle
	}
}

export default new ReactLocalization(generateBundles(navigator.languages))

export function useLocale() {
	const { l10n } = useLocalization()
	return l10n.getString.bind(l10n)
}
