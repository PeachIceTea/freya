import { Button, Form } from "react-bootstrap"

import { useLocale } from "../../locales"

export default function UserForm({
	onSubmit,
	showAdminToggle,
	disable,
	required,

	name,
	admin,
}: UserFormProps) {
	const t = useLocale()

	function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
		event.preventDefault()

		const targets = event.target as unknown as SubmitTargets
		const username = targets.username.value
		const password = targets.password.value
		const passwordConfirm = targets.passwordConfirm.value
		const admin = targets.admin?.checked

		onSubmit({
			username: username,
			password: password,
			passwordConfirm: passwordConfirm,
			admin: admin,
		})
	}

	return (
		<Form onSubmit={handleSubmit}>
			<Form.Group className="mb-3">
				<Form.Label>{t("user-edit--name")}</Form.Label>
				<Form.Control
					type="text"
					defaultValue={name}
					name="username"
					required={required}
				/>
			</Form.Group>
			<Form.Group className="mb-3">
				<Form.Label>{t("user-edit--password")}</Form.Label>
				<Form.Control type="password" name="password" required={required} />
			</Form.Group>
			<Form.Group className="mb-3">
				<Form.Label>{t("user-edit--password-confirm")}</Form.Label>
				<Form.Control
					type="password"
					name="passwordConfirm"
					required={required}
				/>
			</Form.Group>
			{showAdminToggle && (
				<Form.Group className="mb-3">
					<Form.Label>{t("user-edit--admin")}</Form.Label>
					<Form.Check type="checkbox" defaultChecked={admin} name="admin" />
				</Form.Group>
			)}
			<Form.Group>
				<Button variant="primary" type="submit" disabled={disable}>
					{t("user-edit--submit")}
				</Button>
			</Form.Group>
		</Form>
	)
}

type UserFormProps =
	| {
			onSubmit: (data: Required<UserFormData>) => void
			showAdminToggle: boolean
			disable: boolean
			required: true
			name?: string
			admin?: boolean
			// eslint-disable-next-line no-mixed-spaces-and-tabs
	  }
	| {
			onSubmit: (data: UserFormData) => void
			showAdminToggle: boolean
			disable: boolean
			required: false
			name?: string
			admin?: boolean
			// eslint-disable-next-line no-mixed-spaces-and-tabs
	  }

export interface UserFormData {
	username?: string
	password?: string
	passwordConfirm?: string
	admin: boolean
}

interface SubmitTargets {
	username: HTMLInputElement
	password: HTMLInputElement
	passwordConfirm: HTMLInputElement
	admin: HTMLInputElement
}
