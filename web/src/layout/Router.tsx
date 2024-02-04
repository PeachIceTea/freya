import { Redirect, Route, Switch, useLocation } from "wouter"

import Login from "../Login"
import Book from "../book/Book"
import Books from "../book/Books"
import NewBook from "../book/NewBook"
import { useQuery } from "../common"
import { useStore } from "../store"

export default function Router() {
	const state = useStore()

	return state.sessionInfo === null ? (
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
				<Redirect to="/book" />
			</Route>

			<Route path="/book">
				<Books />
			</Route>

			<Route path="/book/new">
				<NewBook />
			</Route>

			<Route path="/book/:id">
				<Book />
			</Route>

			<Route path="/login">
				<RedirectBack />
			</Route>

			<Route>
				<h1>Not Found</h1>
			</Route>
		</Switch>
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
	return <Redirect to={from || "/"} replace={true} />
}
