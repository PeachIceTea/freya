import dayjs from "dayjs"
import { Badge, Container, Table } from "react-bootstrap"
import { Link } from "wouter"

import { useUsers } from "./api/user"
import { useTitle } from "./common"
import { useLocale } from "./locales"

export default function UserManagement() {
	useTitle("user-management--title")
	const t = useLocale()

	const { users, error, isLoading } = useUsers()

	// Guards
	if (isLoading) {
		return <Container>Loading...</Container>
	}

	if (error || !users) {
		return <Container>Error: {error?.errorCode}</Container>
	}

	return (
		<Container>
			<div className="d-flex justify-content-between align-items-center">
				<h1>{t("user-management--title")}</h1>
			</div>
			<Table striped bordered hover>
				<thead>
					<tr>
						<th>{t("user-management--id")}</th>
						<th>{t("user-management--name")}</th>
						<th>{t("user-management--created")}</th>
						<th>{t("user-management--modified")}</th>
						<th>{t("user-management--actions")}</th>
					</tr>
				</thead>
				<tbody>
					{users.map(user => (
						<tr key={user.id}>
							<td>{user.id}</td>
							<td>
								{user.name}
								{user.admin && (
									<Badge bg="primary" className="ms-2">
										{t("user-management--admin")}
									</Badge>
								)}
							</td>
							<td>{dayjs(user.created).format("YYYY-MM-DD HH:mm:ss")}</td>
							<td>{dayjs(user.created).format("YYYY-MM-DD HH:mm:ss")}</td>
							<td className="d-flex gap-2">
								<Link to={`/user/${user.id}`} className="btn btn-secondary">
									{t("user-management--show-profile")}
								</Link>
								<Link to={`/user/${user.id}/edit`} className="btn btn-primary">
									{t("user-management--edit")}
								</Link>
							</td>
						</tr>
					))}
				</tbody>
			</Table>
		</Container>
	)
}
