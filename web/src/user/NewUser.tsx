import { useState } from "react"
import { Container } from "react-bootstrap"
import { useLocation } from "wouter"

import { createUser } from "../api/user"
import { useTitle } from "../common"
import { useLocale } from "../locales"
import UserForm, { UserFormData } from "./components/UserForm"

export default function NewUser() {
	const t = useLocale()
	const [_location, setLocation] = useLocation()

	const title = t("new-user--title")
	useTitle(title, undefined, true)

	const [error, setError] = useState<string | null>(null)
	const [isLoading, setIsLoading] = useState(false)

	async function handleSubmit({
		username,
		password,
		passwordConfirm,
		admin,
	}: Required<UserFormData>) {
		if (isLoading) return

		setIsLoading(true)
		setError(null)

		if (password !== passwordConfirm) {
			setError("Passwords do not match")
			setIsLoading(false)
			return
		}

		const res = await createUser({
			name: username,
			password,
			admin,
		})
		if (!res.success) {
			setError(res.errorCode)
			setIsLoading(false)
			return
		}

		setIsLoading(false)
		setLocation("/user-management")
	}

	return (
		<Container>
			<h1>{title}</h1>
			{error && <p className="text-danger">{t(error)}</p>}
			<UserForm
				onSubmit={handleSubmit}
				showAdminToggle={true}
				required={true}
				disable={isLoading}
			/>
		</Container>
	)
}
