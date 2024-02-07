import { useEffect, useState } from "react"
import { Button, Form, InputGroup, ListGroup, Modal } from "react-bootstrap"
import {
	MdAudioFile,
	MdDescription,
	MdFolder,
	MdImage,
	MdKeyboardArrowUp,
	MdKeyboardReturn,
} from "react-icons/md"
import { useImmer } from "use-immer"

import { Entry, FileCategory, useDirectoryEntries } from "../../api/fs"
import { useLocale } from "../../locales"

export function FileSelect({
	id,
	fileCategory,
	multiSelect,
	onChange,
}: {
	id: string
	fileCategory: FileCategory
	multiSelect: boolean
	onChange: (files: Entry[]) => void
}) {
	const t = useLocale()

	// Data.
	const [path, setPath] = useState<string>("")
	const [editFiles, setEditFiles] = useImmer<Entry[]>([])
	const [files, _setFiles] = useState<Entry[]>([])
	function setFiles(files: Entry[]) {
		_setFiles(files)
		onChange(files)
	}
	const { data } = useDirectoryEntries(path)

	// Modal
	const [showModal, setShowModal] = useState(false)
	const [pathInput, setPathInput] = useState(path)

	function openModal() {
		setShowModal(true)
		setEditFiles(files)
	}

	function exitModal() {
		setShowModal(false)
	}

	function selectFiles() {
		setShowModal(false)
		setFiles(editFiles)
	}

	// Update path and path input when path changes.
	// Setting the path is necessary for the first request since we don't know the default path set on the server.
	useEffect(() => {
		if (data?.path) {
			setPath(data.path)
			setPathInput(data.path)

			// Reset files when path changes
			setEditFiles(() => [])
		}
	}, [data, path, setEditFiles])

	// Handle click on file.
	function handleFileClick(entry: Entry) {
		if (entry.category === "directory") {
			return setPath(entry.path)
		}

		if (entry.category !== fileCategory) {
			return
		}

		if (multiSelect) {
			setEditFiles(draft => {
				const index = draft.findIndex(file => file.name === entry.name)
				if (index === -1) {
					draft.push(entry)
				} else {
					draft.splice(index, 1)
				}
			})
		} else {
			setEditFiles(() => [entry])
		}
	}

	// Select all files in current directory with the correct category.
	function selectAllFiles() {
		setEditFiles(
			() =>
				data?.directory.filter(entry => entry.category === fileCategory) ?? [],
		)
	}

	// Convert file list to array.
	const fileList =
		data?.directory.map(entry => {
			return (
				<ListGroup.Item
					variant="action"
					key={entry.name}
					role={
						entry.category === "directory" || entry.category === fileCategory
							? "button"
							: "none"
					}
					active={editFiles.some(file => file.path === entry.path)}
					onClick={() => handleFileClick(entry)}
				>
					<CategoryIcon category={entry.category} />
					{entry.name}
				</ListGroup.Item>
			)
		}) || []

	return (
		<>
			<InputGroup>
				<Button id={id} variant="secondary" onClick={openModal}>
					{t("new-book--select-file-button", {
						multiSelect: Number(multiSelect),
					})}
				</Button>
				<label
					className="form-control user-select-none"
					htmlFor={id}
					role="button"
				>
					{t("new-book--files-selected", {
						count: files.length,
						fileName: files[0]?.name,
					})}
				</label>
			</InputGroup>

			<Modal show={showModal} size="xl" onHide={exitModal}>
				<Modal.Header>
					<InputGroup className="flex-grow-1">
						<Button
							variant="outline-secondary"
							onClick={() => data?.parentPath && setPath(data.parentPath)}
						>
							<MdKeyboardArrowUp />
						</Button>
						<Form.Control
							type="text"
							value={pathInput}
							onChange={e => setPathInput(e.target.value)}
							onKeyDown={e => e.key === "Enter" && setPath(pathInput)}
						/>
						<Button
							variant="outline-secondary"
							onClick={() => setPath(pathInput)}
						>
							<MdKeyboardReturn />
						</Button>
					</InputGroup>
				</Modal.Header>
				<Modal.Body>
					<ListGroup>{fileList}</ListGroup>
				</Modal.Body>
				<Modal.Footer>
					<Button
						variant="primary"
						className="me-auto"
						onClick={selectAllFiles}
					>
						{t("new-book--modal-select-all-button")}
					</Button>
					<Button variant="primary" onClick={selectFiles}>
						{t("new-book--modal-select-button")}
					</Button>
					<Button variant="secondary" onClick={exitModal}>
						{t("new-book--modal-cancel-button")}
					</Button>
				</Modal.Footer>
			</Modal>
		</>
	)
}

function CategoryIcon({ category }: { category: FileCategory }) {
	let icon
	switch (category) {
		case "audio":
			icon = <MdAudioFile />
			break
		case "image":
			icon = <MdImage />
			break
		case "directory":
			icon = <MdFolder />
			break
		default:
			icon = <MdDescription />
	}
	return <span className="me-2">{icon}</span>
}
