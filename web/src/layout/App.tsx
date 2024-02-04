import { LocalizationProvider } from "@fluent/react"

import locales from "../locales"
import FreyaNavbar from "./Navbar"
import Router from "./Router"

export default function App() {
	return (
		<LocalizationProvider l10n={locales}>
			<div id="app">
				<FreyaNavbar />
				<Router />
			</div>
		</LocalizationProvider>
	)
}
