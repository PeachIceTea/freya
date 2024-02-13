import { Container } from "react-bootstrap"
import { Redirect, Route, Switch, useLocation } from "wouter"

import AdminPage from "../Admin"
import Login from "../Login"
import BookDetails from "../book/BookDetails"
import Books from "../book/Books"
import Library from "../book/Library"
import NewBook from "../book/NewBook"
import { useQuery } from "../common"
import { useStore } from "../store"
import NewUser from "../user/NewUser"
import UserEdit from "../user/UserEdit"
import UserManagement from "../user/UserManagement"

export default function Router() {
	const state = useStore()

	return (
		<main className="overflow-auto h-100">
			{state.sessionInfo === null ? (
				<Switch>
					<Route path="/login">
						<Login />
					</Route>

					<Route>
						<CatchUnauthorized />
					</Route>
				</Switch>
			) : (
				<Switch>
					<Route path="/">
						<Redirect to="/library" replace={true} />
					</Route>

					<Route path="/library">
						<Library />
					</Route>

					<Route path="/book">
						<Books />
					</Route>

					{state.sessionInfo.admin && (
						<Route path="/book/new">
							<NewBook />
						</Route>
					)}

					<Route path="/book/:id">
						<BookDetails />
					</Route>

					{state.sessionInfo.admin && (
						<Route path="/user-management">
							<UserManagement />
						</Route>
					)}
					{state.sessionInfo.admin && (
						<Route path="/user/new">
							<NewUser />
						</Route>
					)}
					{state.sessionInfo.admin && (
						<Route path="/admin">
							<AdminPage />
						</Route>
					)}

					<Route path="/user/:id/edit">
						<UserEdit />
					</Route>

					<Route path="/login">
						<RedirectBack />
					</Route>

					<Route>
						<Container>
							<h1>Not Found</h1>
						</Container>
					</Route>
				</Switch>
			)}
		</main>
	)
}

// Redirect user to login page if not logged in but store the location they came from
function CatchUnauthorized() {
	const [location] = useLocation()
	if (location.startsWith("/login")) {
		return null
	} else {
		const from = location === "/" ? "" : `?from=${location}`
		return <Redirect to={`/login${from}`} replace={true} />
	}
}

// Redirect user to the page they came from
function RedirectBack() {
	const query = useQuery()
	const from = query.get("from")
	return <Redirect to={from ?? "/"} replace={true} />
}
