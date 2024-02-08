import { Container } from "react-bootstrap"

import { useTitle } from "./common"
import { useLocale } from "./locales"

export default function UserManagement() {
	useTitle("user-management--title")
	const t = useLocale()

	return (
		<Container>
			<div className="d-flex justify-content-between align-items-center">
				<h1>{t("user-management--title")}</h1>
			</div>
		</Container>
	)
}
