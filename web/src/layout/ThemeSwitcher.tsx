import { Badge, NavDropdown } from "react-bootstrap"
import { MdDarkMode, MdLightMode, MdMonitor } from "react-icons/md"

import { useTheme } from "../common"
import { useLocale } from "../locales"
import { useStore } from "../store"

export default function ThemeSwitcher() {
	const t = useLocale()
	const store = useStore()
	const [theme, setTheme] = useTheme()

	const icon = theme === "light" ? <MdLightMode /> : <MdDarkMode />

	return (
		<NavDropdown title={icon} align="end">
			<NavDropdown.Item
				onClick={() => {
					setTheme("light")
				}}
			>
				<MdLightMode className="me-2" />
				{t("navbar--theme-light")}
				{store.theme === "light" && (
					<Badge bg="primary" className="ms-2">
						{t("navbar--theme-active")}
					</Badge>
				)}
			</NavDropdown.Item>
			<NavDropdown.Item
				onClick={() => {
					setTheme("dark")
				}}
			>
				<MdDarkMode className="me-2" />
				{t("navbar--theme-dark")}
				{store.theme === "dark" && (
					<Badge bg="primary" className="ms-2">
						{t("navbar--theme-active")}
					</Badge>
				)}
			</NavDropdown.Item>
			<NavDropdown.Divider />
			<NavDropdown.Item
				onClick={() => {
					store.setTheme("system")
				}}
			>
				<MdMonitor className="me-2" />
				{t("navbar--theme-system")}
				{store.theme === "system" && (
					<Badge bg="primary" className="ms-2">
						{t("navbar--theme-active")}
					</Badge>
				)}
			</NavDropdown.Item>
		</NavDropdown>
	)
}
