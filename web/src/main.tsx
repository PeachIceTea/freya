import { StrictMode } from "react"
import ReactDOM from "react-dom/client"

import { checkSession } from "./api/authentication"
import { getThemeFromLocalStorage } from "./common"
import App from "./layout/App"
import { useStore } from "./store"
import "./styles/index.scss"

async function main() {
	// Check if the user is already logged in.
	try {
		await checkSession() // TODO: Show a loading screen while this is happening.
	} catch (error) {
		console.error("Failed to check the session:", error)
	}

	// Load theme preferences from the user's local storage.
	useStore.getState().setTheme(getThemeFromLocalStorage())

	ReactDOM.createRoot(document.getElementById("root")!).render(
		<StrictMode>
			<App />
		</StrictMode>,
	)
}

main()
