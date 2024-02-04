import { useState } from "react"
import { Alert, Button, Container, FloatingLabel, Form } from "react-bootstrap"

import { getSessionInfo, login } from "./api/authentication"
import { useTitle } from "./common"
import { useLocale } from "./locales"

export default function Login() {
	useTitle("login--title")
	const t = useLocale()
	const [error, setError] = useState<string>()
	const [loading, setLoading] = useState(false)

	async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
		event.preventDefault()

		if (loading) {
			return
		}
		setLoading(true)
		setError("")

		const targets = event.target as unknown as SubmitTargets
		const username = targets.username.value
		const password = targets.password.value

		if (!username || !password) {
			setLoading(false)
			return setError("login--error-missing-credentials")
		}

		const res = await login(username, password)
		if (!res.success) {
			setError(res.error_code)
			setLoading(false)
			return
		}

		const sessionInfo = await getSessionInfo()
		if (!sessionInfo.success) {
			setError(sessionInfo.error_code)
			setLoading(false)
			return
		}

		setLoading(false)
	}

	return (
		<Container fluid="xl">
			<h1>{t("app--title")}</h1>
			{error && <Alert variant="danger">{t(error)}</Alert>}
			<Form onSubmit={handleSubmit}>
				<Form.Group className="mb-3">
					<FloatingLabel label={t("login--input-username")}>
						<Form.Control
							id="username"
							type="text"
							placeholder={t("login--input-username")}
						/>
					</FloatingLabel>
				</Form.Group>
				<Form.Group className="mb-3">
					<FloatingLabel label={t("login--input-password")}>
						<Form.Control
							id="password"
							type="password"
							placeholder={t("login--input-password")}
						/>
					</FloatingLabel>
				</Form.Group>
				<div>
					<Button variant="primary" type="submit" disabled={loading}>
						{t("login--button-login")}
					</Button>
				</div>
			</Form>
		</Container>
	)
}

interface SubmitTargets {
	username: HTMLInputElement
	password: HTMLInputElement
}
