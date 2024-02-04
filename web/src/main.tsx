import { StrictMode } from "react"
import ReactDOM from "react-dom/client"

import { checkSession } from "./api/authentication"
import App from "./layout/App"
import "./styles/index.scss"

async function main() {
	await checkSession() // TODO: Show a loading screen while this is happening.

	ReactDOM.createRoot(document.getElementById("root")!).render(
		<StrictMode>
			<App />
		</StrictMode>,
	)
}

main()
