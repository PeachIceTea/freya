import classNames from "classnames"
import { NavDropdown } from "react-bootstrap"
import { MdDarkMode, MdLightMode, MdMonitor } from "react-icons/md"

import { useTheme } from "../common"
import { useStore } from "../store"

export default function ThemeSwitcher() {
	const store = useStore()
	const [theme, setTheme] = useTheme()

	const icon = theme === "light" ? <MdLightMode /> : <MdDarkMode />

	return (
		<NavDropdown title={icon} align="end">
			<NavDropdown.Item
				onClick={() => {
					setTheme("light")
				}}
				className={classNames({
					active: store.theme === "light",
				})}
			>
				<MdLightMode className="me-2" />
				Light
			</NavDropdown.Item>
			<NavDropdown.Item
				onClick={() => {
					setTheme("dark")
				}}
				className={classNames({
					active: store.theme === "dark",
				})}
			>
				<MdDarkMode className="me-2" />
				Dark
			</NavDropdown.Item>
			<NavDropdown.Divider />
			<NavDropdown.Item
				onClick={() => {
					store.setTheme("system")
				}}
				className={classNames({
					active: store.theme === "system",
				})}
			>
				<MdMonitor className="me-2" />
				System
			</NavDropdown.Item>
		</NavDropdown>
	)
}
