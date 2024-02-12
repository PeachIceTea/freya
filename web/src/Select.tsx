// Custom select component.
// Allows scrolling through a list of options and selecting one.
import { useEffect, useRef, useState } from "react"
import { ListGroup } from "react-bootstrap"

// React.HTMLAttributes<HTMLDivElement> without onChange.
type DivProps = Omit<React.HTMLAttributes<HTMLDivElement>, "onChange">

interface SelectOption<T> {
	value: T
	label: string
}

export default function Select<T extends string | number>({
	options,
	value,
	onChange,
	onOpenChange,
	...props
}: {
	options: SelectOption<T>[]
	value: T
	onChange: (value: T) => void
	onOpenChange?: (isOpen: boolean) => void
} & DivProps) {
	const [open, _setOpen] = useState(false)
	function setOpen(value: boolean) {
		_setOpen(value)
		onOpenChange?.(value)
	}
	const dropDownRef = useRef<HTMLDivElement>(null)

	// Listen to click events everywhere in the document.
	// If the user clicks outside of the dropdown, close it.
	useEffect(() => {
		function handleClick(event: MouseEvent) {
			if (
				dropDownRef.current &&
				!dropDownRef.current.contains(event.target as Node)
			) {
				setOpen(false)
			}
		}
		document.addEventListener("click", handleClick)
	}, [dropDownRef, _setOpen])

	return (
		<div className="position-relative" ref={dropDownRef} {...props}>
			<button className="form-select" onClick={() => setOpen(!open)}>
				{value}
			</button>
			<div
				className="position-absolute w-100 z-1 bg-body mb-1 overflow-y-auto rounded bottom-100"
				hidden={!open}
			>
				<ListGroup>
					{options.map(option => (
						<ListGroup.Item
							variant="action"
							key={option.value}
							onClick={() => {
								onChange(option.value)
								setOpen(false)
							}}
							role="button"
							active={option.value === value}
						>
							{option.label}
						</ListGroup.Item>
					))}
				</ListGroup>
			</div>
		</div>
	)
}
