import { Container, Nav, NavDropdown, Navbar } from "react-bootstrap"
import { MdLogin, MdLogout } from "react-icons/md"
import { Link } from "wouter"

import { logout } from "../api/authentication"
import { useLocale } from "../locales"
import { useStore } from "../store"
import ThemeSwitcher from "./ThemeSwitcher"

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
		<Navbar bg="dark" variant="dark" sticky="top" collapseOnSelect expand="md">
			<Container>
				<Link to="/" className="text-decoration-none">
					<Navbar.Brand>{t("app--title")}</Navbar.Brand>
				</Link>

				<Navbar.Toggle aria-controls="navbar-nav" />

				<Navbar.Collapse id="navbar-nav" className="justify-content-end">
					<Nav className="ml-auto">
						<ThemeSwitcher />
						{state.sessionInfo === null ? (
							<Link to="/login" className="nav-link">
								<MdLogin />
								{t("navbar--login")}
							</Link>
						) : (
							<NavDropdown title={state.sessionInfo.username} align="end">
								<NavDropdown.Item onClick={handleLogout}>
									<MdLogout />
									{t("navbar--logout")}
								</NavDropdown.Item>
							</NavDropdown>
						)}
					</Nav>
				</Navbar.Collapse>
			</Container>
		</Navbar>
	)
}
