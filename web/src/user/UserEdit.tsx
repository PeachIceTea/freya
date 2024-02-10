import { useState } from "react"
import { Alert, Button, Container, Form } from "react-bootstrap"
import { useLocation, useParams } from "wouter"

import { User, updateUser, useUser } from "../api/user"
import { useTitle } from "../common"
import { useLocale } from "../locales"
import { useStore } from "../store"
import UserForm, { UserFormData } from "./components/UserForm"

function UserEditComponent({ user }: { user: User }) {
	const t = useLocale()
	const { isAdmin } = useStore(store => ({
		isAdmin: store.sessionInfo!.admin,
	}))
	const { id, name, admin } = user

	const title = t("user-edit--title", {
		id,
	})
	useTitle(title, undefined, true)
	const [_location, setLocation] = useLocation()

	const [error, setError] = useState<string | null>(null)
	const [isLoading, setIsLoading] = useState(false)

	async function handleSubmit({
		username,
		password,
		passwordConfirm,
		admin,
	}: UserFormData) {
		if (isLoading) return

		setIsLoading(true)
		setError(null)

		if (password !== passwordConfirm) {
			setIsLoading(false)
			setError(t("user-edit--password-mismatch"))
			return
		}

		const res = await updateUser(id, {
			name: username ?? undefined,
			password: password ?? undefined,
			admin: isAdmin ? admin : undefined,
		})
		if (!res.success) {
			setError(t(res.errorCode))
		}

		setIsLoading(false)
		setLocation(`/user/${id}`)
	}

	return (
		<Container>
			<h1>{title}</h1>
			{error && <Alert variant="danger">{error}</Alert>}
			<UserForm
				onSubmit={handleSubmit}
				showAdminToggle={isAdmin}
				disable={isLoading}
				required={false}
				name={name}
				admin={admin}
			/>
		</Container>
	)
}

export default function UserEdit() {
	const t = useLocale()
	const { id } = useParams()
	const { userId, isAdmin } = useStore(store => ({
		userId: store.sessionInfo!.userId,
		isAdmin: store.sessionInfo!.admin,
	}))

	const { user, error, isLoading } = useUser(+id!)

	if (error) {
		console.error(error)
		return (
			<Container>
				<Alert variant="danger">Error: {t(error.errorCode)}</Alert>
			</Container>
		)
	}

	if (isLoading) {
		return null
	}

	if (!user) {
		return (
			<Container>
				<Alert variant="danger">User not found</Alert>
			</Container>
		)
	}

	if (!isAdmin && user.id !== userId) {
		return (
			<Container>
				<Alert variant="error">{t("user-edit--not-allowed")}</Alert>
			</Container>
		)
	}

	return <UserEditComponent user={user} />
}
