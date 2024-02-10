import { Container, Nav, NavDropdown, Navbar } from "react-bootstrap"
import { MdLogin, MdLogout } from "react-icons/md"
import { TbBookUpload, TbUser, TbUserEdit } from "react-icons/tb"
import { Link } from "wouter"

import { logout } from "../api/authentication"
import { capitalize } from "../common"
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
		<Navbar
			bg="dark"
			variant="dark"
			sticky="top"
			collapseOnSelect
			expand="md"
			className="shadow-sm px-2"
		>
			<Container>
				<Link to="/" className="text-decoration-none">
					<Navbar.Brand>{t("app--title")}</Navbar.Brand>
				</Link>

				<Navbar.Toggle aria-controls="navbar-nav" />

				<Navbar.Collapse id="navbar-nav" className="justify-content-end">
					<Nav className="ml-auto">
						<ThemeSwitcher />
						{state.sessionInfo?.admin && (
							<NavDropdown title={t("navbar--admin")} align="end">
								<NavDropdown.Item as={Link} to="/user-management">
									<TbUser className="me-2" />
									{t("navbar--user-management")}
								</NavDropdown.Item>
								<NavDropdown.Item as={Link} to="/book/new">
									<TbBookUpload className="me-2" />
									{t("navbar--new-book")}
								</NavDropdown.Item>
							</NavDropdown>
						)}
						{state.sessionInfo === null ? (
							<Link to="/login" className="nav-link">
								<MdLogin className="me-2" />
								{t("navbar--login")}
							</Link>
						) : (
							<NavDropdown
								title={capitalize(state.sessionInfo.username)}
								align="end"
							>
								<NavDropdown.Item
									as={Link}
									to={`/user/${state.sessionInfo.userId}/edit`}
								>
									<TbUserEdit className="me-2" />
									{t("navbar--edit-profile")}
								</NavDropdown.Item>
								<NavDropdown.Divider />
								<NavDropdown.Item onClick={handleLogout}>
									<MdLogout className="me-2" />
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
