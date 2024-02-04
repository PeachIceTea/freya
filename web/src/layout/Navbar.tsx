import { Nav, Navbar } from "react-bootstrap"
import { MdLogin, MdLogout } from "react-icons/md"
import { Link } from "wouter"

import { logout } from "../api/authentication"
import { useLocale } from "../locales"
import { useStore } from "../store"

export default function FreyaNavbar() {
	const t = useLocale()
	const state = useStore()

	async function handleLogout(
		event: React.MouseEvent<HTMLAnchorElement, MouseEvent>,
	) {
		event.preventDefault()
		await logout()
	}

	return (
		<Navbar bg="dark" variant="dark">
			<Link to="/" className="text-decoration-none">
				<Navbar.Brand>{t("app--title")}</Navbar.Brand>
			</Link>

			<Navbar.Toggle aria-controls="navbar-nav" />

			<Navbar.Collapse id="navbar-nav" className="justify-content-end">
				<Nav className="ml-auto">
					{state.sessionInfo === null ? (
						<Link to="/login" className="nav-link">
							<MdLogin />
							{t("navbar--login")}
						</Link>
					) : (
						<Nav.Link href="/logout" onClick={handleLogout} active={false}>
							<MdLogout />
							{t("navbar--logout")}
						</Nav.Link>
					)}
				</Nav>
			</Navbar.Collapse>
		</Navbar>
	)
}
