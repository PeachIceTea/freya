import { LocalizationProvider } from "@fluent/react"

import Player from "../Player"
import locales from "../locales"
import FreyaNavbar from "./Navbar"
import Router from "./Router"

export default function App() {
	return (
		<LocalizationProvider l10n={locales}>
			<div id="app" className="vh-100 d-flex flex-column">
				<FreyaNavbar />
				<Router />
				<Player />
			</div>
		</LocalizationProvider>
	)
}
